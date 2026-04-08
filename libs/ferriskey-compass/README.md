# FerrisKey Compass

## Overview

`ferriskey-compass` provides the navigation and discovery mechanisms for the FerrisKey ecosystem. It is responsible for resolving realms, managing configuration contexts, and ensuring that requests are routed to the correct tenant isolation.

## Domain & Responsibilities

This library operates within the **Tenancy & Configuration** bounded context. Its primary responsibilities include:

- **Realm Resolution**: identifying the target realm from a request (domain, path, or header).
- **Configuration Management**: Loading and serving dynamic realm settings.
- **Tenant Context**: Establishing the isolation boundaries for a given operation.

## Core Components

- **RealmResolver**: Service to determine the active realm.
- **ConfigProvider**: Interface for fetching realm-specific settings.
- **CompassContext**: The resolved context containing tenant information.

## Usage

```rust
use ferriskey_compass::RealmResolver;

// Resolving a realm from a domain
let realm = RealmResolver::resolve("auth.example.com").await?;
println!("Current Realm: {}", realm.name);
```

## Dependencies

- `ferriskey-domain`: Defines the `Realm` entity.