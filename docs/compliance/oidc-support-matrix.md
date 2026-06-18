# OIDC Conformance Support Matrix

Verification status of OpenID Connect Core 1.0 and related RFC requirements
against the FerrisKey codebase. Every claim is grounded in a specific source
file; the "Evidence" column links to the relevant handler or domain module.

Legend: **PASS** = fully implemented / **PARTIAL** = partially implemented /
**GAP** = not implemented or incorrectly implemented.

---

## 1. Discovery — OpenID Provider Metadata (OIDC Discovery 1.0)

| Requirement | Status | Evidence |
|---|---|---|
| `issuer` present and matches token `iss` | PASS | `openid_configuration.rs:58–59`; services.rs builds issuer from host header |
| `authorization_endpoint` | PASS | `openid_configuration.rs:66` |
| `token_endpoint` | PASS | `openid_configuration.rs:67` |
| `userinfo_endpoint` | PASS | `openid_configuration.rs:71` |
| `jwks_uri` | PASS | `openid_configuration.rs:72` — points to `/jwks.json` |
| `revocation_endpoint` | PASS | `openid_configuration.rs:68` |
| `end_session_endpoint` | PASS | `openid_configuration.rs:69` |
| `introspection_endpoint` | PASS | `openid_configuration.rs:70` |
| `response_types_supported` | GAP | Field absent from `GetOpenIdConfigurationResponse` struct |
| `response_modes_supported` | GAP | Not advertised |
| `grant_types_supported` | PARTIAL | Present in struct (`authorization_code`, `refresh_token`, `client_credentials`, `password`); missing `urn:ietf:params:oauth:grant-type:device_code` |
| `subject_types_supported` | GAP | Not advertised |
| `id_token_signing_alg_values_supported` | GAP | Not advertised (RS256 is used — see `services.rs:449`) |
| `scopes_supported` | GAP | Not advertised |
| `token_endpoint_auth_methods_supported` | PASS | `openid_configuration.rs:79–82` (`client_secret_basic`, `client_secret_post`) |
| `claims_supported` | GAP | Not advertised |
| `code_challenge_methods_supported` | GAP | Not advertised; PKCE inbound not implemented |
| `request_parameter_supported` | GAP | Not advertised; JAR not implemented |
| `request_uri_parameter_supported` | GAP | Not advertised; PAR not implemented |
| `backchannel_logout_supported` | GAP | Not implemented |
| `frontchannel_logout_supported` | GAP | Not implemented |

**Summary:** The discovery document is structurally valid but missing ~10 required or
strongly-recommended metadata fields (RFC 8414 / OIDC Discovery §3). The conformance
suite's `oidcc-discovery-endpoint-test` will fail on missing fields.

---

## 2. Authorization Endpoint (OIDC Core §3.1.2)

| Requirement | Status | Evidence |
|---|---|---|
| `response_type=code` accepted | PASS | `auth.rs` — `AuthRequest` passes `response_type` to `AuthInput`; `services.rs` creates auth session |
| `state` echoed back in redirect | PASS | `services.rs:83–91` (`format_authorization_redirect_url`) — state echoed when non-empty |
| `nonce` stored and bound to auth session | PARTIAL | `AuthSession` has `nonce` field (`entities.rs:38`); `nonce` is accepted in `AuthInput` but `AuthRequest` struct in `auth.rs` does NOT include a `nonce` query parameter — it is dropped |
| `scope=openid` triggers ID token | PASS | `services.rs:670–714` — ID token emitted when `openid` in scope |
| Redirect URI strict validation | PARTIAL | Exact match + regex fallback (`services.rs:1945–1956`); regex is overly permissive (noted in 2026-06 audit) |
| `prompt` parameter | GAP | Not handled |
| `max_age` parameter | GAP | `auth_time` always `None` in IdTokenClaims |
| `display` parameter | GAP | Not handled |
| `ui_locales` parameter | GAP | Not handled |
| **PKCE** `code_challenge` / `code_challenge_method` | GAP | `AuthRequest` and `AuthInput` have no `code_challenge` fields; PKCE is only implemented on the outgoing federation leg (`abyss/broker_services.rs:104–174`) |

---

## 3. Token Endpoint (OIDC Core §3.1.3 + RFC 6749)

| Requirement | Status | Evidence |
|---|---|---|
| `authorization_code` grant | PASS | `services.rs:921` (`authorization_code`) |
| `refresh_token` grant | PASS | `services.rs:1215` (`refresh_token`) |
| `client_credentials` grant | PASS | `services.rs` (`client_credential`) |
| `password` grant | PASS | `services.rs` (`password`) |
| Device Code grant (RFC 8628) | PASS | `token.rs:92–98` — dispatched to `poll_device_token` |
| `client_secret_basic` authentication | PASS | `basic_auth.rs` — parsed in token handler |
| `client_secret_post` authentication | PASS | `validators.rs` — `client_id` / `client_secret` form fields |
| Refresh token rotation (old token revoked on use) | PASS | `services.rs:1271–1274` — old jti deleted via `refresh_token_repository.delete` |
| Refresh token reuse detection | PARTIAL | Old token is deleted; a replay attempt hits `get_by_jti` → not-found error (implicit rejection). No explicit family-based cascade revocation. |
| `scope` downscoping on refresh | PASS | `services.rs:1264` — `claims.scope` carried forward |
| PKCE `code_verifier` validation at token exchange | GAP | `ExchangeTokenInput` has no `code_verifier` field; `authorization_code()` does not check it |

---

## 4. ID Token (OIDC Core §2)

