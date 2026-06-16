# FerrisKey Security

## Overview

`ferriskey-security` provides the critical cryptographic primitives and security utilities for the FerrisKey ecosystem. It acts as the backbone for sensitive operations, ensuring that identity protocols are implemented securely.

## Domain & Responsibilities

This library operates within the **Cryptography & Security** bounded context. Its primary responsibilities include:

- **Password Hashing**: Securely hashing and verifying user credentials.
- **Token Management**: Generating, signing, and verifying JSON Web Tokens (JWT) for authentication and API access.
- **Key Management**: Handling cryptographic key pairs (e.g., RSA) for signing assertions and tokens.

## Core Components

- **crypto**: High-level API for hashing and verification using modern algorithms like Argon2.
- **jwt**: Service for the JSON Web Token lifecycle (issue, verify, refresh).
- **SecurityError**: Strongly typed errors covering all cryptographic failure scenarios.

## Technical Details

The library relies on established, audited crates for its cryptographic operations, avoiding custom cryptography. It provides clean, domain-specific abstractions over these primitives (`PasswordService`, `TokenService`) so that other FerrisKey modules can implement security best practices without needing to manage raw algorithms or keys directly.

## Dependencies

- `ferriskey-domain`: Provides the core domain types referenced during token generation.
- `argon2`: For secure password hashing.
- `jsonwebtoken`, `rsa`: For JWT lifecycle and key management.
