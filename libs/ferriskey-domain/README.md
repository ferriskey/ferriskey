# FerrisKey Domain

## Overview

`ferriskey-domain` is the foundational library and **Core API** of the FerrisKey ecosystem. It serves as the heart of FerrisKey, responsible for user, role, permission, and multi-tenant realm management.

As defined in the official documentation, this is the stable core that provides the essential building blocks for all other modules.

## Domain & Responsibilities

This library operates within the **Core Domain** bounded context. Its primary responsibilities include:

- **User & Group Management**: The aggregate root for identity management and logical grouping.
- **Role-Based Access Control (RBAC)**: Assigning and evaluating roles for authorization.
- **Multi-tenancy with Realms**: Isolating users, clients, and configurations into distinct tenant boundaries.
- **Fine-grained Permissions**: Defining and managing granular access rights.
- **Repository Contracts**: Trait definitions for data persistence.

## Core Components

- **User / Realm / Client**: The primary domain entities.
- **Value Objects**: Immutable types like `Email`, `PasswordHash`, `UserId`.
- **Repository Traits**: Data access interfaces (`UserRepository`, `RealmRepository`, etc.) that allow dependency injection and hexagonal architecture implementation.

## Technical Details

`ferriskey-domain` strictly follows a hexagonal architecture model, defining pure business logic without any direct database or HTTP dependencies. It provides standardized error types and the ubiquitous language used throughout the FerrisKey project.

## Dependencies

- **None** (internal dependencies): This is the root dependency for most other FerrisKey libraries.
- `uuid`, `chrono`, `serde`: Standard external crates for data modeling.
