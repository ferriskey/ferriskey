# OIDC Conformance Gap Triage

Each item below is a self-contained follow-up issue ready to file against the
`#1090` milestone. Items are ordered by conformance impact (blocking the OIDF
Basic OP or Config OP suite first).

---

## GAP-01 — PKCE enforcement on the inbound authorization\_code flow

**Title:** feat: enforce PKCE (RFC 7636) on the inbound authorization\_code flow

**Scope:**
Accept `code_challenge` and `code_challenge_method=S256` on the `/auth` endpoint
(`AuthRequest`, `AuthInput`, `AuthSession`). Store the challenge. At token exchange
(`/token` with `grant_type=authorization_code`), require the caller to supply
`code_verifier` in `ExchangeTokenInput` and verify
`BASE64URL(SHA256(verifier)) == stored_challenge`. Reject the exchange with
`invalid_grant` if absent or mismatched.

PKCE is currently only implemented on the outgoing federation leg
(`abyss/broker_services.rs`), not on FerrisKey's own authorization server role.
The OIDF Basic OP suite sends `code_challenge` and will reject responses that
ignore it.

---

## GAP-02 — `nonce` injection into the ID token

**Title:** fix: propagate `nonce` from auth session into ID token claims

**Scope:**
`AuthSession.nonce` is stored (`entities.rs:38`) but the `IdTokenClaims` struct
(`libs/ferriskey-security/src/jwt/entities.rs:64`) has no `nonce` field.
Add `nonce: Option<String>` to `IdTokenClaims`. Thread the nonce from the auth
session through `GenerateTokenInput` and into `create_jwt`.
OIDC Core §3.1.2.1 requires the `nonce` be included in the ID token when it was
present in the authorization request. The conformance suite always sends a nonce
and verifies its presence in the ID token.

**Dependency:** Also requires that `AuthRequest` (`auth.rs`) expose `nonce` as a
query parameter so it is captured when the authorization flow starts.

---

## GAP-03 — `auth_time` claim in ID token

**Title:** feat: populate `auth_time` claim in the ID token (OIDC Core §2)

**Scope:**
`IdTokenClaims.auth_time` is declared but always set to `None`
(`services.rs:698`). Record the timestamp of the user's last interactive
authentication (at session creation in `AuthSession`) and include it in the
ID token. Required when `max_age` is sent in the authorization request.

---

## GAP-04 — Discovery document missing required and recommended metadata fields

**Title:** fix: extend `.well-known/openid-configuration` to satisfy OIDC Discovery 1.0

**Scope:**
`GetOpenIdConfigurationResponse` (`openid_configuration.rs:12–23`) is missing
the following fields required by OIDC Discovery 1.0 / RFC 8414:

- `response_types_supported` (REQUIRED) — add `["code"]`
- `subject_types_supported` (REQUIRED) — add `["public"]`
- `id_token_signing_alg_values_supported` (REQUIRED) — add `["RS256"]`
- `scopes_supported` (RECOMMENDED) — `["openid","profile","email","address","phone","offline_access"]`
- `claims_supported` (RECOMMENDED) — list of standard claims returned
- `code_challenge_methods_supported` (RECOMMENDED) — add `["S256"]` once GAP-01 is resolved
- `response_modes_supported` (OPTIONAL but tested) — `["query"]`
- `grant_types_supported` — add `urn:ietf:params:oauth:grant-type:device_code`

The OIDF `oidcc-discovery-endpoint-test` validates all REQUIRED fields and many
RECOMMENDED ones. Missing REQUIRED fields cause immediate test failure.

---

## GAP-05 — `nonce` not captured at the authorization endpoint

**Title:** fix: add `nonce` query parameter to the authorization endpoint

**Scope:**
`AuthRequest` in `auth.rs` only captures `response_type`, `client_id`,
`redirect_uri`, `scope`, and `state`. The `nonce` query parameter is silently
dropped. Add `nonce: Option<String>` to `AuthRequest` and pass it through
`AuthInput` → `AuthSession`. This is a prerequisite for GAP-02 (ID token nonce).

---

## GAP-06 — Redirect URI regex matching is overly permissive

**Title:** fix: restrict redirect\_uri validation to exact match; remove regex fallback

**Scope:**
`services.rs:1945–1956` falls back to `regex::Regex::new(&uri.value)` when an
exact match fails. Any URI pattern stored for a client is compiled as a regex and
matched against arbitrary incoming `redirect_uri` values. This allows open-redirect
attacks if a client URI contains unescaped regex metacharacters, and does not
satisfy RFC 6749 §3.1.2 / OIDC Core §3.1.2.1 which require exact URI matching.
Limit matching to exact equality only (remove the regex branch). Provide a
wildcard mechanism through an explicit prefix-match configuration field if needed.

---

## GAP-07 — Signed UserInfo response (OPTIONAL but tested in some profiles)

**Title:** feat: support signed UserInfo responses (`application/jwt`)

**Scope:**
OIDC Core §5.3.2 allows the UserInfo endpoint to return a signed JWT when the
client has registered `userinfo_signed_response_alg`. Currently `userinfo.rs`
returns plain JSON. Add opt-in signing when the client configuration requests it.
Advertise `userinfo_signing_alg_values_supported` in the discovery document when
implemented.

---

## GAP-08 — `acr` and `amr` claims in the ID token

**Title:** feat: populate `acr` and `amr` claims in the ID token

**Scope:**
OIDC Core §2 defines `acr` (Authentication Context Class Reference) and `amr`
(Authentication Methods References) as OPTIONAL standard ID token claims.
The OIDF suite sends `acr_values` in authorization requests and checks that
`acr` is reflected in the ID token. Add `acr: Option<String>` and
`amr: Option<Vec<String>>` to `IdTokenClaims`; populate them from the
authentication flow context (password, webauthn, totp, etc.).

---

## GAP-09 — Backchannel logout (OIDC Back-Channel Logout 1.0)

**Title:** feat: implement OIDC Back-Channel Logout

**Scope:**
When a user session ends (via `/logout`), send a signed logout token (JWT with
`events` claim) to each client's registered `backchannel_logout_uri`. Required for
the `oidcc-rp-initiated-logout` and related session management test plans. Advertise
`backchannel_logout_supported: true` in the discovery document once implemented.
This is a larger feature; file as a dedicated milestone item under `#1090`.

---

## GAP-10 — `kid` continuity between signing key and JWKS

**Title:** fix: verify `kid` in signed token headers matches the JWKS entry

**Scope:**
Tokens are signed with RS256. The JWKS endpoint returns `JwkKey` entries. The OIDF
suite fetches the JWKS and matches the `kid` in the token header to validate the
signature. Confirm that the `kid` embedded in the JWT header (`Header.kid`)
exactly matches the `kid` in the JWKS response for the same key. Add an integration
test that decodes a freshly-issued access token, extracts `kid`, fetches
`/jwks.json`, and asserts a matching entry exists.
