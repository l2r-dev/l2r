use axum::response::IntoResponse;
use bevy::{
    app::App,
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    log,
    prelude::*,
};
use bevy_defer::{AsyncAccess, AsyncWorld};
use bevy_webgate::{WebServerAppExt, WebServerConfig};
use memory_stats::memory_stats;
use prometheus::{Encoder, IntCounter, IntGauge, Registry};
use std::{
    self,
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};
use strum::{AsRefStr, Display, EnumIter, EnumString};

#[derive(Clone, Debug)]
pub enum MetricsMode {
    /// Full mode with web server for metrics endpoint
    Full { port: u16 },
    /// Mock mode for tests - no web server, just metrics collection
    Mock,
}

pub struct MetricsPlugin {
    prefix: String,
    mode: MetricsMode,
}

impl MetricsPlugin {
    pub fn new(prefix: String, port: u16) -> Self {
        MetricsPlugin {
            prefix,
            mode: MetricsMode::Full { port },
        }
    }

    pub fn mock(prefix: String) -> Self {
        MetricsPlugin {
            prefix,
            mode: MetricsMode::Mock,
        }
    }
}

impl Plugin for MetricsPlugin {
    fn build(&self, app: &mut App) {
        // Only configure web server in Full mode
        if let MetricsMode::Full { port } = self.mode {
            app.insert_resource(WebServerConfig {
                ip: IpAddr::V4(Ipv4Addr::from_str("0.0.0.0").unwrap()),
                port,
            });

            app.port_route(port, "/metrics", axum::routing::get(metrics_handler));
        }

        let mut metrics = Metrics::new(self.prefix.clone());
        // Register core infrastructure metrics
        if let Err(e) = metrics.register_core_metrics() {
            log::error!("Failed to register core metrics: {}", e);
        }
        app.insert_resource(metrics);

        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin,
            SystemInformationDiagnosticsPlugin,
        ));

        app.add_systems(Update, (update_metrics, sync_bevy_diagnostics));
    }
}

async fn metrics_handler() -> impl IntoResponse {
    AsyncWorld
        .resource::<Metrics>()
        .get(|metrics| metrics.gather())
        .unwrap()
}

#[derive(Clone, Resource)]
pub struct Metrics {
    registry: Registry,
    prefix: String,
    counters: HashMap<String, IntCounter>,
    gauges: HashMap<String, IntGauge>,
}

/// Core infrastructure counter metrics that are always available
#[derive(AsRefStr, Clone, Copy, Display, EnumIter, EnumString, Eq, Hash, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum CoreCounterName {
    /// Server uptime in seconds
    Uptime,
}

/// Core infrastructure gauge metrics that are always available
#[derive(AsRefStr, Clone, Copy, Display, EnumIter, EnumString, Eq, Hash, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum CoreGaugeName {
    // Memory metrics
    VirtualMemoryUsedBytes,
    PhysicalMemoryUsedBytes,

    // Bevy engine metrics
    FramesPerSecond,
    FrameTimeMs,
    EntityCount,
    SystemCpuUsage,
    SystemMemoryUsage,
}

/// Extension trait for Bevy App to register metrics easily
pub trait MetricsAppExt {
    /// Register a counter metric with the metrics system
    fn register_counter(&mut self, name: impl std::fmt::Display, help: &str) -> &mut Self;

    /// Register a gauge metric with the metrics system
    fn register_gauge(&mut self, name: impl std::fmt::Display, help: &str) -> &mut Self;
}

impl MetricsAppExt for App {
    fn register_counter(&mut self, name: impl std::fmt::Display, help: &str) -> &mut Self {
        let name_str = name.to_string();

        self.world_mut()
            .resource_mut::<Metrics>()
            .register_counter(&name_str, help)
            .unwrap_or_else(|e| {
                log::warn!("Failed to register counter '{}': {}", name_str, e);
            });
        self
    }

    fn register_gauge(&mut self, name: impl std::fmt::Display, help: &str) -> &mut Self {
        let name_str = name.to_string();

        self.world_mut()
            .resource_mut::<Metrics>()
            .register_gauge(&name_str, help)
            .unwrap_or_else(|e| {
                log::warn!("Failed to register gauge '{}': {}", name_str, e);
            });
        self
    }
}
impl Default for Metrics {
    fn default() -> Self {
        Self::new("l2r_".to_string())
    }
}

impl Metrics {
    pub fn new(prefix: String) -> Self {
        Metrics {
            registry: Registry::new(),
            prefix,
            counters: HashMap::new(),
            gauges: HashMap::new(),
        }
    }

    /// Register a new counter metric. The name will be prefixed automatically.
    /// Returns Ok(()) if registration succeeds, Err if the metric already exists.
    pub fn register_counter(&mut self, name: impl std::fmt::Display, help: &str) -> Result<()> {
        let key = name.to_string();
        let full_name = format!("{}{}", self.prefix, key);

        if self.counters.contains_key(&key) {
            return Err(BevyError::from(format!(
                "Counter '{}' already registered",
                key
            )));
        }

        let counter = IntCounter::new(full_name, help)
            .map_err(|e| BevyError::from(format!("Failed to create counter: {}", e)))?;

        self.registry
            .register(Box::new(counter.clone()))
            .map_err(|e| BevyError::from(format!("Failed to register counter: {}", e)))?;

        self.counters.insert(key, counter);
        Ok(())
    }

