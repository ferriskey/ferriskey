# FerrisKey Trident

## Overview

`ferriskey-trident` is the Multi-Factor Authentication (MFA) library for the FerrisKey ecosystem. It implements various second-factor mechanisms to strengthen user authentication security beyond simple passwords.

## Domain & Responsibilities

This library operates within the **Authentication Assurance** bounded context. Its primary responsibilities include:

- **TOTP (Time-based One-Time Password)**: Generating and verifying 6-digit codes compatible with authenticator apps (RFC 6238).
- **Recovery Codes**: Managing secure backup codes for account recovery.
- **MFA Challenges**: Orchestrating step-up authentication flows.

## Core Components

- **TotpService**: Core logic for generating secrets and validating time-based codes.
- **RecoveryCodeGenerator**: Utility for creating cryptographically secure, single-use recovery codes.
- **MfaPolicy**: Rules determining when MFA challenges are required.

## Usage

```rust
use ferriskey_trident::TotpService;

// Verify a user's input code against their stored secret
let is_valid = TotpService::verify_code(
    "JBSWY3DPEHPK3PXP", // User's secret (base32)
    "123456"            // Input code from user
)?;

if is_valid {
    println!("MFA verified successfully!");
}
```

## Dependencies

- `ferriskey-domain`: Core domain entities.
- `base32`: Standard encoding for TOTP secrets.
- `chrono`: Handling time windows for TOTP validation.