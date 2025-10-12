# Database Migrations

This document explains how to manage database migrations for the L2R using SeaORM's migration system.
L2R uses **embedded migrations** that run automatically on startup, but also provides a **CLI interface** for manual migration management during development.

## Migration Architecture

- **Embedded Migrations**: Migrations run automatically when the server starts (see `*_server/src/plugins/db/migrations/mod.rs`)
- **CLI Interface**: Separate binary for manual migration management

## Using the Migration CLI

### Prerequisites

Set the `DATABASE_URL` environment variable:

For PowerShell:
```powershell
$env:DATABASE_URL="postgres://l2r:l2r@localhost:5432/l2r"
```

For bash/zsh:
```bash
export DATABASE_URL="postgres://l2r:l2r@localhost:5432/l2r"
```

### Available Commands

#### Check Migration Status
See which migrations have been applied:
```bash
cargo run --bin game_migrations -- status
```

#### Apply Pending Migrations
Apply all pending migrations:
```bash
cargo run --bin login_migrations -- up
```

Apply a specific number of migrations:
```bash
cargo run --bin game_migrations -- up -n 2
```

#### Rollback Migrations
Rollback the last applied migration:
```bash
cargo run --bin login_migrations -- down
```

Rollback multiple migrations:
```bash
cargo run --bin game_migrations -- down -n 2
```

#### Fresh Migration (Development)
Drop all tables and reapply all migrations (⚠️ **DESTRUCTIVE**):
```bash
cargo run --bin game_migrations -- fresh
```

#### Refresh Migrations
Rollback all migrations and reapply them:
```bash
cargo run --bin login_migrations -- refresh
```

#### Reset Database
Rollback all applied migrations:
```bash
cargo run --bin game_migrations -- reset
```

#### Generate New Migration
Create a new migration file:
```bash
cargo run --bin game_migrations -- generate create_new_table
```

## Creating New Migrations

1. **Generate the migration file**:
   ```bash
   cargo run --bin game_migrations -- generate add_player_stats
   ```

2. **Implement the migration** in the generated file:
   ```rust
   use sea_orm_migration::prelude::*;

   pub struct Migration;

   #[async_trait::async_trait]
   impl MigrationTrait for Migration {
       async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
           manager
               .create_table(
                   Table::create()
                       .table(PlayerStats::Table)
                       .col(ColumnDef::new(PlayerStats::Id).integer().not_null().primary_key())
                       .col(ColumnDef::new(PlayerStats::PlayerId).integer().not_null())
                       .to_owned(),
               )
               .await
       }

       async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
           manager
               .drop_table(Table::drop().table(PlayerStats::Table).to_owned())
               .await
       }
   }

   #[derive(Iden)]
   enum PlayerStats {
       Table,
       Id,
       PlayerId,
   }
   ```

3. **Register the migration** in `mod.rs`:
   ```rust
   mod add_player_stats;
   use add_player_stats::*;

   // Add to ServerMigrator::migrations()
   vec![
       Box::new(CharactersMigration),
       Box::new(ItemsMigration),
       Box::new(CharacterShortcutsMigration),
       Box::new(CharacterSkillsMigration),
       Box::new(AddPlayerStats),  // New migration
   ]
   ```

## Integration with Application

The application automatically runs migrations on startup via the `MigrationPlugin`. The migration system:

1. Spawns an async task during the `LoadingSystems::Migration` state
2. Connects to the database using the configured `database_url`
3. Runs `ServerMigrator::up()` to apply pending migrations
4. Transitions to `LoadingSystems::RepositoryInit` when complete


## Best Practices

1. **Always test migrations in development first** before applying to production
2. **Include both `up` and `down` implementations** for rollback capability
3. **Keep migrations small and focused** - don't make complex migrations.
4. **Never modify existing migrations** that have been applied to production
5. **Use `status` command** to verify migration state before making changes
6. **Version control** all migration files

## Files Structure

```
game_server/
├── src/
│   ├── bin/
│   │   └── migrations.rs          # CLI entry point (--bin game_migrations)
│   └── plugins/
│       └── db/
│           └── migrations/
│               ├── mod.rs          # InitialMigrations migrator
│               ├── characters_init.rs
│               ├── items_init.rs
│               ├── character_shortcuts_init.rs
│               └── character_skills_init.rs
└── Cargo.toml                      # Includes [[bin]] target

login_server/
├── src/
│   ├── bin/
│   │   └── migrations.rs          # CLI entry point (--bin login_migrations)
│   └── plugins/
│       └── db/
│           └── migrations/
│               ├── mod.rs          # LoginServerMigrator
│               └── accounts_init.rs
└── Cargo.toml                      # Includes [[bin]] target
```

## References

- [SeaORM Migration Documentation](https://www.sea-ql.org/SeaORM/docs/migration/running-migration/)
- [SeaORM CLI Guide](https://www.sea-ql.org/sea-orm-tutorial/ch01-02-migration-cli.html)

[← Back to README](../README.md)
