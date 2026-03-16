# FerrisKey Domain

## Overview

`ferriskey-domain` is the foundational library of the FerrisKey ecosystem. It defines the ubiquitous language, core entities, value objects, and repository interfaces used across all other modules.

## Domain & Responsibilities

This library operates within the **Core Domain** bounded context. Its primary responsibilities include:

- **Entity Definitions**: Representing business objects (User, Realm, Client).
- **Value Objects**: Immutable types (Email, PasswordHash, UserId).
- **Repository Contracts**: Trait definitions for data persistence.
- **Domain Errors**: Standardized error types for business logic failures.

## Core Components

- **User**: The aggregate root for identity management.
- **Realm**: The container for multi-tenancy configuration.
- **Client**: Represents applications or services consuming the IAM.
- **Repository Traits**: `UserRepository`, `RealmRepository`, etc.

## Usage

```rust
use ferriskey_domain::entities::User;
use ferriskey_domain::value_objects::Email;

let user = User::new(
    Email::parse("user@example.com")?,
    "username"
);
```

## Dependencies

- **None** (internal dependencies): This is the root dependency for most other FerrisKey libraries.
- `uuid`, `chrono`, `serde`: Standard external crates for data modeling.