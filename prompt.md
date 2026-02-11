In `@core/src/domain/authentication/services.rs`:
- Around line 1356-1380: The code fetches an access token record via
access_token_repository.get_by_token_hash and then re-verifies the JWT signature
with verify_token, which can incorrectly mark DB-stored tokens inactive after
key rotation; remove the redundant verify_token call in the DB-backed path so
that once stored.revoked is false and stored.expires_at (if present) is in the
future, the function proceeds to deserialize stored.claims into JwtClaim and
return claims_to_introspection_response(realm.name), treating the database as
the source of truth for persisted tokens.

In `@api/src/application/http/authentication/handlers/introspect.rs`:
- Around line 16-28: The current try_parse_basic_client_credentials uses
strip_prefix("Basic ") which is case-sensitive; update it to perform a
case-insensitive check for the "Basic " auth-scheme before slicing off the
prefix (e.g., test the first 6 chars with eq_ignore_ascii_case or
to_ascii_lowercase and then take the remainder) so values like "basic " or
"BASIC " are accepted; keep the rest of the logic (base64 decode, utf8,
split_once, return Some((client_id, client_secret))) unchanged and only adjust
the prefix-detection and slicing in try_parse_basic_client_credentials.

In `@core/migrations/20260208120000_create-access-tokens.down.sql`:
- Line 1: The file containing the SQL statement "DROP TABLE IF EXISTS
access_tokens;" is missing a trailing newline; open the migration file
(create-access-tokens.down.sql) and ensure the final line ends with a newline
character so the file terminates with a newline and the end-of-file-fixer
pre-commit hook stops modifying it.

In `@core/migrations/20260208120000_create-access-tokens.up.sql`:
- Around line 1-15: The migration file defining the access_tokens table is
missing a trailing newline which breaks the pre-commit end-of-file-fixer hook;
open the migration that contains the CREATE TABLE access_tokens (and the
subsequent CREATE INDEX statements) and add a single newline character at the
end of the file so the file ends with a newline.

In `@core/src/domain/authentication/services.rs`:
- Around line 1337-1356: Extract the duplicated construction of
TokenIntrospectionResponse into a single helper function (e.g.,
build_token_introspection_response or map_claims_to_introspection) that accepts
the JwtClaim (claims) and the Realm (realm) and returns a
TokenIntrospectionResponse; move the field mappings (active: true, scope,
client_id from azp, username from preferred_username, sub from sub.to_string(),
token_type match on ClaimsTyp, exp, iat/nbf, aud joined, iss, jti.to_string(),
realm.name, etc.) into that helper and replace both the opaque-token path and
the JWT-fallback path returns with a call to this helper wrapped in Ok(...).
- Around line 1273-1279: Replace the direct equality check on the stored client
secret with a constant-time comparison to prevent timing attacks: instead of
comparing client.secret.as_deref() to Some(&input.client_secret) using !=, use a
constant-time equality check (e.g., subtle::ConstantTimeEq or an HMAC-based
comparison) to compare client.secret and input.client_secret within the
authentication flow (the block that returns CoreError::InvalidClientSecret).
Also update the analogous comparisons in the client_credential and password
handlers to the same constant-time method to keep behavior consistent.
- Around line 220-238: Replace the non-constant-time client secret check in the
authentication flow (the comparison inside the block using
client.secret.as_deref() != Some(&input.client_secret) in the function that
verifies client credentials) with a constant-time comparison (e.g., use
subtle::ConstantTimeEq or equivalent) by normalizing both secrets to byte slices
and performing a constant-time equality check, returning
CoreError::InvalidClientSecret on mismatch; ensure you still handle the Option
case safely. Also remove the duplicated TokenIntrospectionResponse construction
by extracting the shared mapping logic used in the opaque-token path (around the
block that builds TokenIntrospectionResponse at the opaque token branch) and the
JWT fallback path into a single helper function (e.g.,
build_token_introspection_response) and call it from both branches to DRY up the
code.