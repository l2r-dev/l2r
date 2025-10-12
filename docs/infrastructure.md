# 🏗️ Core Infrastructure & Foundation

*Building the backbone of a modern MMO server emulator*

---

## ★ **Database Architecture**
- **Generic Repository Pattern** - Type-safe CRUD operations with `DbRepository<PK, Entity>` abstraction over SeaORM
- **Connection Pooling** - PostgreSQL with automatic connection management and health monitoring with possibility of pausing some game systems, but keep players online if db fails.
- **[Migration System](database-migrations.md)** - Version-controlled schema evolution with rollback capabilities
- **Multi-database Support** - PostgreSQL for persistence, Redis for session/caching

## ★ **Flexible Configuration**
- **Hierarchical TOML Configs** - Environment-specific overrides with validation and hot-reloading
- **Environment Variables** - Full `L2R_*` prefix support for containerized deployments
- **Feature Flags** - Compile-time chronicle selection and runtime feature toggling
- **Structured Validation** - Type-safe config parsing with detailed error reporting

## ★ **Asset Management Pipeline**
- **Hot-Reloading** - Live asset updates during development without server restart
- **Multi-format Support** - JSON, RON, TOML assets with automatic deserialization
- **Chronicle Assets** - Version-specific game data loading based on chronicle selection
- **Validation** - Compile-time asset schema validation and runtime integrity checks

## ★ **Security & Cryptography Foundation**
- **RSA Key Exchange** - 1024-bit RSA to implement original game 'ecnryption'
- **Blowfish Encryption** - Implemented orinal game block-encryption.
- **Password Security** - Argon2 hashing for auto-created accounts.

## ⚡ **Monitoring & Observability**
- **Metrics Collection** - Expose OpenMetrics performance counters, connection stats, and custom game metrics  
- **Health Checks** - Database connectivity, memory usage, and service health endpoints
- **Real-time Diagnostics** - Bevy's diagnostic system for ECS performance monitoring

## 🔨 **Advanced Infrastructure (Dreams)**
- **WebSocket API** - Real-time admin dashboard with live server statistics
- **Performance Profiling** - Automated benchmarking suite with historical tracking
- **Multi-Chronicle Engine** - Chronicle switching with shared infrastructure

---

*Legend: ★ = Almost working | ⚡ = In Development | 🔨 = Dreams*

[← Back to README](../README.md)