# Fix Forgot Password OAuth Redirect Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When a user is in the middle of an OAuth flow (a `FERRISKEY_SESSION` cookie exists) and resets their password via the "Forgot password" flow, resume the original OAuth flow and redirect to the client's `redirect_uri` instead of dumping the user on the FerrisKey console (`/realms/{realm}/overview`).

**Architecture:** Mirror the existing `MagicLink` pattern. Persist the `auth_session_code` on the `password_reset_tokens` row at the request step (read from the `FERRISKEY_SESSION` cookie), then at completion time resolve the associated `AuthSession`, mint an authorization code, and return the resulting `login_url`. The frontend uses that URL (if present) to perform a full-page redirect, otherwise it falls back to the current "log into the console" behavior.

**Tech Stack:** Rust / Axum / SeaORM (backend), React / TypeScript / TanStack Query (frontend), PostgreSQL.

**Issue:** [#1041](https://github.com/ferriskey/ferriskey/issues/1041) — `redirect to the correct page after clicking 'Forgot password'`.

**PR decomposition (per user preference):** Four sequential PRs — Domain → Infra → API → Frontend. Pause between each PR.

---

## Reference: existing pattern to mirror

Magic link already solves the same problem. Key reference points (do **not** modify these — they're the template to follow):

- `libs/ferriskey-trident/src/entities/mod.rs:60-72` — `MagicLink` carries `auth_session_code: Option<Uuid>`
- `core/src/domain/trident/ports.rs:142-149` — `MagicLinkInput.session_code: Option<String>` (with explanatory doc-comment)
- `core/src/domain/trident/services.rs:1196-1208` — `request` step parses the cookie value and persists it on the row
- `core/src/domain/trident/services.rs:1364-1428` — `verify` step resolves the `AuthSession` from the stored `auth_session_code` and calls `store_auth_code_and_generate_login_url`
- `core/src/infrastructure/repositories/magic_link_repository.rs:31-74` — mapper + insert persist `auth_session_code`
- `api/src/application/http/trident/handlers/magic_link.rs:60-91` — handler reads `cookie.get("FERRISKEY_SESSION")` and passes it through
- `core/migrations/20260404000000_add_auth_session_code_to_magic_links.{up,down}.sql` — schema template

---

## PR 1 — Domain layer

**Branch:** `1041-forgot-password-redirect-domain`

**Scope:** Carry `auth_session_code` through the domain entity, ports (input/output value objects), and service logic. The repository implementation will accept the new field but ignore it for now (persisted as `None`). No schema changes yet — domain stays compileable and unit-testable in isolation.

### Task 1.1: Add `auth_session_code` to the `PasswordResetToken` domain entity

**Files:**
- Modify: `libs/ferriskey-trident/src/entities/mod.rs:80-95`

- [ ] **Step 1: Add the field**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub token_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Session code of the AuthSession that was active when the password reset
    /// was requested. Used at completion time to redirect the user back to the
    /// original OAuth client (e.g. an external SPA) instead of dropping them on
    /// the FerrisKey console.
    pub auth_session_code: Option<Uuid>,
}
```

- [ ] **Step 2: Verify the crate still compiles**

Run: `cargo check -p ferriskey-trident`
Expected: clean compile (we have not yet touched the mapper that constructs `PasswordResetToken`, so this may surface "missing field" errors in `core` — that's expected and addressed in Task 1.2).

### Task 1.2: Carry `session_code` through `RequestPasswordResetInput` and `CompletePasswordResetOutput`

**Files:**
- Modify: `core/src/domain/trident/ports.rs:156-171`

- [ ] **Step 1: Add `session_code` to the input and `login_url` to the output**

```rust
pub struct RequestPasswordResetInput {
    pub realm_name: String,
    pub email: String,
    pub base_url: String,
    /// Session code from the FERRISKEY_SESSION cookie at request time,
    /// stored so completion can resume the original OAuth flow.
    pub session_code: Option<String>,
}

pub struct CompletePasswordResetInput {
    pub token_id: Uuid,
    pub token: String,
    pub new_password: String,
}

pub struct CompletePasswordResetOutput {
    pub user_id: Uuid,
    pub realm_id: Uuid,
    /// When the password reset was initiated inside an OAuth flow, this is the
    /// login URL (containing an authorization code) the browser should be
    /// redirected to so the original client gets its callback.
    pub login_url: Option<String>,
}
```

### Task 1.3: Persist `auth_session_code` when creating a `PasswordResetToken`

**Files:**
- Modify: `core/src/domain/trident/services.rs:1442-1519` (the `request_password_reset` function body)

- [ ] **Step 1: Parse the cookie value and store it on the row**

In `request_password_reset`, just above the `PasswordResetToken { … }` literal (currently at line 1509), parse the input:

```rust
let auth_session_code = input
    .session_code
    .as_deref()
    .and_then(|s| Uuid::parse_str(s).ok());
```

Then add the field to the struct literal:

```rust
let prt = PasswordResetToken {
    id: generate_uuid_v7(),
    user_id: user.id,
    realm_id: realm.id.into(),
    token_id,
    token_hash: token_hash.hash,
    created_at: Utc::now(),
    expires_at,
    auth_session_code,
};
```

- [ ] **Step 2: Update the existing unit tests in the same file**

Search the file for `PasswordResetToken {` and `RequestPasswordResetInput {`. Add `auth_session_code: None,` and `session_code: None,` respectively to every existing test literal so the test module still compiles. Specifically:

- `services.rs:1509` already updated above
- `services.rs:2005` — `request_password_reset(RequestPasswordResetInput { … })` → add `session_code: None,`
- `services.rs:2046` — same
- `services.rs:2106` — same
- `services.rs:2143` — same
- `services.rs:2161` — `PasswordResetToken { … }` → add `auth_session_code: None,`
- `services.rs:2263` — same
- `services.rs:2305` — same

- [ ] **Step 3: Run the trident unit tests**

Run: `cargo test -p ferriskey-core --lib domain::trident`
Expected: all existing tests pass with no behavior change.

### Task 1.4: Resolve `AuthSession` and mint `login_url` in `complete_password_reset`

**Files:**
- Modify: `core/src/domain/trident/services.rs:1674-1777`

Pattern reference: `verify_magic_link` at `services.rs:1350-1440` already does exactly this — fetch by session_code, validate realm match, call `store_auth_code_and_generate_login_url`.

- [ ] **Step 1: Capture `auth_session_code` after the validity checks pass**

Right after the existing token verification (currently around line 1708, after the `if !is_valid { … }` block) and **before** any side-effecting writes, capture the session code so we still have access to it after `prt` is moved/consumed:

```rust
let auth_session_code = prt.auth_session_code;
```

- [ ] **Step 2: After step 8 (webhook), compute `login_url` and return it**

Replace the existing `Ok(CompletePasswordResetOutput { user_id, realm_id })` at line 1776 with:

```rust
let login_url = if let Some(session_code) = auth_session_code {
    match self
        .auth_session_repository
        .get_by_session_code(session_code)
        .await
    {
        Ok(auth_session) if Uuid::from(auth_session.realm_id) == realm_id => {
            match store_auth_code_and_generate_login_url::<AS>(
                &self.auth_session_repository,
                &auth_session,
                user_id,
            )
            .await
            {
                Ok(url) => Some(url),
                Err(e) => {
                    warn!(
                        "Failed to generate login URL after password reset, falling back to console: {}",
                        e
                    );
                    None
                }
            }
        }
        Ok(auth_session) => {
            warn!(
                "AuthSession realm {} does not match password reset realm {}, falling back to console",
                Uuid::from(auth_session.realm_id),
                realm_id
            );
            None
        }
        Err(_) => {
            // Session might have expired or been purged between request and completion.
            // Falling back to console-login is safer than returning a 500 here.
            None
        }
    }
} else {
    None
};

Ok(CompletePasswordResetOutput {
    user_id,
    realm_id,
    login_url,
})
```

Note: `auth_session.realm_id` is a `RealmId` newtype, hence the `Uuid::from(...)` call (same pattern as `verify_magic_link` at line 1380).

- [ ] **Step 3: Run trident unit tests**

Run: `cargo test -p ferriskey-core --lib domain::trident`
Expected: existing tests still pass (they all pass `auth_session_code: None` so `login_url` is always `None`, matching the previous output shape semantically).

### Task 1.5: Add a unit test for the new behavior

**Files:**
- Modify: `core/src/domain/trident/services.rs` (test module at the bottom)

- [ ] **Step 1: Add the test next to the other `complete_password_reset_*` tests** (around line 2247)

The test must cover the new branch: when `auth_session_code` is set on the stored token AND the corresponding session exists, the output must contain a `login_url`.

Look for an existing happy-path test like `complete_password_reset_valid_token_succeeds` (search the file for `complete_password_reset` to locate it). Copy it, change the name to `complete_password_reset_with_session_returns_login_url`, set `auth_session_code: Some(<test uuid>)` on the `PasswordResetToken`, and add the expectations on `auth_session_repository` so that `get_by_session_code` returns a valid `AuthSession` for the same realm. Assert `result.login_url.is_some()`.

(Defer this to a follow-up if mockall ceremony is heavy — minimum viable: assert in `complete_password_reset_valid_token_succeeds` that `result.login_url.is_none()` when `auth_session_code` is `None`, which adds coverage without new mocking.)

- [ ] **Step 2: Run new test**

Run: `cargo test -p ferriskey-core --lib domain::trident::services::tests::complete_password_reset`
Expected: all matching tests pass.

### Task 1.6: Keep the repository implementation compiling

**Files:**
- Modify: `core/src/infrastructure/repositories/password_reset_token_repository.rs:29-43`

The mapper would silently drop the new field if we left it as-is; once the column lands in PR 2 it will be needed. For PR 1, just default it to `None` so the workspace compiles. (We are intentionally not persisting it yet — the column doesn't exist.)

- [ ] **Step 1: Add `auth_session_code: None` to the `From<PrtModel> for PasswordResetToken` mapper**

```rust
impl From<PrtModel> for PasswordResetToken {
    fn from(model: PrtModel) -> Self {
        let created_at: DateTime<Utc> = model.created_at.into();
        let expires_at: DateTime<Utc> = model.expires_at.into();

        PasswordResetToken {
            id: model.id,
            user_id: model.user_id,
            realm_id: model.realm_id,
            token_id: model.token_id,
            token_hash: model.token_hash,
            created_at,
            expires_at,
            auth_session_code: None, // wired up in PR 2
        }
    }
}
```

- [ ] **Step 2: Workspace check**

Run: `cargo check --workspace`
Expected: clean compile.

### Task 1.7: Commit PR 1

- [ ] **Step 1: Stage and create commit**

(Per user preference, **do not run `git commit`** — instead, propose to the user the branch name and commit message below for them to apply.)

- Branch: `1041-forgot-password-redirect-domain`
- Commit message:

```
fix(trident): carry auth_session_code through password reset domain

Mirror the magic-link pattern so a password reset initiated inside an
OAuth flow can later be resolved back to the original AuthSession.

- Add `PasswordResetToken.auth_session_code: Option<Uuid>`
- Add `RequestPasswordResetInput.session_code: Option<String>`
- Add `CompletePasswordResetOutput.login_url: Option<String>`
- In `complete_password_reset`, when the token has an auth_session_code
  and the matching session still exists, mint an authorization-code URL
  via `store_auth_code_and_generate_login_url` and surface it on the
  output. Fall back to `None` on any lookup failure.

Refs #1041
```

---

## PR 2 — Infrastructure layer

**Branch:** `1041-forgot-password-redirect-infra`

**Scope:** Database migration + SeaORM entity regen + repository implementation actually persists and reads back `auth_session_code`. After this PR, the domain logic landed in PR 1 starts working end-to-end at the repository boundary — the API still doesn't pass `session_code` in (so behavior is unchanged from the user's POV).

### Task 2.1: Migration

**Files:**
- Create: `core/migrations/<NEW_TIMESTAMP>_add_auth_session_code_to_password_reset_tokens.up.sql`
- Create: `core/migrations/<NEW_TIMESTAMP>_add_auth_session_code_to_password_reset_tokens.down.sql`

Use a timestamp **after** the most recent migration in `core/migrations/`. Inspect that directory first; pick a timestamp greater than the latest one and after today's date (`20260530…`).

- [ ] **Step 1: Write the up migration**

Content (mirror `20260404000000_add_auth_session_code_to_magic_links.up.sql`):

```sql
ALTER TABLE password_reset_tokens ADD COLUMN auth_session_code UUID;
```

- [ ] **Step 2: Write the down migration**

```sql
ALTER TABLE password_reset_tokens DROP COLUMN auth_session_code;
```

- [ ] **Step 3: Run the migration locally**

Run:
```bash
cd core
DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5432/ferriskey sqlx migrate run
```

Expected: migration applied without error. Sanity check the column landed:
```bash
psql postgres://ferriskey:ferriskey@localhost:5432/ferriskey -c "\d password_reset_tokens"
```

### Task 2.2: Regenerate the SeaORM entity

**Files:**
- Modify (auto-generated): `core/src/entity/password_reset_tokens.rs`

- [ ] **Step 1: Regenerate**

Run from `core/` (with `sea-orm-cli` already installed per CLAUDE.md). `--expanded-format` is **required** — it produces the verbose layout the rest of the codebase uses:

```bash
cd core
sea-orm-cli generate entity \
  -o src/entity \
  --expanded-format \
  --database-url postgres://ferriskey:ferriskey@127.0.0.1:5432/ferriskey
```

- [ ] **Step 2: Inspect the diff**

Run: `git diff core/src/entity/password_reset_tokens.rs`
Expected: a new `auth_session_code: Option<Uuid>` field on `Model`, a new `AuthSessionCode` `Column` variant, and `Self::AuthSessionCode => ColumnType::Uuid.def().null(),` in `ColumnTrait::def`.

If `sea-orm-cli` over-generates by touching other entity files, revert any unrelated changes (`git checkout -- core/src/entity/<other>.rs`).

### Task 2.3: Wire the field through the repository

**Files:**
- Modify: `core/src/infrastructure/repositories/password_reset_token_repository.rs:29-64`

- [ ] **Step 1: Update the `From<PrtModel>` mapper to read the column**

Replace the `auth_session_code: None, // wired up in PR 2` line introduced in PR 1 with:

```rust
auth_session_code: model.auth_session_code,
```

- [ ] **Step 2: Update `create` to persist the column**

Add to the `PrtActiveModel` literal inside `create`:

```rust
let active_model = PrtActiveModel {
    id: Set(token.id),
    user_id: Set(token.user_id),
    realm_id: Set(token.realm_id),
    token_id: Set(token.token_id),
    token_hash: Set(token.token_hash.clone()),
    created_at: Set(token.created_at.fixed_offset()),
    expires_at: Set(token.expires_at.fixed_offset()),
    auth_session_code: Set(token.auth_session_code),
};
```

- [ ] **Step 3: Workspace check + tests**

Run: `cargo check --workspace && cargo test -p ferriskey-core`
Expected: clean compile, all tests green.

### Task 2.4: Commit PR 2

- [ ] **Step 1: Propose commit (do not run `git commit`)**

- Branch: `1041-forgot-password-redirect-infra`
- Commit message:

```
fix(trident): persist auth_session_code on password_reset_tokens

Adds a nullable `auth_session_code` column on `password_reset_tokens`,
regenerates the SeaORM entity, and wires it through the Postgres
repository (read in the mapper, write in `create`).

Refs #1041
```

---

## PR 3 — API layer

**Branch:** `1041-forgot-password-redirect-api`

**Scope:** Forgot-password handler reads the `FERRISKEY_SESSION` cookie and forwards it to the domain. Reset-password handler returns the new `login_url` field.

### Task 3.1: Forward `session_code` from the forgot-password cookie

**Files:**
- Modify: `api/src/application/http/trident/handlers/forgot_password.rs`

Mirror exactly what `send_magic_link` does (`api/src/application/http/trident/handlers/magic_link.rs:60-84`).

- [ ] **Step 1: Add `CookieManager` and read `FERRISKEY_SESSION`**

Replace the entire `forgot_password` handler:

```rust
use axum::extract::{Path, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::trident::ports::{RequestPasswordResetInput, TridentService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct ForgotPasswordResponse;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/forgot-password",
    tag = "auth",
    summary = "Request a password reset",
    description = "Sends a password reset email to the user if the email exists in the realm. Always returns 204 to prevent email enumeration.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Request processed (email sent if user exists)", body = ForgotPasswordResponse),
        (status = 400, description = "Bad Request", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn forgot_password(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<ForgotPasswordRequest>,
) -> Result<Response<ForgotPasswordResponse>, ApiError> {
    let base_url = state.args.webapp_url.trim_end_matches('/').to_string();
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .map(|c| c.value().to_string());

    state
        .service
        .request_password_reset(RequestPasswordResetInput {
            realm_name,
            email: payload.email,
            base_url,
            session_code,
        })
        .await?;

    Ok(Response::OK(ForgotPasswordResponse))
}
```

### Task 3.2: Expose `login_url` from the reset-password response

**Files:**
- Modify: `api/src/application/http/trident/handlers/reset_password.rs`

The current handler ignores everything from `CompletePasswordResetOutput` except `user_id` / `realm_id` (used to mint a fresh JWT). We add the new field to the response body.

- [ ] **Step 1: Extend the response with `login_url`**

The handler currently returns `axum::Json(token)` where `token: JwtToken`. We need a wrapper so the frontend can read `login_url`. Add a new response struct above the handler:

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    #[serde(flatten)]
    pub token: JwtToken,
    /// Present when the password reset happened inside an OAuth flow.
    /// The frontend should perform a full-page redirect to this URL to
    /// complete the original authorization flow and reach the client's
    /// `redirect_uri`.
    pub login_url: Option<String>,
}
```

(Add the `serde::Serialize` import if not already present.)

- [ ] **Step 2: Plumb `login_url` through and update the OpenAPI annotation**

Update `reset_password_with_token`:

```rust
#[utoipa::path(
    post,
    path = "/login-actions/reset-password",
    tag = "auth",
    summary = "Reset password with token",
    description = "Completes the password reset flow by verifying the token and setting a new password. Returns authentication tokens to log the user in directly. When the reset was initiated inside an OAuth flow, also returns `login_url` so the browser can resume that flow.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = ResetPasswordResponse),
        (status = 400, description = "Invalid or expired token", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn reset_password_with_token(
    Path(_realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    ValidateJson(payload): ValidateJson<ResetPasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .service
        .complete_password_reset(CompletePasswordResetInput {
            token_id: payload.token_id,
            token: payload.token,
            new_password: payload.new_password,
        })
        .await?;

    let is_secure = base_url.starts_with("https://");

    let token = state
        .service
        .generate_tokens_for_user(GenerateTokensForUserInput {
            user_id: result.user_id,
            realm_id: result.realm_id,
            base_url,
            client_id: None,
        })
        .await?;

    let mut identity_cookie = Cookie::build((IDENTITY_COOKIE, token.access_token().to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if is_secure {
        identity_cookie = identity_cookie.secure(true);
    }

    let cookie_value = HeaderValue::from_str(&identity_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))?;

    Ok((
        StatusCode::OK,
        [(SET_COOKIE, cookie_value)],
        axum::Json(ResetPasswordResponse {
            token,
            login_url: result.login_url,
        }),
    ))
}
```

- [ ] **Step 3: Register the new schema in OpenAPI**

If `ResetPasswordResponse` is referenced anywhere in `api/src/application/http/server/openapi.rs` schema list (the codebase typically lists every `ToSchema`), add it there. Search:

Grep: `JwtToken,` in `api/src/application/http/server/openapi.rs` — add `ResetPasswordResponse,` adjacent to it if it appears in a `components(schemas(...))` block.

- [ ] **Step 4: Verify compile**

Run: `cargo check -p ferriskey-api`
Expected: clean.

### Task 3.3: API integration test

**Files:**
- Modify or create: `api/tests/it/` (use the existing trident or auth integration test file as a host — search for a test that already exercises `/login-actions/reset-password`)

- [ ] **Step 1: Add a test that asserts `login_url` is present when the cookie was set**

Pseudocode (adapt to the existing `it` harness — see `api/tests/it/postgres_context.rs` and other `it` tests for the actual test-context boilerplate):

1. Start the test server with the postgres test context.
2. Hit `GET /realms/master/protocol/openid-connect/auth?client_id=…&redirect_uri=https://example.test/cb&response_type=code&state=abc` and capture the `FERRISKEY_SESSION` cookie from the response.
3. Hit `POST /realms/master/login-actions/forgot-password` with that cookie + a valid user email.
4. Pull the freshly created `password_reset_tokens` row from the DB and assert `auth_session_code` is `Some`.
5. Hit `POST /realms/master/login-actions/reset-password` with the token + a new password.
6. Assert the JSON response body has `login_url` set to a string containing `https://example.test/cb` (or at least containing `code=`).

Also add a negative test: same flow but without the session cookie → `login_url` is `null` / absent.

- [ ] **Step 2: Run integration tests**

Run: `cargo test --test it -- reset_password`
Expected: all pass.

### Task 3.4: Commit PR 3

- [ ] **Step 1: Propose commit (do not run `git commit`)**

- Branch: `1041-forgot-password-redirect-api`
- Commit message:

```
fix(api): resume OAuth flow after password reset

The forgot-password handler now reads the FERRISKEY_SESSION cookie and
forwards the session_code into the domain so it's persisted on the
password_reset_tokens row.

The reset-password handler returns the new `login_url` field
(serde-flattened with the existing JwtToken) so the frontend can
perform a full-page redirect that completes the original OAuth flow
instead of dumping the user on the FerrisKey console.

Refs #1041
```

---

## PR 4 — Frontend layer

**Branch:** `1041-forgot-password-redirect-frontend`

**Scope:** Regenerate the OpenAPI-driven client and update the reset-password feature to honor `login_url` when present.

### Task 4.1: Regenerate the API client + tanstack types

**Files:**
- Regenerated: `front/src/api/api.client.ts`, `front/src/api/api.tanstack.ts`

- [ ] **Step 1: Regenerate**

The API server must be running on `:3333` first (`cd api && cargo run`). Then, from `front/`:

```bash
pnpm typed-openapi http://localhost:3333/api-docs/openapi.json \
  -o src/api/api.client.ts \
  --tanstack=api.tanstack.ts
```

This regenerates `src/api/api.client.ts` (raw client) and `src/api/api.tanstack.ts` (TanStack Query wrappers used as `window.tanstackApi` throughout the frontend).

- [ ] **Step 2: Sanity-check the diff**

Run: `git diff front/src/api/api.client.ts | grep -B 1 -A 2 login_url`
Expected: the new `login_url` property appears in the reset-password response shape.

### Task 4.2: Honor `login_url` in the reset-password feature

**Files:**
- Modify: `front/src/pages/authentication/feature/page-reset-password-feature.tsx:48-67`

- [ ] **Step 1: Replace `onSuccess` to branch on `login_url`**

```tsx
function onSubmit(data: ResetPasswordSchema) {
  if (missingParams) return

  resetPassword(
    {
      path: { realm_name: realm_name ?? 'master' },
      body: {
        token_id: tokenId,
        token: token,
        new_password: data.password,
      },
    },
    {
      onSuccess: (data) => {
        setAuthTokens(data.access_token, data.refresh_token, data.id_token ?? null)

        // If the reset was initiated mid-OAuth, the API hands us back a URL
        // that completes the original authorization flow and redirects the
        // user to the client's redirect_uri. Otherwise fall back to the
        // console for direct console-style password resets.
        if (data.login_url) {
          window.location.href = data.login_url
          return
        }

        navigate(`/realms/${realm_name}/overview`)
      },
    }
  )
}
```

- [ ] **Step 2: Type-check the frontend**

Run: `cd front && pnpm run build` (or `pnpm tsc --noEmit` if available)
Expected: clean, with `data.login_url` typed as `string | null | undefined`.

### Task 4.3: Smoke-test in the browser

The repo runs the dev server on port 5555 (per `CLAUDE.md`).

- [ ] **Step 1: Start backend + frontend**

```bash
# terminal 1
cd api && cargo run
# terminal 2
cd front && pnpm run dev
```

- [ ] **Step 2: End-to-end golden path**

1. Configure an OAuth client with redirect_uri `http://localhost:3000/cb` (or any test app you have).
2. From a test app at that origin, kick off the OAuth flow → land on the FerrisKey login page (with `client_id` and `redirect_uri` in URL params, and the `FERRISKEY_SESSION` cookie set on the FerrisKey domain).
3. Click **Forgot your password?** → submit your email.
4. Open the reset email (or grab the link from server logs if SMTP isn't configured).
5. Submit a new password on the reset page.
6. **Expected**: the browser is redirected to `http://localhost:3000/cb?code=…&state=…` — the original `redirect_uri`. **Not** to `/realms/{realm}/overview`.

- [ ] **Step 3: Edge-case check — console-initiated reset**

1. Sign in to the FerrisKey console directly (no OAuth client, no `FERRISKEY_SESSION` cookie from a third-party flow).
2. Hit the forgot-password flow from the console login page.
3. **Expected**: after reset, the user lands on `/realms/{realm}/overview` (current/legacy behavior).

- [ ] **Step 4: Edge-case check — stale session**

1. Start an OAuth flow → submit forgot password → wait long enough for the auth session to expire (or manually delete the row) → click the email link → submit a new password.
2. **Expected**: graceful fallback to `/realms/{realm}/overview`. No 500. No crash.

### Task 4.4: Commit PR 4

- [ ] **Step 1: Propose commit (do not run `git commit`)**

- Branch: `1041-forgot-password-redirect-frontend`
- Commit message:

```
fix(front): redirect to original redirect_uri after password reset

When the reset-password response carries a `login_url` (set by the API
when the reset happened mid-OAuth), perform a full-page redirect to it
so the browser lands on the OAuth client's `redirect_uri` instead of
the FerrisKey console.

Falls back to the console for direct console-initiated resets.

Fixes #1041
```

---

## Closing checklist

- [ ] All four PRs reviewed and merged in order.
- [ ] Issue #1041 closed by the final PR's commit footer (`Fixes #1041`).
- [ ] Documentation: no separate docs page is required for this fix — behavior is the user-expected default.
