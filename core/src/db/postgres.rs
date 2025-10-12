use bevy::prelude::*;
use bevy_defer::{AccessError, AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use derive_more::{From, Into};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use std::{env, sync::Arc};

pub struct PostgresPlugin {
    pub connection: Option<DbConnection>,
    pub config: Option<DbConfig>,
}

impl Plugin for PostgresPlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone().unwrap_or_default();
        let connection = self.connection.clone().unwrap_or_default();

        app.insert_resource(config).insert_resource(connection);
        app.init_resource::<super::RepositoryManager>();

        app.add_systems(Startup, initial_connect)
            .add_systems(Update, spawn_health_check_tasks);
    }
}

#[derive(Clone, Debug, Deref, From, Into, Resource)]
pub struct DbConfig(ConnectOptions);

impl Default for DbConfig {
    fn default() -> Self {
        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://l2r:l2r@localhost/l2r".to_string());
        let mut connect_options = ConnectOptions::new(url.clone());
        connect_options
            .min_connections(1)
            .max_connections(10)
            .connect_timeout(std::time::Duration::from_secs(5));
        connect_options.into()
    }
}

#[derive(Clone, Resource)]
pub struct DbConnection(Arc<DatabaseConnection>, bool);
impl Default for DbConnection {
    fn default() -> Self {
        DbConnection(Arc::new(DatabaseConnection::Disconnected), false)
    }
}

impl DbConnection {
    pub fn new(connection: DatabaseConnection, connecting_now: bool) -> Self {
        DbConnection(Arc::new(connection), connecting_now)
    }

    pub fn connection(&self) -> Arc<DatabaseConnection> {
        self.0.clone()
    }

    pub fn disconnected(&self) -> bool {
        matches!(*self.0, DatabaseConnection::Disconnected)
    }

    pub fn is_mock(&self) -> bool {
        matches!(*self.0, DatabaseConnection::MockDatabaseConnection(_))
    }

    pub fn connecting_now(&self) -> bool {
        self.1
    }

    pub fn set_connecting_now(&mut self, connecting_now: bool) {
        self.1 = connecting_now;
    }
}

fn initial_connect(mut commands: Commands) {
    commands.spawn_task(connect_to_db);
}

fn spawn_health_check_tasks(
    mut commands: Commands,
    time: Res<Time>,
    mut last_check_time: Local<f32>,
) {
    let elapsed = time.elapsed_secs() - *last_check_time;
    if elapsed < 0.5 {
        return;
    }
    *last_check_time = time.elapsed_secs();
    commands.spawn_task(run_health_check);
}

async fn run_health_check() -> Result<(), AccessError> {
    let (connection, connecting_now) = AsyncWorld
        .resource::<DbConnection>()
        .get(|res| (res.0.clone(), res.1))?;

    if connecting_now {
        return Ok(());
    }

    if matches!(*connection, DatabaseConnection::MockDatabaseConnection(_)) {
        return Ok(());
    }

    let result = connection
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT 1".to_owned(),
        ))
        .await;

    if result.is_err() {
        let url = AsyncWorld
            .resource::<DbConfig>()
            .get(|res| res.get_url().to_string())?;
        log::error!("Error checking database ({url}) connection: {result:?}");
        AsyncWorld.resource::<DbConnection>().get_mut(|res| {
            *res = DbConnection(DatabaseConnection::Disconnected.into(), false);
        })?;
        connect_to_db().await?;
    }

    Ok(())
}

async fn connect_to_db() -> Result<(), AccessError> {
    let connection = AsyncWorld
        .resource::<DbConnection>()
        .get(|res| res.0.clone());

    if let Ok(conn) = connection
        && matches!(*conn, DatabaseConnection::Disconnected)
    {
        let config = AsyncWorld.resource::<DbConfig>().get(|res| res.clone())?;
        let url = config.get_url().to_string();

        AsyncWorld.resource::<DbConnection>().get_mut(|res| {
            res.set_connecting_now(true);
        })?;

        let new_connection = Database::connect(config).await;
        if let Ok(new_conn) = new_connection {
            log::info!("Connected to database: {url}");
            AsyncWorld.resource::<DbConnection>().get_mut(|res| {
                *res = DbConnection::new(new_conn, false);
            })?;
        } else {
            AsyncWorld.resource::<DbConnection>().get_mut(|res| {
                res.set_connecting_now(false);
            })?;
        }
    }
    Ok(())
}
