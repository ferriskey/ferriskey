In `@core/src/domain/authentication/services.rs` around lines 1273 - 1279, Replace
the direct equality check on the stored client secret with a constant-time
comparison to prevent timing attacks: instead of comparing
client.secret.as_deref() to Some(&input.client_secret) using !=, use a
constant-time equality check (e.g., subtle::ConstantTimeEq or an HMAC-based
comparison) to compare client.secret and input.client_secret within the
authentication flow (the block that returns CoreError::InvalidClientSecret).
Also update the analogous comparisons in the client_credential and password
handlers to the same constant-time method to keep behavior consistent.
