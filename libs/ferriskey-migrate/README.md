# FerrisKey Migrate

## Overview

`ferriskey-migrate` provides the infrastructure and tooling for managing database schema changes and data migrations within the FerrisKey ecosystem. It ensures that the persistent state of the application evolves safely and consistently across versions.

## Domain & Responsibilities

This library operates within the **Infrastructure & Persistence** bounded context. Its primary responsibilities include:

- **Schema Evolution**: Managing versioned database changes.
- **Migration Tracking**: Recording which migrations have been applied.
- **Data Migration**: Facilitating complex data transformations during upgrades.

## Core Components

- **Migration**: Trait or struct representing a single unit of change.
- **Migrator**: Service responsible for planning and executing migrations.
- **MigrationError**: Standardized errors for migration failures.

## Usage

```rust
use ferriskey_migrate::Migrator;

// Conceptual usage
let result = Migrator::up().await;
match result {
    Ok(_) => println!("Migrations applied successfully"),
    Err(e) => eprintln!("Migration failed: {}", e),
}
```

## Dependencies

- `chrono`: For timestamping migrations.
- `thiserror`: For error handling.