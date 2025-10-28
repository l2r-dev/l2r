use super::{DbConnection, error::DbError};
use crate::utils::AllocatedReflectExt;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_mod_scripting::{bindings::InteropError, prelude::ScriptValue};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, DeleteMany, DeleteResult,
    EntityTrait, InsertResult, IntoActiveModel, PaginatorTrait, PrimaryKeyTrait, QueryFilter,
    QuerySelect, Statement, TransactionTrait, UpdateMany, UpdateResult,
    prelude::async_trait::async_trait,
    sea_query::{IntoCondition, OnConflict},
};
use std::{any::Any, marker::PhantomData, sync::Arc};

pub struct DbRepositoryPlugin;
impl Plugin for DbRepositoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RepositoryManager>();
        app.add_systems(Update, manage_repositories_system);
    }
}

pub fn manage_repositories_system(
    db_connection: Res<DbConnection>,
    mut repository_registry: ResMut<RepositoryManager>,
) {
    if db_connection.is_changed() {
        repository_registry.update_each_connection(db_connection.connection());
    }
}

/**
A trait providing a generic interface for database operations.

# Type Parameters

* `PK` - The type used for the primary key
* `T` - The entity type that implements `EntityTrait`

# Examples
```rust,ignore
// Creating a new repository for Account entities
type AccountRepository = DbRepository<Uuid, Entity>;
let account_repository = AccountRepository::new(db_connection);

// Fetching an account by ID
let account = account_repository.find_by_id(account_id).await?;
```
*/
#[async_trait]
pub trait Repository<PK, T>
where
    T: EntityTrait + Clone + Send + Sync,
    T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync,
    T::ActiveModel: Send + Sync,
    PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send + Sync,
{
    /// Creates a new database record using the provided data model.
    ///
    /// # Returns
    ///
    /// The created model with any database-generated fields populated.
    async fn create(&self, data: &T::Model) -> Result<T::Model, DbError>;

    /// Creates multiple database records in a single operation.
    ///
    /// # Returns
    ///
    /// A vector containing all created models.
    async fn create_many<I>(&self, data: I) -> Result<Vec<T::Model>, DbError>
    where
        I: IntoIterator<Item = T::Model> + Send,
        I::IntoIter: Send,
    {
        let mut result = Vec::new();
        for item in data {
            let created_item = self.create(&item).await?;
            result.push(created_item);
        }
        Ok(result)
    }

    /// Updates an existing database record.
    ///
    /// # Parameters
    ///
    /// * `data` - The active model containing the changes to apply
    async fn update(&self, data: &T::ActiveModel) -> Result<(), DbError>;

    /// Updates multiple records within a single database transaction.
    ///
    /// If any update fails, the entire transaction is rolled back.
    async fn update_in_transaction(&self, models: &[T::ActiveModel]) -> Result<(), DbError>;

    /// Creates a new record or updates an existing one based on the conflict policy.
    ///
    /// # Parameters
    ///
    /// * `data` - The model to create or update
    /// * `on_conflict` - The strategy to use when a conflict occurs
    async fn create_or_update(
        &self,
        data: &T::Model,
        on_conflict: OnConflict,
    ) -> Result<InsertResult<T::ActiveModel>, DbError>;

    /// Updates multiple records using a query builder.
    ///
    /// # Parameters
    ///
    /// * `builder_fn` - A function that configures the update operation
    async fn update_many<F>(&self, builder_fn: F) -> Result<UpdateResult, DbError>
    where
        F: FnOnce(UpdateMany<T>) -> UpdateMany<T> + Send;

    /// Deletes a database record.
    async fn delete(&self, data: &T::Model) -> Result<(), DbError>;

    /// Deletes a record by its primary key.
    async fn delete_by_id(&self, id: PK) -> Result<(), DbError>;

    /// Deletes multiple records by their primary keys in a single transaction.
    /// use into iter here
    async fn delete_by_ids<I>(&self, ids: I) -> Result<(), DbError>
    where
        I: IntoIterator<Item = PK> + Send,
        I::IntoIter: Send;

    async fn delete_many<F>(&self, builder_fn: F) -> Result<DeleteResult, DbError>
    where
        F: FnOnce(DeleteMany<T>) -> DeleteMany<T> + Send;

    /// Finds a record by its primary key.
    ///
    /// # Returns
    ///
    /// `None` if no record with the given ID exists.
    async fn find_by_id(&self, id: PK) -> Result<Option<T::Model>, DbError>;

    /// Finds multiple records by their primary keys.
    ///
    /// # Returns
    ///
    /// A vector containing only the records that were found.
    async fn find_by_ids(&self, ids: Vec<PK>) -> Result<Vec<T::Model>, DbError>;

    /// Finds records that match the given conditions.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// repository.find_with_conditions([
    ///     account::Column::Name.eq("username"),
    ///     account::Column::Active.eq(true)
    /// ]).await?;
    ///
    /// repository.find_with_conditions([
    ///     item::model::Column::OwnerId.is_null()
    /// ]).await?;
    /// ```
    async fn find_with_conditions<I>(&self, conditions: I) -> Result<Vec<T::Model>, DbError>
    where
        I: IntoIterator + Send,
        I::Item: IntoCondition + Send;

    /// Fetches records with pagination support.
    ///
    /// # Returns
    ///
    /// A tuple containing the records for the requested page and the total number of pages.
    async fn find_with_pagination(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<T::Model>, u64), DbError>;

    /// Retrieves values from a specific column for all records.
    async fn list_column<C>(&self, column: T::Column) -> Result<Vec<C>, DbError>
    where
        C: Send + sea_orm::TryGetable;
}

