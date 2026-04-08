# FerrisKey Security

## Overview

`ferriskey-security` provides the cryptographic primitives and security utilities for the FerrisKey ecosystem. It encapsulates sensitive operations such as password hashing, token generation (JWT), and key management to ensure secure implementation of identity protocols.

## Domain & Responsibilities

This library operates within the **Cryptography & Security** bounded context. Its primary responsibilities include:

- **Password Hashing**: Securely hashing and verifying user passwords using Argon2.
- **Token Management**: Generating, signing, and verifying JSON Web Tokens (JWT).
- **Key Management**: Handling RSA key pairs for signing and encryption.

## Core Components

- **PasswordService**: High-level API for hashing and verification.
- **TokenService**: Service for JWT lifecycle (issue, verify, refresh).
- **KeyProvider**: Interface for retrieving signing keys.

## Usage

```rust
use ferriskey_security::PasswordService;

// Hashing a password
let hash = PasswordService::hash("my_secure_password").await?;

// Verifying a password
let is_valid = PasswordService::verify("my_secure_password", &hash).await?;
```

## Dependencies

- `ferriskey-domain`: Domain types.
- `argon2`, `jsonwebtoken`, `rsa`: Cryptographic implementations.