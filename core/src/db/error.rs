use bevy_defer::AccessError;
use bevy_mod_scripting::bindings::InteropError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to create record: {0}")]
    CreateError(#[source] sea_orm::DbErr),

    #[error("Failed to update record: {0}")]
    UpdateError(#[source] sea_orm::DbErr),

    #[error("Failed to delete record: {0}")]
    DeleteError(#[source] sea_orm::DbErr),

    #[error("Failed to read record: {0}")]
    ReadError(#[source] sea_orm::DbErr),

    #[error("Failed to read record by field: {0}")]
    ReadByFieldError(#[source] sea_orm::DbErr),

    #[error("Transaction rollback failed: {0}")]
    RollbackError(#[source] sea_orm::DbErr),

    #[error("Pagination error: {0}")]
    PaginationError(#[source] sea_orm::DbErr),

    #[error("Database connection error: {0}")]
    ConnectionError(#[source] sea_orm::DbErr),

    #[error("Raw SQL execution error: {0}")]
    ExecutionError(#[source] sea_orm::DbErr),

    #[error("Error during parsing ScriptValue: {0}")]
    Scripting(String),

    #[error("Unknown database error: {0}")]
    Unknown(String),
}

impl From<DbError> for AccessError {
    fn from(db_error: DbError) -> Self {
        match db_error {
            DbError::CreateError(_) => AccessError::Custom("DbErr create operation failed"),
            DbError::UpdateError(_) => AccessError::Custom("DbErr update operation failed"),
            DbError::DeleteError(_) => AccessError::Custom("DbErr delete operation failed"),
            DbError::ReadError(_) => AccessError::Custom("DbErr read operation failed"),
            DbError::ReadByFieldError(_) => AccessError::Custom("DbErr field query failed"),
            DbError::RollbackError(_) => AccessError::Custom("DbErr transaction rollback failed"),
            DbError::PaginationError(_) => AccessError::Custom("DbErr pagination failed"),
            DbError::ConnectionError(_) => AccessError::Custom("DbErr connection failed"),
            DbError::ExecutionError(_) => AccessError::Custom("DbErr raw SQL execution failed"),
            DbError::Scripting(_) => AccessError::Custom("DbErr scripting value error"),
            DbError::Unknown(_) => AccessError::Custom("DbErr unknown error occurred"),
        }
    }
}

impl From<DbError> for InteropError {
    fn from(db_error: DbError) -> Self {
        match db_error {
            DbError::CreateError(err) => InteropError::external(Box::new(err)),
            DbError::UpdateError(err) => InteropError::external(Box::new(err)),
            DbError::DeleteError(err) => InteropError::external(Box::new(err)),
            DbError::ReadError(err) => InteropError::external(Box::new(err)),
            DbError::ReadByFieldError(err) => InteropError::external(Box::new(err)),
            DbError::RollbackError(err) => InteropError::external(Box::new(err)),
            DbError::PaginationError(err) => InteropError::external(Box::new(err)),
            DbError::ConnectionError(err) => InteropError::external(Box::new(err)),
            DbError::ExecutionError(err) => InteropError::external(Box::new(err)),
            DbError::Scripting(msg) => InteropError::invariant(msg),
            DbError::Unknown(msg) => InteropError::invariant(msg),
        }
    }
}
