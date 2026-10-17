# Threat model - FerrisKey

Lightweight threat model written for the CNCF General Technical Review ([general-technical-review.md](./general-technical-review.md)). It lists, per flow, what an attacker wants, what stops them today, and where the evidence lives. It is a living document: a new flow or a new trust boundary adds a table here.

Weaknesses that are not fixed yet are not listed in this file. They go through the private process described in [SECURITY.md](../../SECURITY.md) and appear here once the fix is released.

## Scope

- The FerrisKey API (`api/`, `core/`, `libs/`), the admin console and the login portal (`front/`).
- The Kubernetes operator (`operator/`) and the Helm charts (`charts/`).
- Out of scope: the relying applications, the reverse proxy or ingress in front of FerrisKey, the PostgreSQL server, the SMTP server and the upstream identity providers. Their security is the operator's responsibility; this document states what FerrisKey expects from them.

## Assets

| Asset | Where it lives |
| --- | --- |
| User credentials (password hashes, TOTP seeds, WebAuthn keys, recovery code hashes) | PostgreSQL |
| Realm signing keys | PostgreSQL |
| Client secrets, webhook secrets, identity provider secrets | PostgreSQL |
| Issued tokens: authorization codes, access, refresh and ID tokens | PostgreSQL (codes, refresh lineage) and the client side |
| Sessions (`FERRISKEY_SESSION`, `FERRISKEY_SSO`) | Browser cookies and PostgreSQL |
| Security event journal | PostgreSQL, hash-chained |
| Admin and database passwords created by the operator | Kubernetes Secrets |

## Actors

- **Anonymous attacker on the network**: reaches the public endpoints (authorization, token, discovery, login portal).
- **Malicious or compromised relying application**: holds a valid `client_id`, possibly a secret.
- **Authenticated user**: holds valid tokens in one realm.
- **Realm administrator**: holds admin rights in one realm and tries to reach another one.
- **Attacker inside the cluster**: reaches pod-to-pod traffic or the Kubernetes API with limited rights.

## Trust boundaries

1. Browser to FerrisKey (login portal, authorization endpoint, console).
2. Relying application to FerrisKey (token, introspection, revocation endpoints).
3. FerrisKey to outbound targets (identity providers, client metadata documents, webhooks, SMTP).
4. FerrisKey to PostgreSQL.
5. Realm to realm inside one deployment. This is the boundary FerrisKey enforces itself, so it gets the most tests.
6. Operator to the Kubernetes API.

## Flow 1 - Browser login

| Threat | Mitigation in place | Evidence |
| --- | --- | --- |
| Authorization code stolen in transit or from logs | PKCE with S256, required at registration for public clients and for new clients unless they opt out | `api/tests/pkce_test.rs` |
| Redirect to an attacker address | Redirect URIs matched against the client's registered values; post-logout redirect must equal a registered value | `api/tests/redirect_uri_validation_test.rs`, `api/tests/features/oidc/logout.feature` |
| Password guessing | Argon2id hashing, legacy hashes rehashed on login, per-realm account lockout (10 failures, 15 minutes by default) | `api/tests/account_lockout_test.rs` |
| MFA bypass by replaying a step token | The login-action token is single use | `api/tests/mfa_bypass_test.rs` |
| Recovery code or magic link reuse | Recovery codes stored hashed, burnt on use | `api/tests/trident_cross_realm_test.rs` |
| Session cookie theft through scripts | Session cookies are `HttpOnly` and `SameSite=Lax`, `Secure` on HTTPS | `api/tests/sso_session_test.rs` |
| Login or MFA step replayed through another realm's URL | Every step is bound to the realm of its session | `api/tests/trident_cross_realm_test.rs`, `api/tests/auth_session_cross_realm_test.rs` |
| ID token replay | `nonce` carried from the request into the ID token | `api/tests/nonce_id_token_test.rs` |

## Flow 2 - Token endpoint