    /// Register a new gauge metric. The name will be prefixed automatically.
    /// Returns Ok(()) if registration succeeds, Err if the metric already exists.
    pub fn register_gauge(&mut self, name: impl std::fmt::Display, help: &str) -> Result<()> {
        let key = name.to_string();
        let full_name = format!("{}{}", self.prefix, key);

        if self.gauges.contains_key(&key) {
            return Err(BevyError::from(format!(
                "Gauge '{}' already registered",
                key
            )));
        }

        let gauge = IntGauge::new(full_name, help)
            .map_err(|e| BevyError::from(format!("Failed to create gauge: {}", e)))?;

        self.registry
            .register(Box::new(gauge.clone()))
            .map_err(|e| BevyError::from(format!("Failed to register gauge: {}", e)))?;

        self.gauges.insert(key, gauge);
        Ok(())
    }

    /// Get a counter by name. Accepts any type that implements Display.
    pub fn counter(&self, name: impl std::fmt::Display) -> Result<&IntCounter> {
        let key = name.to_string();
        self.counters
            .get(&key)
            .ok_or_else(|| BevyError::from(format!("Counter '{}' not found", key)))
    }

    /// Get a gauge by name. Accepts any type that implements Display.
    pub fn gauge(&self, name: impl std::fmt::Display) -> Result<&IntGauge> {
        let key = name.to_string();
        self.gauges
            .get(&key)
            .ok_or_else(|| BevyError::from(format!("Gauge '{}' not found", key)))
    }

    pub fn gather(&self) -> String {
        let mut buffer = vec![];
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();

        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            log::error!("Failed to encode metrics: {:?}", e);
            return String::from("# Failed to encode metrics");
        }

        String::from_utf8(buffer).unwrap_or_else(|e| {
            log::error!("Failed to convert metrics buffer to UTF-8: {:?}", e);
            String::from("# Failed to convert metrics to UTF-8")
        })
    }

    /// Helper to register all core infrastructure metrics
    pub fn register_core_metrics(&mut self) -> Result<()> {
        // Memory metrics
        self.register_gauge(
            CoreGaugeName::VirtualMemoryUsedBytes,
            "Virtual memory used in bytes",
        )?;
        self.register_gauge(
            CoreGaugeName::PhysicalMemoryUsedBytes,
            "Physical memory used in bytes",
        )?;

        // Bevy engine metrics
        self.register_gauge(CoreGaugeName::FramesPerSecond, "Frames per second")?;
        self.register_gauge(CoreGaugeName::FrameTimeMs, "Frame time in milliseconds")?;
        self.register_gauge(CoreGaugeName::EntityCount, "Total entity count")?;
        self.register_gauge(CoreGaugeName::SystemCpuUsage, "System CPU usage percentage")?;
        self.register_gauge(
            CoreGaugeName::SystemMemoryUsage,
            "System memory usage percentage",
        )?;

        // Server metrics
        self.register_counter(CoreCounterName::Uptime, "Server uptime in seconds")?;

        Ok(())
    }
}

pub fn update_metrics(
    metrics: Res<Metrics>,
    mut last_time: Local<f32>,
    time: Res<Time>,
) -> Result<()> {
    if (time.elapsed_secs() - *last_time) >= 1.0 {
        *last_time = time.elapsed_secs();

        metrics.counter(CoreCounterName::Uptime)?.inc();

        if let Some(usage) = memory_stats() {
            metrics
                .gauge(CoreGaugeName::PhysicalMemoryUsedBytes)?
                .set(usage.physical_mem as i64);
            metrics
                .gauge(CoreGaugeName::VirtualMemoryUsedBytes)?
                .set(usage.virtual_mem as i64);
        }
    }
    Ok(())
}

/// System to sync Bevy diagnostics with Prometheus metrics
pub fn sync_bevy_diagnostics(
    diagnostics: Res<DiagnosticsStore>,
    metrics: Res<Metrics>,
) -> Result<()> {
    // Frame time diagnostics
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diag| diag.smoothed())
    {
        metrics
            .gauge(CoreGaugeName::FramesPerSecond)?
            .set(fps as i64);
    }

    if let Some(frame_time) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diag| diag.smoothed())
    {
        metrics
            .gauge(CoreGaugeName::FrameTimeMs)?
            .set(frame_time as i64);
    }

    // Entity count diagnostics
    if let Some(entity_count) = diagnostics
        .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        .and_then(|diag| diag.smoothed())
    {
        metrics
            .gauge(CoreGaugeName::EntityCount)?
            .set(entity_count as i64);
    }

    // System information diagnostics
    if let Some(cpu_usage) = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE)
        .and_then(|diag| diag.smoothed())
    {
        metrics
            .gauge(CoreGaugeName::SystemCpuUsage)?
            .set(cpu_usage as i64);
    }

    if let Some(mem_usage) = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE)
        .and_then(|diag| diag.smoothed())
    {
        metrics
            .gauge(CoreGaugeName::SystemMemoryUsage)?
            .set(mem_usage as i64);
    }
    Ok(())
}
