# <img src="./img/l2r_logo3.png" alt="L2R" width="40" align="center"/> L2R - MMORPG Server Emulator

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Lua](https://img.shields.io/badge/lua-%232C2D72.svg?style=flat&logo=lua&logoColor=white)](https://www.lua.org/)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=flat&logo=docker&logoColor=white)](https://www.docker.com/)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=flat&logo=telegram&logoColor=white)](https://t.me/l2r_dev)

An experimental MMO server emulator written in Rust, designed for modern infrastructure with safety, speed, and reliability at its core.

> ⚠️ **Note**: This project is in early development and not intended for production use.

## 🚀 Features

- **High Performance**: Built with Rust for maximum performance and minimal memory footprint
- **Memory Safety**: Zero-cost abstractions and memory safety without garbage collection
- **Docker Integration**: Seamless containerized setup using Docker Compose for streamlined development and modern deployment workflows.
- **Chronicle Support**: Initially targeting High Five chronicle, but has 'foundation' (some mechanics implemented behind cargo features) to develop different game chronicles in single codebase.

## 🎯 Why Rust?

One night before bed, I decided to learn Rust, and this project became my pet project. 😄

- **Performance**: Near C/C++ performance with zero-cost abstractions
- **Borrow Checker**: ❤️ Quite good memory control and special-way of understanding memory layouts.
- **Concurrency**: Handy support for async programming and multithreading
- **Modern Tooling**: Cargo package manager, built-in testing, and documentation
- **Cross-platform**: Easy deployment on Linux, Windows, and macOS

## ⚙️ Why Bevy?

Building an MMO server requires complex entity management, and Bevy's architecture fits perfectly for our MMO emulator needs:

- **🏗️ Entity Component System (ECS)**: Bevy's data-driven ECS architecture naturally models game entities (players, NPCs, items) with maximum performance and flexibility. Every character, monster, and item is an entity with modular components.

- **🚀 Performance at Scale**: Built for handling thousands of entities simultaneously - perfect for MMO environments with thousands of concurrent players, NPCs, and world objects.

- **🧩 Plugin System**: Each game system (combat, movement, inventory, chat) is a separate plugin, allowing for clean separation of concerns and easy feature development. While hot-reloading is not currently implemented, it is possible with known limitations—such as potential state mismatches and issues with function declarations, which must always remain static in dynamic libs.

- **📊 Query System**: Powerful entity querying with compile-time safety enables complex game logic like "find all players within these components, filter by only changed last frame" or "update all items in a specific zone entity filtered by MyComponent" with optimal performance.

- **⚡ Parallel Processing**: Bevy's system scheduler automatically parallelizes non-conflicting operations across CPU cores, maximizing throughput for server-side calculations like movement validation, combat resolution, and other systems processing.

- **🔄 Handy Async**: Support for async operations through `bevy_defer`, enabling database operations, some HTTP or I/O without blocking the main game loop.

- **🔗 Networking Support**: Integration with `bevy_slinet` provides client-server stuff for implementg TCP/UDP client-servers.

- **🧪 Complete Unit Testing**: Bevy's ECS architecture enables comprehensive unit tests that simulate exactly all game mechanics that happen during real gameplay - complete with entity interactions, combat systems, movement validation, and state changes - but without networking overhead and with database mocking. Tests can run the full game loop using `app.update()` while using mocked database connections, allowing for fast, reliable testing of complex MMO scenarios like player vs monster combat, skill usage, and world interactions.

- **🎮 [Developer GUI & 3D Management](docs/game-server.md)**: Bevy makes it quite easy to use crates implementing GUIs, and this project uses `bevy-inspector-egui` to provide a feature-toggled developer interface. This GUI lets you visualize the world through reflected components and edit entities in real time for debugging and investigation, supporting 3D transforms and custom entity hierarchies for MMO world management. By default, the server runs as a CLI application for minimal resource usage on headless servers (such as Linux-servers), with the GUI available only when explicitly enabled via cargo features.

---

## 🛠 Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (nightly)
- [Docker](https://www.docker.com/) and [Docker Compose](https://docs.docker.com/compose/)
- Git
- [High Five Client](docs/client-requirements.md)
- [GeoData Files](docs/geodata-setup.md) - download and extract geodata.

### Quick Start

1. **Clone the repository**

   ```bash
   git clone https://github.com/aggyomfg/l2r.git
   cd l2r
   ```

2. **Start the development environment**

   ```bash
   docker compose up -d
   ```

3. **Install runcc for easier start**

   ```bash
   cargo install runcc
   ```

4. **Run both servers (login and game)**

   ```bash
   cargo runcc -c .
   ```

### Development Environment

The development setup includes:

- **PostgreSQL**
- **Redis**
- **pgAdmin** for database management (accessible at <http://localhost:8082> with <l2r@l2r.com>:l2r)

### Configuration

Modify config as needed (also can be configured via ENVs with L2R_ prefix):

```bash
login_server/data/config.toml
game_server/data/config.toml
```

### 📚 **Feature Documentation**

| Component | Status | Description |
|-----------|--------|-------------|
| **[🏗️ Infrastructure](docs/infrastructure.md)** | ★ ⚡ 🔨 | Database, configuration, monitoring, security & network protocols |
| **[🔐 Login Server](docs/login-server.md)** | ⚡ 🔨 | Authentication, session management & server gateway |
| **[🎮 Game Server](docs/game-server.md)** | ★ ⚡ 🔨 | World simulation, combat, items & MMO mechanics |

</div>

**Legend**: ★ = Almost working | ⚡ = In Development | 🔨 = Dreams & Future Plans

### 🌟 **Quick Highlights**

- **🚀 ECS Architecture**: Built on Bevy's high-performance Entity Component System
- **🔍 Monitoring**: OpenMetrics integration for any game/infra metrics. Reconnectable DB pool.
- **🗺️ L2J Geodata**: Full pathfinding with A* algorithm implementation  
- **⚔️ Combat System**: Physical damage, critical hits, hit-miss and Lua skill framework
- **📊 Stats Calculation**: System with formulas for P.Atk, M.Atk, Speed, and other character stats
- **🎒 Item Management**: Inventory system with equipment stats
- **🤖 NPC AI**: Spawn system with basic monster behaviors
- **👥 Player Visibility**: Encounter system with distance-based updates

## 🤝 Contributing

Want to contribute? Here is how:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** and add tests if possible.
4. **Run tests**: `cargo test`
5. **Commit your changes**: `git commit -m 'Add amazing feature'`
6. **Push to the branch**: `git push`
7. **Open a Pull Request**

### Development Guidelines

- Follow Rust conventions and idioms
- Use `bevy_lint` to verify some bevy related-stuff.
- Add tests for new functionality
- Update documentation for public APIs, remeber rustdoc is great
- TODO: add ci/cd to check fmt and lint

## 📈 Performance

TODO Some benchmarks-tests.

## Contacts

Join Telegram channel and chat for updates, discussions, and community support: **[t.me/l2r_dev](https://t.me/l2r_dev)**

## 🙏 Acknowledgments & Shoutouts

### 🏛️ L2 Server Emulation

- **[L2J Team](https://l2jserver.com/)** - The original pioneers of open-source server emulation
- **[L2jMobius](https://l2jmobius.org/)** ([GitLab](https://gitlab.com/MobiusDevelopment/L2J_Mobius)) - The most mature and actively maintained Java emulator with extensive chronicles support

### 🦀 Essential Rust & Bevy Ecosystem Crates

- **[bevy_slinet](https://crates.io/crates/bevy_slinet)** ([GitHub](https://github.com/edouardparis/bevy_slinet)) - Simple networking plugin for Bevy with TCP/UDP support, essential for client-server communication
- **[bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)** ([GitHub](https://github.com/jakobhellermann/bevy-inspector-egui)) - Real-time component inspector and debug GUI for Bevy ECS, invaluable for MMO development
- **[bevy_mod_scripting](https://crates.io/crates/bevy_mod_scripting)** ([GitHub](https://github.com/makspll/bevy_mod_scripting)) - Scripting engine for Bevy, enabling dynamic game logic and content creation without recompilation and makes development more flexible
- **[egui_dock](https://crates.io/crates/egui_dock)** ([GitHub](https://github.com/Adanos020/egui_dock)) - Docking support for egui, enabling professional IDE-like interfaces with draggable tabs and windows
- **[avian3d](https://crates.io/crates/avian3d)** ([GitHub](https://github.com/Jondolf/avian)) - ECS-driven 3D physics engine for Bevy
- **[bevy_defer](https://crates.io/crates/bevy_defer)** ([GitHub](https://github.com/mintlu8/bevy_defer)) - Async runtime for Bevy with World access, enabling async functions like database operations without blocking the game loop
- **[bevy_webgate](https://crates.io/crates/bevy_webgate)** - Web integration for Bevy applications, useful for admin panels and metrics endpoints
- **[pathfinding](https://crates.io/crates/pathfinding)** ([GitHub](https://github.com/evenfurther/pathfinding)) - A* pathfinding implementation

### 🛠️ Core Infrastructure & Development Tools

- **[Sea-ORM](https://crates.io/crates/sea-orm)** ([GitHub](https://github.com/SeaQL/sea-orm)) - Async & dynamic ORM for Rust, providing type-safe database operations
- **[Redis](https://crates.io/crates/redis)** ([GitHub](https://github.com/redis-rs/redis-rs)) - High-level Redis client for session management and caching
- **[Tera](https://crates.io/crates/tera)** ([GitHub](https://github.com/Keats/tera)) - Template engine for dynamic HTML generation

### 🌟 Special Recognition

- **The Rust Community** for creating an ecosystem that makes systems programming both safe and performant
- **Bevy Contributors** for building a game engine that perfectly matches MMO server architecture needs  
- **NCSoft** for creating the legendary world of Lineage 2 that continues to inspire developers worldwide

> ⚠️ **Note**: This is an educational project. Please ensure you comply with all applicable laws and terms of service when using this software.
---
Made with ❤️ and 🦀
