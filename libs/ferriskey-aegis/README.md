# FerrisKey Aegis

## Overview

`ferriskey-aegis` serves as the authorization and protection layer of the FerrisKey ecosystem. It provides the logic and structures necessary to enforce access control policies, ensuring that authenticated identities are authorized to perform specific actions on resources.

## Domain & Responsibilities

This library operates within the **Authorization** bounded context. Its primary responsibilities include:

- **Access Control**: Evaluating permissions based on roles, scopes, or attributes.
- **Policy Enforcement**: Verifying that requests meet security criteria before processing.
- **Resource Protection**: Defining boundaries for sensitive domain entities.

## Core Components

- **AccessPolicy**: Defines rules for accessing resources.
- **Authorizer**: The primary service interface for checking permissions.
- **Permission**: Granular access rights definitions.

## Usage

```rust
// Conceptual usage of the authorization layer
use ferriskey_aegis::Authorizer;

if Authorizer::can_access(&user, &resource, Action::Read) {
    println!("Access granted");
} else {
    println!("Access denied");
}
```

## Dependencies

- `ferriskey-domain`: Provides the user and resource definitions required for authorization checks.