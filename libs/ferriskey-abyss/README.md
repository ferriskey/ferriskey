# FerrisKey Abyss

## Overview

`ferriskey-abyss` is the domain library responsible for managing authentication flows, session handling, and deep authorization contexts within FerrisKey. It serves as the bridge between raw identity data and actionable security contexts.

## Domain & Responsibilities

This library operates within the **Authentication & Authorization** bounded context. Its primary responsibilities include:

- **Session Management**: Tracking active user sessions across devices.
- **Authentication Context**: Building and validating the security context for incoming requests.
- **Identity Resolution**: Linking ephemeral credentials to persistent identities.

## Core Components

- **Session**: Represents an active authenticated session.
- **AuthContext**: The aggregate root for security decisions during a request lifecycle.
- **AbyssService**: The primary entry point for managing session lifecycles.

## Usage

```rust
use ferriskey_abyss::SessionService;

// Example usage (conceptual)
async fn login_flow(user_id: Uuid) {
    let session = SessionService::create_session(user_id).await?;
    println!("Session created: {}", session.id);
}
```

## Dependencies

- `ferriskey-domain`: Core domain types and traits.
- `maskass`: Utility for masking sensitive data.