pub struct DbRepository<PK, T> {
    conn: Arc<DatabaseConnection>,
    pub name: Arc<str>,
    _phantom_pk: PhantomData<PK>,
    _phantom: PhantomData<T>,
}

impl<PK, T> Clone for DbRepository<PK, T> {
    fn clone(&self) -> Self {
        Self {
            conn: self.conn.clone(),
            name: self.name.clone(),
            _phantom_pk: PhantomData,
            _phantom: PhantomData,
        }
    }
}

impl<PK, T> DbRepository<PK, T>
where
    T: EntityTrait + Clone + Send + Sync,
    T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync,
    T::ActiveModel: Send + Sync,
    PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
        + Send
        + Sync
        + 'static,
{
    /// Creates a new repository instance.
    pub fn new(name: &str) -> Self {
        Self {
            // Connection will be set later by RepositoryManager
            conn: Arc::new(DatabaseConnection::default()),
            name: Arc::from(name),
            _phantom_pk: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Get the database backend for this repository connection.
    pub fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }

    /// Creates a Statement from raw SQL string using the repository's database backend.
    ///
    /// # Parameters
    ///
    /// * `sql` - The raw SQL string
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let stmt = repository.statement_from_string("SELECT COUNT(*) FROM accounts");
    /// let result = repository.query_one_raw(stmt).await?;
    /// ```
    pub fn statement_from_string(&self, sql: &str) -> Statement {
        Statement::from_string(self.get_database_backend(), sql)
    }

    /// Creates a Statement from SQL with parameter values using the repository's database backend.
    ///
    /// # Parameters
    ///
    /// * `sql` - The SQL string with parameter placeholders (? for MySQL/SQLite, $N for PostgreSQL)
    /// * `values` - The parameter values to bind
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let stmt = repository.statement_from_sql_and_values(
    ///     "SELECT * FROM accounts WHERE id = ? AND active = ?",
    ///     [account_id.into(), true.into()]
    /// );
    /// let accounts = repository.find_by_raw_sql(stmt).await?;
    /// ```
    pub fn statement_from_sql_and_values<I>(&self, sql: &str, values: I) -> Statement
    where
        I: IntoIterator<Item = sea_orm::Value>,
    {
        Statement::from_sql_and_values(self.get_database_backend(), sql, values)
    }
}

#[async_trait]
impl<PK, T> Repository<PK, T> for DbRepository<PK, T>
where
    T: EntityTrait + Clone + Send + Sync,
    T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync,
    T::ActiveModel: Send + Sync,
    PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send + Sync,
{
    async fn create(&self, data: &T::Model) -> Result<T::Model, DbError> {
        let model = data.clone().into_active_model();
        Ok(T::insert(model)
            .exec_with_returning(self.conn.as_ref())
            .await
            .map_err(|e: sea_orm::DbErr| DbError::CreateError(e))?)
    }

    async fn update(&self, data: &T::ActiveModel) -> Result<(), DbError> {
        let model = data.clone();
        T::update(model)
            .exec(self.conn.as_ref())
            .await
            .map_err(|e: sea_orm::DbErr| DbError::UpdateError(e))?;
        Ok(())
    }

    async fn update_in_transaction(&self, models: &[T::ActiveModel]) -> Result<(), DbError> {
        let txn = self.conn.begin().await.map_err(DbError::RollbackError)?;

        for model in models {
            if let Err(e) = T::update(model.clone()).exec(&txn).await {
                txn.rollback().await.map_err(DbError::RollbackError)?;
                return Err(DbError::UpdateError(e));
            }
        }

        txn.commit().await.map_err(DbError::RollbackError)?;

        Ok(())
    }

    async fn create_or_update(
        &self,
        data: &T::Model,
        on_conflict: OnConflict,
    ) -> Result<InsertResult<T::ActiveModel>, DbError> {
        let model = data.clone().into_active_model();

        T::insert(model)
            .on_conflict(on_conflict)
            .exec(self.conn.as_ref())
            .await
            .map_err(DbError::CreateError)
    }

    async fn update_many<F>(&self, builder_fn: F) -> Result<UpdateResult, DbError>
    where
        F: FnOnce(UpdateMany<T>) -> UpdateMany<T> + Send,
    {
        let update_builder = T::update_many();
        let configured_update = builder_fn(update_builder);

        configured_update
            .exec(self.conn.as_ref())
            .await
            .map_err(DbError::UpdateError)
    }

    async fn delete(&self, data: &T::Model) -> Result<(), DbError> {
        let model = data.clone().into_active_model();
        T::delete(model)
            .exec(self.conn.as_ref())
            .await
            .map_err(|e: sea_orm::DbErr| DbError::DeleteError(e))?;
        Ok(())
    }

    async fn delete_by_id(&self, id: PK) -> Result<(), DbError> {
        T::delete_by_id(id)
            .exec(self.conn.as_ref())
            .await
            .map_err(DbError::DeleteError)?;
        Ok(())
    }

    async fn delete_by_ids<I>(&self, ids: I) -> Result<(), DbError>
    where
        I: IntoIterator<Item = PK> + Send,
        I::IntoIter: Send,
    {
        let txn = self.conn.begin().await.map_err(DbError::RollbackError)?;

        for id in ids {
            T::delete_by_id(id)
                .exec(&txn)
                .await
                .map_err(DbError::RollbackError)?;
        }

        txn.commit().await.map_err(DbError::RollbackError)?;

        Ok(())
    }

    async fn delete_many<F>(&self, builder_fn: F) -> Result<DeleteResult, DbError>
    where
        F: FnOnce(DeleteMany<T>) -> DeleteMany<T> + Send,
    {
        let delete_builder = T::delete_many();
        let configured_delete = builder_fn(delete_builder);

        configured_delete
            .exec(self.conn.as_ref())
            .await
            .map_err(DbError::DeleteError)
    }

    async fn find_by_id(&self, id: PK) -> Result<Option<T::Model>, DbError> {
        let result = T::find_by_id(id)
            .one(self.conn.as_ref())
            .await
            .map_err(DbError::ReadError)?;
        Ok(result)
    }

    async fn find_by_ids(&self, ids: Vec<PK>) -> Result<Vec<T::Model>, DbError> {
        let mut result = Vec::new();

        for id in ids {
            if let Some(model) = T::find_by_id(id)
                .one(self.conn.as_ref())
                .await
                .map_err(DbError::ReadError)?
            {
                result.push(model);
            }
        }

        Ok(result)
    }

    async fn find_with_conditions<I>(&self, conditions: I) -> Result<Vec<T::Model>, DbError>
    where
        I: IntoIterator + Send,
        I::Item: IntoCondition + Send,
    {
        let mut query = T::find();

        for condition in conditions {
            query = query.filter(condition);
        }

        query
            .all(self.conn.as_ref())
            .await
            .map_err(DbError::ReadError)
    }

    async fn find_with_pagination(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<T::Model>, u64), DbError> {
        let paginator = T::find().paginate(self.conn.as_ref(), page_size);
        let num_pages = paginator
            .num_pages()
            .await
            .map_err(DbError::PaginationError)?;

        let result = paginator
            .fetch_page(page - 1)
            .await
            .map(|p| (p, num_pages))
            .map_err(DbError::PaginationError)?;

        Ok(result)
    }

    async fn list_column<C>(&self, column: T::Column) -> Result<Vec<C>, DbError>
    where
        C: Send + sea_orm::TryGetable,
    {
        Ok(T::find()
            .select_only()
            .column(column)
            .into_tuple()
            .all(self.conn.as_ref())
            .await
            .map_err(DbError::ReadError)?)
    }
}

/// Object-safe trait for type-erased repository operations
#[async_trait]
pub trait AnyRepository: Send + Sync {
    /// Get the repository name/identifier
    fn name(&self) -> &str;

    /// Get type information about the repository
    fn get_type_info(&self) -> RepositoryTypeInfo;

    /// Downcast to Any for type recovery
    fn as_any(&self) -> &dyn Any;

    fn connection(&self) -> &Arc<DatabaseConnection>;
    fn set_connection(&mut self, new_conn: Arc<DatabaseConnection>);
    fn ready(&self) -> bool;
}

/// Information about repository types for debugging/introspection
#[derive(Clone, Debug)]
pub struct RepositoryTypeInfo {
    pub name: String,
    pub primary_key_type: &'static str,
    pub entity_type: &'static str,
}

#[async_trait]
impl<PK, T> AnyRepository for DbRepository<PK, T>
where
    T: EntityTrait + Clone + Send + Sync + 'static,
    T::Model: IntoActiveModel<T::ActiveModel>
        + Send
        + Sync
        + PrimaryKeyColumns
        + UpdatableModel
        + 'static,
    T::ActiveModel: Send + Sync + 'static,
    PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
        + Send
        + Sync
        + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn get_type_info(&self) -> RepositoryTypeInfo {
        RepositoryTypeInfo {
            name: self.name.to_string(),
            primary_key_type: std::any::type_name::<PK>(),
            entity_type: std::any::type_name::<T>(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn connection(&self) -> &Arc<DatabaseConnection> {
        &self.conn
    }

    fn set_connection(&mut self, new_conn: Arc<DatabaseConnection>) {
        self.conn = new_conn;
    }

    fn ready(&self) -> bool {
        !matches!(*self.conn, DatabaseConnection::Disconnected)
    }
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct RepositoryManager(HashMap<String, Box<dyn AnyRepository>>);

pub trait TypedRepositoryManager {
    /// Get a repository by name, with type checking
    fn typed_by_name<PK, T>(&self, name: &str) -> Result<DbRepository<PK, T>>
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync + 'static,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static;

    /// Get a repository by type, searching through all repositories and using downcast
    fn typed<PK, T>(&self) -> Result<DbRepository<PK, T>>
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync + 'static,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static;

    /// Try to get a repository by type, returning InteropError on failure
    fn typed_interop<PK, T>(&self) -> Result<DbRepository<PK, T>, InteropError>
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync + 'static,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static;
}

impl RepositoryManager {
    /// Register a new repository with a given name
    pub fn register<PK, T>(&mut self, repository: DbRepository<PK, T>) -> &mut Self
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel>
            + Send
            + Sync
            + PrimaryKeyColumns
            + UpdatableModel
            + 'static,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static,
    {
        let name = repository.name.to_string();
        self.insert(name, Box::new(repository));
        self
    }

    /// Get repository by name without type checking (returns trait object)
    pub fn get_any(&self, name: &str) -> Option<&dyn AnyRepository> {
        self.get(name).map(|r| r.as_ref())
    }

    pub fn any_ready(&self, name: &str) -> bool {
        match self.get(name) {
            Some(repo) => repo.ready(),
            None => false,
        }
    }

    pub fn all_ready(&self) -> bool {
        self.values().all(|repo| repo.ready())
    }

    /// List all registered repository names
    pub fn list_names(&self) -> Vec<&str> {
        self.keys().map(|k| k.as_str()).collect()
    }

    /// Get type information for all repositories
    pub fn list_type_info(&self) -> Vec<RepositoryTypeInfo> {
        self.values().map(|repo| repo.get_type_info()).collect()
    }

    pub fn update_each_connection(&mut self, new_conn: Arc<DatabaseConnection>) {
        for repo in self.values_mut() {
            repo.set_connection(new_conn.clone());
        }
    }

    pub fn is_mock(&self) -> bool {
        self.values().all(|repo| {
            matches!(
                **repo.connection(),
                DatabaseConnection::MockDatabaseConnection(_)
            )
        })
    }
}

impl ScriptingRepositoryManager for RepositoryManager {}
impl TypedRepositoryManager for RepositoryManager {
    /// Get a repository by name, with type checking
    fn typed_by_name<PK, T>(&self, name: &str) -> Result<DbRepository<PK, T>>
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync + 'static,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static,
    {
        match self.get(name) {
            Some(repo) => {
                if let Some(typed_repo) = repo.as_any().downcast_ref::<DbRepository<PK, T>>() {
                    Ok(typed_repo.clone())
                } else {
                    Err(BevyError::from(format!(
                        "Type mismatch for repository {}: expected PK {} and model {}",
                        name,
                        std::any::type_name::<PK>(),
                        std::any::type_name::<T>()
                    )))
                }
            }
            None => Err(BevyError::from(format!(
                "{} not found with PK {} and model {} in RepositoryManager",
                name,
                std::any::type_name::<PK>(),
                std::any::type_name::<T>()
            ))),
        }
    }

    /// Get a repository by type, searching through all repositories and using downcast
    fn typed<PK, T>(&self) -> Result<DbRepository<PK, T>>
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync + 'static,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static,
    {
        for (_name, repo) in self.iter() {
            if let Some(typed_repo) = repo.as_any().downcast_ref::<DbRepository<PK, T>>() {
                return Ok(typed_repo.clone());
            }
        }

        Err(BevyError::from(format!(
            "No repository found with PK {} and model {} in RepositoryManager",
            std::any::type_name::<PK>(),
            std::any::type_name::<T>()
        )))
    }

    fn typed_interop<PK, T>(&self) -> Result<DbRepository<PK, T>, InteropError>
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync + 'static,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static,
    {
        for (_name, repo) in self.iter() {
            if let Some(typed_repo) = repo.as_any().downcast_ref::<DbRepository<PK, T>>() {
                return Ok(typed_repo.clone());
            }
        }

        Err(InteropError::invariant(format!(
            "No repository found with PK {} and model {} in RepositoryManager",
            std::any::type_name::<PK>(),
            std::any::type_name::<T>()
        )))
    }
}

pub trait ScriptingRepositoryManager: TypedRepositoryManager {
    /// Find by ID and create allocated reference in one operation
    /// This is a convenience method that combines typed repository lookup, async find_by_id call,
    /// and creation of allocated reflection reference
    fn find_by_id_allocated<PK, T>(&self, id: PK, world: &mut World) -> ScriptValue
    where
        T: EntityTrait + Clone + Send + Sync + 'static,
        T::Model: IntoActiveModel<T::ActiveModel> + Send + Sync + 'static + Reflect,
        T::ActiveModel: Send + Sync + 'static,
        PK: Into<<<T as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Sync
            + 'static,
    {
        self.typed::<PK, T>()
            .map(|repo| {
                crate::utils::block_on(|| async move { repo.find_by_id(id).await })
                    .map(|model| world.new_allocated(model))
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }
}

pub trait UpdatableModel {
    type Column: ColumnTrait;

    /// Returns the columns that can be updated for this model
    fn update_columns() -> &'static [Self::Column];
}

pub trait PrimaryKeyColumns {
    type Column: ColumnTrait;

    /// Returns the columns that can be updated for this model
    fn pk_columns() -> &'static [Self::Column];
}

/// Combine PrimaryKeyColumns and UpdatableModel to provide
/// common repository operations including a prebuilt OnConflict strategy.
pub trait RepositoryModel: PrimaryKeyColumns + UpdatableModel {
    fn on_conflict() -> OnConflict {
        OnConflict::columns(Self::pk_columns().to_vec())
            .update_columns(Self::update_columns().to_vec())
            .to_owned()
    }
}
