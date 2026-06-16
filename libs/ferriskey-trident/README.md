# FerrisKey Trident

## Overview

`ferriskey-trident` is the **Multi-Factor Authentication (MFA)** library for the FerrisKey ecosystem. It provides an advanced suite of second-factor mechanisms to strengthen user authentication security beyond simple passwords.

As described in the FerrisKey modules documentation, Trident handles TOTP, WebAuthn passkeys, magic links, and recovery codes.

## Domain & Responsibilities

This library operates within the **Authentication Assurance** bounded context. Its primary responsibilities include:

- **TOTP (Time-based One-Time Password)**: Generating and verifying 6-digit codes compatible with authenticator apps (RFC 6238).
- **WebAuthn**: Managing passkeys and hardware security keys for passwordless or step-up authentication.
- **Recovery Codes**: Managing secure backup codes for account recovery in case primary MFA methods are lost.
- **Magic Links**: Providing email-based login or verification mechanisms.

## Core Components

- **TotpService**: Core logic for generating secrets and validating time-based codes.
- **RecoveryCodeGenerator**: Utility for creating cryptographically secure, single-use recovery codes.
- **MfaPolicy**: Rules determining when MFA challenges are required.

## Technical Details

`ferriskey-trident` relies on standard cryptographic crates like `base32` for secrets encoding and `chrono` for time window validation. It interacts closely with `ferriskey-domain` to attach MFA configurations and policies to user entities.

## Dependencies

- `ferriskey-domain`: Core domain entities.
- `base32`: Standard encoding for TOTP secrets.
- `chrono`: Handling time windows for TOTP validation.
