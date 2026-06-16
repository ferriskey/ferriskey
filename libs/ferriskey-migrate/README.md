# FerrisKey Migrate

## Overview

`ferriskey-migrate` is the database migration and schema management library for the FerrisKey ecosystem. It handles the initialization and evolution of the underlying data store, ensuring that the database schema remains synchronized with the domain models.

## Domain & Responsibilities

This library operates within the **Infrastructure & Persistence** bounded context. Its primary responsibilities include:

- **Schema Management**: Applying, tracking, and rolling back database migrations.
- **Bootstrapping**: Initializing the database for fresh FerrisKey installations.
- **State Verification**: Ensuring the running application is compatible with the current database schema.

## Core Components

- **Migrator**: The engine responsible for executing SQL or programmatic migration steps.
- **Migrations**: The sequential list of schema changes.

## Technical Details

The library wraps the underlying ORM or database driver's migration capabilities (e.g., SeaORM or SQLx), providing a clean CLI or programmatic interface for CI/CD pipelines and deployment scripts. It ensures migrations run transactionally and safely across distributed instances.

## Dependencies

- Database driver / ORM (e.g., `sea-orm-migration` or `sqlx`).
