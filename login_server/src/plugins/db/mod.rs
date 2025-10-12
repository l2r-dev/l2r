use crate::plugins::config::Config;
use bevy::{log, prelude::*};
use l2r_core::db::{DbRepositoryPlugin, PostgresPlugin, RedisConfig, RedisPlugin};
use sea_orm::ConnectOptions;
use std::time::Duration;

pub mod migrations;

pub struct LoginDbPlugin;

impl Plugin for LoginDbPlugin {
    fn build(&self, app: &mut App) {
        let (database_url, redis_url) = {
            let config = app.world().resource::<Config>();
            (
                config.general().database_url.0.clone(),
                config.general().redis_url.0.clone(),
            )
        };

        app.add_plugins(RedisPlugin::new(RedisConfig {
            redis_url,
            timeout: Duration::from_secs(5),
        }));

        let mut connect_options = ConnectOptions::new(database_url);
        connect_options
            .min_connections(1)
            .max_connections(10)
            .connect_timeout(std::time::Duration::from_secs(5));

        app.add_plugins(PostgresPlugin {
            connection: None,
            config: Some(connect_options.into()),
        });

        log::debug!("Database and Redis plugins initialized");
        app.add_plugins(DbLoginRepositoryPlugin);
        log::debug!("Database repositories initialized");

        app.add_plugins(migrations::LoginServerMigrationPlugin);
    }
}

pub struct DbLoginRepositoryPlugin;
impl Plugin for DbLoginRepositoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DbRepositoryPlugin);
    }
}
