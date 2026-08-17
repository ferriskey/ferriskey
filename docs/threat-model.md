# Threat Model — Self-Service MFA, Step-Up & Recovery Codes

This document captures the security design decisions for the self-service
multi-factor-authentication (MFA) endpoints introduced on the
`fix/passkey-split` branch (PR #1220). It is the authoritative reference for
the threat model the review comments asked to record, in particular the
recovery-code redesign (review finding #3).

## Scope

Endpoints covered:

| Endpoint | AuthN | Sensitive? |
|---|---|---|
| `POST /realms/{r}/me/reauthenticate` | Bearer | Mints step-up token |
| `POST /realms/{r}/me/totp/setup` | Bearer | Generates pending TOTP secret |
| `POST /realms/{r}/me/totp/verify` | Bearer + step-up token | Enrolls OTP factor |
| `POST /realms/{r}/me/passkey/registration` (options + complete) | Bearer + step-up token | Enrolls passkey factor |
| `DELETE /realms/{r}/me/credentials/{id}` | Bearer + step-up token | Removes a factor |
| `POST /realms/{r}/login-actions/reset-password-with-recovery-code` | **Unauthenticated** | Burns a recovery code |

## Trust boundaries

- A **Bearer access token** proves only that the holder was authenticated at
  some point. It does NOT prove the holder currently knows the password or a
  second factor. Threats: stolen token (XSS, leaked token, compromised device),
  so any operation that changes the account's authentication posture must be
  re-bound to a fresh proof of knowledge.
- The **unauthenticated recovery-code endpoint** is reachable by anyone who
  knows (or guesses) an email address. It must never become an account-takeover
  primitive.

## Step-up authentication (defense against stolen tokens)

`/me/reauthenticate` verifies the account password and, when an OTP factor is
configured, the current OTP code. On success it mints a **short-lived
(`STEP_UP_TOKEN_TTL` = 5 min), single-use, user-bound** step-up token:

- The raw token is returned once; only a hash is stored (`step_up_tokens`).
- `consume_step_up_token` atomically deletes the row via `take`, so it cannot
  be replayed.
- It is required on `/me/totp/verify`, `/me/passkey/registration`, and
  `DELETE /me/credentials/{id}`.

This closes the gap where a stolen access token could enroll or remove factors
without ever knowing the password.

### Brute-force protection

Both reauthentication and the unauthenticated recovery-code endpoint are wired
to the existing account lockout (`lockout_compute_locked_until`, thresholds in
`RealmSetting`). Failed attempts increment `failed_login_attempts` and emit a
`reauthentication_failed` / `recovery_code_burned` (failure) `SecurityEvent`
so SeaWatch can detect guessing.

## TOTP enrollment (server-side pending secret)

`/me/totp/setup` generates the secret **server-side** and persists it in
`pending_totp_secrets` (single-use, TTL'd). `/me/totp/verify` takes **only the
code** and consumes the pending secret via `take`. Consequences:

- The client can never supply its own secret, so an attacker holding a token
  cannot silently replace the victim's authenticator.
- Enrollment still requires the step-up token (above).

## Recovery codes are a *second* factor, never a login path (finding #3)

**Design decision:** a recovery code unlocks the password-reset step only; it
never mints a session or bypasses MFA.

`complete_password_reset_with_recovery_code`:

1. Resolves the realm + user from the email.
2. Enforces the realm password policy on the submitted new password up front.
3. Enforces account lockout (per-account rate limit) before any code check.
4. Derives a fast lookup key from the submitted code and verifies **only the
   single matching candidate** (`find_recovery_code_by_lookup`) — one Argon2id
   verification instead of one per stored code, preventing the memory-hard DoS
   on this anonymous endpoint.
5. On a match: **burns** the code (deletes the credential) and issues a
   password-reset token, emailing the reset link to the account. The user still
   proves email control by clicking the link and still sets the new password;
   no access/refresh tokens or `FERRISKEY_IDENTITY` cookie are minted here.
6. Emits `recovery_code_burned` (success) + `auth.reset_password` webhook.

This matches Keycloak's model: recovery codes are a recovery *second* factor,
not a standalone authentication method. A single leaked recovery code is no
longer full account takeover.

## Audit trail

Every sensitive self-service operation emits a `SecurityEvent` (SeaWatch) and,
where relevant, an `auth.*` webhook:

| Event | Trigger |
|---|---|
| `mfa_enrolled` | OTP or passkey factor added |
| `mfa_removed` | Primary factor (passkey/OTP) removed |
| `credential_deleted` | Recovery code removed |
| `reauthentication_failed` | Step-up password/OTP check failed |
| `recovery_code_burned` | Recovery code consumed (success or failure) |

### Compensating control: factor-change email

In addition to the audit trail, the account owner is emailed whenever a factor
is added or removed (`notify_factor_change`): OTP/passkey enrollment and
credential deletion all send a "a sign-in method was added/removed" email. This
is the compensating control for the self-service MFA endpoints — even if a
stolen access token passes the step-up check, the legitimate user is alerted to
the change. The email is best-effort: if SMTP is not configured for the realm an
`EmailNotSent` event is logged and the factor operation still succeeds, so a
misconfigured realm never blocks enrollment/removal.

## Realm binding (token ↔ URL)

The middleware derives the authoritative realm from the **resolved identity**
(`output.identity.realm_id()`), not from string-parsing the `iss` claim. The
URL path realm is resolved to its id and compared against the token's realm id,
so a `root_path` containing `/realms/` or a realm rename cannot bind a token to
the wrong realm.

## Known limitations / future work

- TOTP codes remain valid for their standard 30s window (inherent to TOTP);
  the `counter` field stored on the OTP credential is unused for TOTP.
- `pending_totp_secrets` / `step_up_tokens` / `webauthn_challenges` rows are
  purged via `cleanup_expired()` on write; a periodic sweeper is recommended for
  environments with high churn.
