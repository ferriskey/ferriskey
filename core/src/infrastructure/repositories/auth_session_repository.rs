use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    prelude::Expr,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::authentication::{
    entities::{AuthSession, AuthenticationError, WebAuthnChallenge},
    ports::AuthSessionRepository,
};

impl From<crate::entity::auth_sessions::Model> for AuthSession {
    fn from(model: crate::entity::auth_sessions::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = Utc.from_utc_datetime(&model.expires_at);
        let webauthn_challenge_issued_at = model
            .webauthn_challenge_issued_at
            .map(|ref dt| Utc.from_utc_datetime(dt));

        let webauthn_challenge = if let Some(webauthn_challenge) = model.webauthn_challenge {
            serde_json::from_value(webauthn_challenge).ok()
        } else {
            None
        };

        AuthSession {
            id: model.id,
            realm_id: model.realm_id.into(),
            client_id: model.client_id,
            redirect_uri: model.redirect_uri,
            response_type: model.response_type,
            scope: model.scope,
            state: model.state,
            nonce: model.nonce,
            code: model.code,
            authenticated: model.authenticated,
            user_id: model.user_id,
            created_at,
            expires_at,
            webauthn_challenge,
            webauthn_challenge_issued_at,
            compass_flow_id: model.compass_flow_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PostgresAuthSessionRepository {
    pub db: DatabaseConnection,
}

impl PostgresAuthSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl AuthSessionRepository for PostgresAuthSessionRepository {
    async fn create(&self, session: &AuthSession) -> Result<AuthSession, AuthenticationError> {
        let model = crate::entity::auth_sessions::ActiveModel {
            id: Set(session.id),
            realm_id: Set(session.realm_id.into()),
            client_id: Set(session.client_id),
            redirect_uri: Set(session.redirect_uri.clone()),
            response_type: Set(session.response_type.clone()),
            scope: Set(session.scope.clone()),
            state: Set(session.state.clone()),
            nonce: Set(session.nonce.clone()),
            code: Set(session.code.clone()),
            authenticated: Set(session.authenticated),
            user_id: Set(session.user_id),
            created_at: Set(session.created_at.naive_utc()),
            expires_at: Set(session.expires_at.naive_utc()),
            webauthn_challenge: Set(None),
            webauthn_challenge_issued_at: Set(None),
            compass_flow_id: Set(session.compass_flow_id),
        };

        let t = model
            .insert(&self.db)
            .await
            .map_err(|e| {
                error!("Error creating session: {:?}", e);
                AuthenticationError::InternalServerError
            })?
            .into();

        Ok(t)
    }

    async fn get_by_session_code(
        &self,
        session_code: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::find()
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error getting session: {:?}", e);
                AuthenticationError::NotFound
            })?;

        let session = session.ok_or(AuthenticationError::NotFound)?.into();

        Ok(session)
    }

    async fn get_by_code(&self, code: String) -> Result<Option<AuthSession>, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::find()
            .filter(crate::entity::auth_sessions::Column::Code.eq(code))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error getting session: {:?}", e);
                AuthenticationError::NotFound
            })?;

        let session: Option<AuthSession> = session.map(|s| s.into());

        Ok(session)
    }

    async fn update_code_and_user_id(
        &self,
        session_code: Uuid,
        code: String,
        user_id: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::update_many()
            .col_expr(
                crate::entity::auth_sessions::Column::Code,
                Expr::value(code),
            )
            .col_expr(
                crate::entity::auth_sessions::Column::UserId,
                Expr::value(user_id),
            )
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating session: {:?}", e);
                AuthenticationError::Invalid
            })?
            .into_iter()
            .next()
            .ok_or(AuthenticationError::NotFound)?
            .into();

        Ok(session)
    }

    async fn save_webauthn_challenge(
        &self,
        session_code: Uuid,
        challenge: WebAuthnChallenge,
    ) -> Result<AuthSession, AuthenticationError> {
        let challenge = serde_json::to_value(&challenge).map_err(|e| {
            error!("Error serializing WebAuthnChallenge: {e:?}");
            AuthenticationError::InternalServerError
        })?;

        let session = crate::entity::auth_sessions::Entity::update_many()
            .col_expr(
                crate::entity::auth_sessions::Column::WebauthnChallenge,
                Expr::value(challenge),
            )
            .col_expr(
                crate::entity::auth_sessions::Column::WebauthnChallengeIssuedAt,
                Expr::value(Utc::now()),
            )
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating session: {:?}", e);
                AuthenticationError::Invalid
            })?
            .into_iter()
            .next()
            .ok_or(AuthenticationError::NotFound)?
            .into();

        Ok(session)
    }

    async fn take_webauthn_challenge(
        &self,
        session_code: Uuid,
    ) -> Result<Option<WebAuthnChallenge>, AuthenticationError> {
        // apparently this can be done in a single sql query with CTEs
        // sea_orm doesn't support them well so two queries it will be

        let auth_session_model = crate::entity::auth_sessions::Entity::find()
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error fetching session: {e:?}");
                AuthenticationError::InternalServerError
            })?
            .ok_or(AuthenticationError::NotFound)?;

        if let Some(challenge) = auth_session_model.webauthn_challenge.clone() {
            let mut active: crate::entity::auth_sessions::ActiveModel = auth_session_model.into();

            active.webauthn_challenge = Set(None);
            active.update(&self.db).await.map_err(|e| {
                error!("Error updating session: {e:?}");
                AuthenticationError::InternalServerError
            })?;

            let challenge = serde_json::from_value(challenge).map_err(|e| {
                error!("Error deserializing webauthn_challenge: {e:?}");
                AuthenticationError::InternalServerError
            })?;

            Ok(Some(challenge))
        } else {
            Ok(None)
        }
    }

    async fn update_user_id(
        &self,
        session_code: Uuid,
        user_id: Uuid,
    ) -> Result<AuthSession, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::update_many()
            .col_expr(
                crate::entity::auth_sessions::Column::UserId,
                Expr::value(user_id),
            )
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating session user_id: {:?}", e);
                AuthenticationError::Invalid
            })?
            .into_iter()
            .next()
            .ok_or(AuthenticationError::NotFound)?
            .into();

        Ok(session)
    }

    /// Stamps a fresh authorization code onto an existing auth session and resets
    /// the `authenticated` flag to `false`.
    ///
    /// # RFC 6749 semantics
    ///
    /// The OAuth 2.0 Authorization Code flow (RFC 6749 §4.1) separates two
    /// distinct moments:
    ///
    /// * **§4.1.2 – Authorization Response**: the authorization server issues a
    ///   short-lived, single-use authorization code and redirects the client.
    ///   At this point the code has *not* yet been exchanged.
    ///
    /// * **§4.1.3 – Access Token Request**: the client presents the code to the
    ///   token endpoint in exchange for tokens.  Only after a successful exchange
    ///   should the code be considered "used".
    ///
    /// RFC 6749 §10.5 further requires that the authorization server ensure codes
    /// "cannot be used more than once".  Ferriskey implements this invariant with
    /// the `authenticated` column: `false` means the code is ready to be
    /// exchanged; `true` means it has already been exchanged (or revoked) and
    /// must be rejected by the token endpoint.
    ///
    /// This function is called during the brokered SSO callback (RFC 8414 / OIDC
    /// Core §3.1) when Ferriskey reuses an existing `auth_session` that already
    /// has a user linked.  A fresh code is generated for the downstream client,
    /// therefore `authenticated` **must** be reset to `false` so that the token
    /// endpoint will accept the exchange.  Leaving `authenticated = true` here
    /// causes `authorization_code()` to reject the brand-new code immediately
    /// with `CoreError::InvalidToken`, breaking the entire brokered login flow.
    async fn update_code(
        &self,
        session_code: Uuid,
        code: String,
    ) -> Result<AuthSession, AuthenticationError> {
        let session = crate::entity::auth_sessions::Entity::update_many()
            .col_expr(
                crate::entity::auth_sessions::Column::Code,
                Expr::value(code),
            )
            // Reset to false: this code has not yet been exchanged for tokens.
            // See RFC 6749 §4.1.2 (code issuance) vs §4.1.3 (code exchange) and
            // the `authenticated` invariant described in the doc-comment above.
            .col_expr(
                crate::entity::auth_sessions::Column::Authenticated,
                Expr::value(false),
            )
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating session code: {:?}", e);
                AuthenticationError::Invalid
            })?
            .into_iter()
            .next()
            .ok_or(AuthenticationError::NotFound)?
            .into();

        Ok(session)
    }

    async fn update_compass_flow_id(
        &self,
        session_code: Uuid,
        compass_flow_id: Uuid,
    ) -> Result<(), AuthenticationError> {
        crate::entity::auth_sessions::Entity::update_many()
            .col_expr(
                crate::entity::auth_sessions::Column::CompassFlowId,
                Expr::value(compass_flow_id),
            )
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating session compass_flow_id: {:?}", e);
                AuthenticationError::Invalid
            })?;

        Ok(())
    }

    async fn update_authenticated(
        &self,
        session_code: Uuid,
        authenticated: bool,
    ) -> Result<(), AuthenticationError> {
        crate::entity::auth_sessions::Entity::update_many()
            .col_expr(
                crate::entity::auth_sessions::Column::Authenticated,
                Expr::value(authenticated),
            )
            .filter(crate::entity::auth_sessions::Column::Id.eq(session_code))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating session authenticated: {:?}", e);
                AuthenticationError::Invalid
            })?;

        Ok(())
    }
}