| Requirement | Status | Evidence |
|---|---|---|
| `iss` | PASS | `IdTokenClaims.iss` set to issuer string |
| `sub` | PASS | `IdTokenClaims.sub` = user UUID |
| `aud` | PASS | `IdTokenClaims.aud` = `azp` (client\_id string) |
| `exp` | PASS | `IdTokenClaims.exp` set |
| `iat` | PASS | `IdTokenClaims.iat` set |
| `nonce` (REQUIRED when nonce in auth request) | GAP | `IdTokenClaims` struct has no `nonce` field; even if auth session stores nonce, it is never injected |
| `at_hash` (RECOMMENDED for code flow) | PASS | Computed at `services.rs:682–685` (SHA-256 left-half, base64url) |
| `auth_time` | GAP | Field present in struct but always `None` |
| `acr` | GAP | Not implemented |
| `amr` | GAP | Not implemented |
| `azp` | PASS | Present and set to `client_id` |
| ID token signed with RS256 | PASS | `services.rs:449` — `Header::new(Algorithm::RS256)` |
| Profile / email claims via mappers | PASS | Protocol mapper engine populates `additional_claims` in ID token |

---

## 5. UserInfo Endpoint (OIDC Core §5.3)

| Requirement | Status | Evidence |
|---|---|---|
| Bearer token accepted | PASS | `userinfo.rs` — `OptionalToken` extractor reads Bearer |
| Returns `sub` | PASS | `UserInfoResponse` includes subject |
| `profile` scope claims | PASS | Mapper engine (`user_property_mapper`, `user_attribute_mapper`) |
| `email` scope claims | PASS | Mapper engine |
| Returns JSON by default | PASS | Handler returns `Response<UserInfoResponse>` (JSON) |
| Signed UserInfo response (JWT) | GAP | Not implemented |

---

## 6. Token Revocation (RFC 7009)

| Requirement | Status | Evidence |
|---|---|---|
| `POST /revoke` endpoint present | PASS | `revoke.rs` |
| Accepts `token` + `token_type_hint` | PASS | `RevokeTokenRequestValidator` |
| Returns 200 even for unknown tokens | PASS | RFC 7009 §2.2 requires silent success |
| Requires client authentication | PASS | `client_id` required in form |

---

## 7. Token Introspection (RFC 7662)

| Requirement | Status | Evidence |
|---|---|---|
| `POST /token/introspect` present | PASS | `introspect.rs` |
| Requires confidential client credentials | PASS | `basic_auth` or `client_secret_post` checked; unauthorized returns 401 |
| Returns `active`, `sub`, `exp`, `iat`, `scope` | PASS | `TokenIntrospectionResponse` (from `ferriskey-domain` crate) |
| Returns `active: false` for invalid tokens | PASS | Service returns `false` on lookup/validation failure |

---

## 8. RP-Initiated Logout (OIDC Session Management / RP-Initiated Logout 1.0)

| Requirement | Status | Evidence |
|---|---|---|
| `GET /logout` | PASS | `logout.rs:120–127` (`logout_get`) |
| `POST /logout` | PASS | `logout.rs:144–151` (`logout_post`) |
| `id_token_hint` accepted | PASS | `LogoutRequestValidator.id_token_hint`; validated in `end_session` |
| `post_logout_redirect_uri` accepted | PASS | Redirect returned when present |
| `state` echoed in redirect | PASS | `EndSessionInput.state` forwarded |
| `client_id` accepted | PASS | `LogoutRequestValidator.client_id` |
| Session cookies cleared | PASS | `clear_session_cookies_headers` sets both cookies to removal |
| Backchannel logout | GAP | Not implemented |
| Frontchannel logout | GAP | Not implemented |

---

## 9. JWKS / Key Material (RFC 7517)

| Requirement | Status | Evidence |
|---|---|---|
| JWKS endpoint (`/jwks.json`) | PASS | `get_certs.rs:66–71` |
| `/certs` alias | PASS | `get_certs.rs:43–48` |
| RS256 keys exposed | PASS | `JwkKey` type used; algorithm RS256 (`services.rs:449`) |
| Key rotation support | PARTIAL | Keys fetched from keystore per realm; no automated rotation schedule documented |
| `kid` header in signed tokens | PARTIAL | Present in `Header` via `ferriskey-security`; verify `kid` matches JWKS |

---

## 10. Scopes and Claims

| Requirement | Status | Evidence |
|---|---|---|
| `openid` scope required for ID token | PASS | `services.rs:670` |
| `profile` scope | PASS | Defined in `scope.rs`; mappers in `user_property_mapper.rs` |
| `email` scope | PASS | Defined in `scope.rs`; mapper in `user_property_mapper.rs` |
| `address` scope | PARTIAL | Scope constant defined (`scope.rs:8`); no dedicated mapper verified |
| `phone` scope | PARTIAL | Scope constant defined; no dedicated mapper verified |
| `offline_access` scope | PARTIAL | Scope defined; no explicit offline token lifetime differentiation verified |

---

## Overall Readiness Summary

| Profile | Expected OIDF Result | Blocking Gaps |
|---|---|---|
| Config OP | FAIL | Discovery missing 10+ fields (GAP-05) |
| Basic OP | FAIL | PKCE inbound (GAP-01), nonce in ID token (GAP-04) |
| Hybrid OP | Not applicable | `response_type=token` / implicit not implemented |
| RP-Initiated Logout | PARTIAL FAIL | Backchannel/frontchannel logout absent |

Addressing GAP-01, GAP-04, GAP-05 (and the sub-gaps under each) brings Basic OP
and Config OP into reach. See `oidc-gap-triage.md` for scoped issue descriptions.
