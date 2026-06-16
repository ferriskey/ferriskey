# maskass

## Overview

`maskass` is a lightweight Rust utility library within the FerrisKey ecosystem designed to store sensitive values while ensuring they are always safely redacted in logs, telemetry, and serialized outputs.

## Domain & Responsibilities

This library operates as a **Security Utility**. Its primary responsibilities include:

- **Data Redaction**: Preventing accidental leakage of PII, tokens, passwords, and secrets.
- **Safe Serialization**: Intercepting `Serialize`, `Display`, and `Debug` implementations to output masked strings.

## Core Components

- `Masked<T>`: A generic wrapper that always serializes/logs as `"***"`.
- `MaskedWith<S>`: A string wrapper that applies a specific masking strategy.
- **MaskStrategy Implementations**: `FullMask`, `EmailMask`, `PartialMask`, `HashMask`.

## Technical Details

The library ensures that the real underlying value is only available programmatically when explicitly requested (or via `Deserialize`), but any automatic formatting (`format!("{token}")`) or serialization (`serde_json::to_string`) will intercept the call and apply the `Redaction::Strategy`. `Masked` defaults to a complete mask, whereas `MaskedWith` allows domain-aware masking (like keeping the domain in an email but hiding the username).

## Usage

```rust
use maskass::{Masked, MaskedWith, EmailMask, PartialMask};

let api_key = Masked::new("super-secret".to_string());
assert_eq!(format!("{}", api_key), "***");
assert_eq!(serde_json::to_string(&api_key).unwrap(), "\"***\"");

let email = MaskedWith::<EmailMask>::new("user@example.com");
assert_eq!(format!("{}", email), "******@example.com");

let token = MaskedWith::<PartialMask<2, 2>>::new("123456789");
assert_eq!(format!("{}", token), "12*****89");
```

## Dependencies

- `serde`: For interception of serialization routines.