/// Integration tests for `PostgresAuthSessionRepository`.
///
/// These tests require a running PostgreSQL instance and are skipped during
/// regular `cargo test` runs.  Execute them explicitly with:
///
/// ```text
/// cargo test -p ferriskey-core -- --ignored
/// ```
///
/// Environment variables (defaults shown):
/// ```text
/// DATABASE_URL = postgres://ferriskey:ferriskey@localhost:5432/ferriskey
/// ```
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use uuid::Uuid;

    use crate::domain::authentication::{
        entities::{AuthSession, AuthSessionParams},
        ports::AuthSessionRepository as _,
    };
    use crate::domain::realm::entities::RealmId;

    async fn setup() -> (PostgresAuthSessionRepository, Uuid, Uuid) {
        let base_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string());

        let schema = format!("auth_session_test_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{}""#, schema)))
            .await
            .expect("create test schema");

        let schema_url = format!("{}?options=-c search_path={}", base_url, schema);
        let schema_pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");
        sqlx::migrate!("migrations")
            .run(&schema_pool)
            .await
            .expect("run migrations");

        let realm_id = Uuid::new_v4();
        let client_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(realm_id)
        .bind(format!("test-realm-{realm_id}"))
        .execute(&schema_pool)
        .await
        .expect("insert test realm");

        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, \
             public_client, service_account_enabled, client_type, created_at, updated_at) \
             VALUES ($1, $2, 'test-client', 'test-client', TRUE, 'openid-connect', \
             TRUE, FALSE, 'public', NOW(), NOW())",
        )
        .bind(client_id)
        .bind(realm_id)
        .execute(&schema_pool)
        .await
        .expect("insert test client");

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");
        (PostgresAuthSessionRepository::new(db), realm_id, client_id)
    }

    fn make_session(realm_id: Uuid, client_id: Uuid, authenticated: bool) -> AuthSession {
        AuthSession::new(AuthSessionParams {
            realm_id: RealmId::new(realm_id),
            client_id,
            redirect_uri: "http://localhost/callback".into(),
            response_type: "code".into(),
            scope: "openid".into(),
            state: None,
            nonce: None,
            user_id: None,
            code: None,
            authenticated,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
        })
    }

    /// A freshly-issued authorization code must have `authenticated = false`.
    ///
    /// RFC 6749 §4.1.2 defines code issuance (step D of the flow); the code has
    /// not been exchanged yet so it must not be pre-marked as used.
    ///
    /// Regression: `update_code` was previously setting `authenticated = true`,
    /// causing the token endpoint to reject the brand-new code immediately with
    /// `CoreError::InvalidToken` — breaking every brokered SSO login where
    /// Ferriskey reused an existing `auth_session` row.
    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn update_code_resets_authenticated_to_false() {
        let (repo, realm_id, client_id) = setup().await;

        // Start with an already-consumed session (authenticated = true).
        let session = make_session(realm_id, client_id, true);
        let created = repo.create(&session).await.expect("create session");
        assert!(created.authenticated, "pre-condition");

        let updated = repo
            .update_code(created.id, "fresh-auth-code".into())
            .await
            .expect("update_code");

        assert!(
            !updated.authenticated,
            "update_code must reset authenticated to false (RFC 6749 §4.1.2)"
        );
        assert_eq!(updated.code, Some("fresh-auth-code".into()));
    }

    /// `update_code` must reset `authenticated` on every call so a session can
    /// be reused across multiple brokered SSO login cycles.
    ///
    /// RFC 6749 §10.5 requires single-use per code, not per session: after a
    /// successful exchange (`authenticated = true`) a subsequent `update_code`
    /// must produce a fresh, exchangeable code.
    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn update_code_enables_reuse_across_login_cycles() {
        let (repo, realm_id, client_id) = setup().await;

        let session = make_session(realm_id, client_id, false);
        let created = repo.create(&session).await.expect("create session");

        // First SSO callback — issues code for first login.
        let after_first = repo
            .update_code(created.id, "code-round-1".into())
            .await
            .expect("first update_code");
        assert!(!after_first.authenticated, "ready for first exchange");

        // Token exchange marks the code as used (RFC 6749 §4.1.3).
        repo.update_authenticated(created.id, true)
            .await
            .expect("mark used");

        // Second SSO callback — reuses the session for a new login cycle.
        let after_second = repo
            .update_code(created.id, "code-round-2".into())
            .await
            .expect("second update_code");

        assert!(
            !after_second.authenticated,
            "update_code must reset authenticated to false on reuse (RFC 6749 §10.5)"
        );
        assert_eq!(after_second.code, Some("code-round-2".into()));
    }
}