| Threat | Mitigation in place | Evidence |
| --- | --- | --- |
| Code redeemed by another client, another realm, another redirect URI, or after expiry | The code is bound to its client, realm, redirect URI and protocol, expires after 10 minutes, and is claimed atomically | `api/tests/oidc_conformance_test.rs`, `api/tests/oidc_cross_realm_test.rs` |
| Code replay | A second redemption is refused and revokes the tokens the code produced | `api/tests/features/oidc/authorization_code.feature` |
| Refresh token theft | Rotation on every use; reuse of a rotated token revokes the whole family | `api/tests/refresh_token_rotation_test.rs`, `api/tests/features/oidc/token_endpoint.feature` |
| Confidential client impersonation | Client authentication by `client_secret_basic` or `client_secret_post`, compared in constant time, the same way on every OAuth endpoint | `api/tests/token_basic_auth_test.rs` |
| Introspection used as an oracle | Introspection requires a confidential client with the `introspect` role | `api/tests/token_revocation_test.rs` |
| Token exchange used to escalate | Exchange disabled by default per client; delegation and impersonation are separate opt-ins; audiences limited to resources the client owns | `api/tests/token_exchange_test.rs`, `api/tests/token_exchange_policy_test.rs` |
| Long-lived stolen access token | Access tokens last 300 seconds by default, refresh tokens 24 hours, both configurable per client | realm and client settings |

## Flow 3 - Admin API

| Threat | Mitigation in place | Evidence |
| --- | --- | --- |
| A realm administrator reads or changes another realm | Realm is resolved through one scoped resolver; a source-level test freezes the list of files allowed to bypass it | `api/tests/realm_scope_ratchet_test.rs` |
| Probing which realms or objects exist | A missing right in one's own realm answers 403; a foreign realm, and any object it holds, answers 404 | `api/tests/realm_cross_realm_test.rs`, `api/tests/*_cross_realm_test.rs` |
| Cross-realm access through nested objects (clients, roles, users, organizations, webhooks, themes, sessions) | One cross-realm suite per domain | `api/tests/*_cross_realm_test.rs` (15 suites) |
| Permission checks drifting between endpoints | One authorization matrix (`AUTHZ_MATRIX`) in the `ferriskey-authz` crate | `libs/ferriskey-authz` |
| Covering tracks after an admin action | Security events are hash-chained and the chain can be verified | `api/tests/seawatch_test.rs` |
| Browser-based calls from an unknown origin | CORS allow-list from `ALLOWED_ORIGINS` and per-client web origins, no wildcard, empty by default | `api/tests/web_origin_test.rs` |
| SSRF through webhooks or client metadata documents | Outbound targets in private address ranges are refused by default | `libs/ferriskey-webhook` |
| Forged webhook deliveries | Payloads signed with HMAC-SHA256 | `libs/ferriskey-webhook` |
| SQL injection | Queries go through SeaORM and sqlx with bound parameters | `core/src/infrastructure` |

**Design choice to know:** users of the `master` realm administer every realm. Deployments that host several tenants should keep `master` limited to platform operators.

## Flow 4 - Operator and deployment

| Threat | Mitigation in place | Evidence |
| --- | --- | --- |
| Weak or shared bootstrap passwords | The operator generates the admin and database passwords randomly and stores them in Kubernetes Secrets | `operator/src/infrastructure/cluster/manifests/api/secret.rs` |
| Container escape or privilege escalation from the operator | Non-root, `RuntimeDefault` seccomp, no privilege escalation, all capabilities dropped, read-only root filesystem | `charts/ferriskey-operator/values.yaml` |
| Operator abused to reach unrelated resources | The ClusterRole covers only the resource kinds the operator manages | `charts/ferriskey-operator/templates/operator/cluster-role.yaml` |

## What FerrisKey expects from the deployment

- TLS terminated in front of the API, or the built-in rustls listener enabled. The issuer and cookie `Secure` flag follow the scheme the API sees.
- PostgreSQL reached over TLS (`sslMode: require` on the cluster resource).
- `/metrics` and the OpenAPI pages (`/swagger-ui`, `/redoc`, `/scalar`) reachable only from the internal network.
- The default `admin`/`admin` credentials from `env.example` replaced before any exposure.

## Planned

- External security review with TAG Security and Compliance after Sandbox acceptance.
- Extend this model when new flows ship (SAML IdP-initiated login, organizations, authorization rules with Cedar).
