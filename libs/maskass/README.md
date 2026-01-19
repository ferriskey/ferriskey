# maskass

Small Rust crate to store sensitive values while ensuring they are always redacted in logs and serialized output.

## Features
- `Masked<T>`: generic wrapper that always serializes/logs as `"***"`.
- `MaskedWith<S>`: string wrapper that masks using a strategy (email, partial, hash, etc.).
- `MaskStrategy` implementations: `FullMask`, `EmailMask`, `PartialMask`, `HashMask`.

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

## Notes
- `Deserialize` stores the real value; `Serialize`, `Display`, and `Debug` never leak it.
- `MaskedWith` uses `Redaction::Strategy` by default; `Masked` always uses full redaction.
