use crate::metrics::Metrics;
use bevy::{
    prelude::*,
    time::{Fixed, Time},
};
use redis::{Client, Connection, ConnectionLike, RedisResult};
use std::time::Duration;
use strum::{AsRefStr, Display};

pub struct RedisPlugin(RedisConfig);

impl RedisPlugin {
    pub fn new(config: RedisConfig) -> Self {
        RedisPlugin(config)
    }
}

impl Plugin for RedisPlugin {
    fn build(&self, app: &mut App) {
        let redis_client = RedisClient::new(&self.0).expect("Failed to connect to Redis");

        app.insert_resource(redis_client);

        app.add_systems(FixedUpdate, RedisClient::ping_and_reconnect);
    }
}

#[derive(Clone)]
pub struct RedisConfig {
    pub redis_url: String,
    pub timeout: Duration,
}

impl Default for RedisConfig {
    fn default() -> Self {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379/0".to_string());
        RedisConfig {
            redis_url,
            timeout: Duration::from_secs(3),
        }
    }
}

#[derive(AsRefStr, Display)]
#[strum(serialize_all = "snake_case")]
enum RedisMetric {
    RedisConnected,
}

#[derive(Resource)]
pub struct RedisClient {
    pub connection: Connection,
}

impl RedisClient {
    pub fn new(config: &RedisConfig) -> RedisResult<Self> {
        let client = Client::open(config.redis_url.clone())?;

        loop {
            match client.get_connection_with_timeout(config.timeout) {
                Ok(connection) => return Ok(RedisClient { connection }),
                Err(err) => {
                    log::error!(
                        "Failed to connect to Redis {} : {}. Retrying...",
                        config.redis_url,
                        err
                    );
                }
            }
        }
    }

    pub fn ping_and_reconnect(
        mut redis_client: ResMut<RedisClient>,
        mut metrics: ResMut<Metrics>,
        time: Res<Time<Fixed>>,
        mut last_time: Local<f32>,
        mut registered: Local<bool>,
    ) -> Result<()> {
        // Register metric on first run
        if !*registered {
            metrics.register_gauge(
                RedisMetric::RedisConnected,
                "Redis connection status (1=connected, 0=disconnected)",
            )?;
            *registered = true;
        }

        let time_spent = time.elapsed_secs() - *last_time;
        if (time_spent) >= 1.0 {
            *last_time = time.elapsed_secs();
            if redis_client.connection.check_connection() {
                metrics.gauge(RedisMetric::RedisConnected)?.set(1);
            } else {
                metrics.gauge(RedisMetric::RedisConnected)?.set(0);
                let client = RedisClient::new(&RedisConfig::default())?;
                redis_client.connection = client.connection;
                metrics.gauge(RedisMetric::RedisConnected)?.set(1);
            }
        }
        Ok(())
    }
}
