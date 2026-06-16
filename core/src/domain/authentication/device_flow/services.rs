use std::sync::Arc;

use chrono::{Duration, Utc};
use tracing::warn;

use crate::domain::authentication::device_flow::entities::{
    DeviceAuthSession, DeviceAuthSessionConfig, DeviceAuthStatus, DeviceFlowEventPayload, UserCode,
};
use crate::domain::authentication::device_flow::error::DeviceFlowError;
use crate::domain::authentication::device_flow::ports::{
    DeviceAuthRepository, DeviceFlowService, DeviceTokenIssuer,
};
use crate::domain::authentication::device_flow::value_objects::{
    InitiateDeviceFlowOutput, InitiateDeviceFlowParams, PollDeviceTokenParams,
};
use crate::domain::authentication::entities::JwtToken;
use crate::domain::authentication::value_objects::GenerateTokensForUserInput;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookRepository;

/// Maximum number of attempts to find a collision-free user code.
const MAX_USER_CODE_ATTEMPTS: usize = 5;
const DEFAULT_INTERVAL_SECONDS: i64 = 5;
const DEFAULT_EXPIRES_IN_SECONDS: i64 = 600;

/// Tunable policy for the device flow: how long a session lives and how often
/// a device may poll.
#[derive(Debug, Clone, Copy)]
pub struct DeviceFlowConfig {
    pub interval_seconds: i64,
    pub expires_in_seconds: i64,
}

impl Default for DeviceFlowConfig {
    fn default() -> Self {
        Self {
            interval_seconds: DEFAULT_INTERVAL_SECONDS,
            expires_in_seconds: DEFAULT_EXPIRES_IN_SECONDS,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceFlowServiceImpl<DR, WR, I>
where
    DR: DeviceAuthRepository,
    WR: WebhookRepository,
    I: DeviceTokenIssuer,
{
    pub(crate) device_auth_repository: Arc<DR>,
    pub(crate) webhook_repository: Arc<WR>,
    pub(crate) token_issuer: Arc<I>,
    pub(crate) config: DeviceFlowConfig,
}

impl<DR, WR, I> DeviceFlowServiceImpl<DR, WR, I>
where
    DR: DeviceAuthRepository,
    WR: WebhookRepository,
    I: DeviceTokenIssuer,
{
    pub fn new(
        device_auth_repository: Arc<DR>,
        webhook_repository: Arc<WR>,
        token_issuer: Arc<I>,
        config: DeviceFlowConfig,
    ) -> Self {
        Self {
            device_auth_repository,
            webhook_repository,
            token_issuer,
            config,
        }
    }

    /// Draw user codes until one is free in the repository, bounded by
    /// [`MAX_USER_CODE_ATTEMPTS`].
    async fn generate_unique_user_code(&self) -> Result<UserCode, DeviceFlowError> {
        for _ in 0..MAX_USER_CODE_ATTEMPTS {
            let candidate = UserCode::generate();
            let existing = self
                .device_auth_repository
                .find_by_user_code(candidate.as_str().to_string())
                .await?;

            if existing.is_none() {
                return Ok(candidate);
            }
        }

        Err(DeviceFlowError::UserCodeGenerationExhausted)
    }

    /// Fire a device flow webhook. Best-effort: failures are logged, never
    /// propagated (mirrors the rest of the codebase).
    async fn fire_event(&self, session: &DeviceAuthSession, trigger: WebhookTrigger) {
        let realm_id = session.realm_id;
        let reveal_device_code = matches!(trigger, WebhookTrigger::AuthDeviceFlowInitiated);
        let payload = WebhookPayload::new(
            trigger,
            realm_id.into(),
            Some(DeviceFlowEventPayload::new(session, reveal_device_code)),
        );

        if let Err(err) = self.webhook_repository.notify(realm_id, payload).await {
            warn!("failed to fire device flow webhook: {err}");
        }
    }
}

impl<DR, WR, I> DeviceFlowService for DeviceFlowServiceImpl<DR, WR, I>
where
    DR: DeviceAuthRepository,
    WR: WebhookRepository,
    I: DeviceTokenIssuer,
{
    async fn initiate(
        &self,
        params: InitiateDeviceFlowParams,
    ) -> Result<InitiateDeviceFlowOutput, DeviceFlowError> {
        if !params.oauth_device_code_grant_enabled {
            return Err(DeviceFlowError::UnauthorizedClient);
        }

        let user_code = self.generate_unique_user_code().await?;

        let mut session = DeviceAuthSession::new(DeviceAuthSessionConfig {
            realm_id: params.realm_id,
            client_id: params.client_id,
            scope: params.scope,
            interval: self.config.interval_seconds,
            expires_in: self.config.expires_in_seconds,
        });
        session.user_code = user_code;

        let session = self.device_auth_repository.create(&session).await?;

        self.fire_event(&session, WebhookTrigger::AuthDeviceFlowInitiated)
            .await;

        let verification_uri_complete = format!(
            "{}?user_code={}",
            params.verification_uri,
            session.user_code.as_str()
        );

        Ok(InitiateDeviceFlowOutput {
            device_code: session.device_code.to_string(),
            user_code: session.user_code.into_inner(),
            verification_uri: params.verification_uri,
            verification_uri_complete,
            expires_in: self.config.expires_in_seconds,
            interval: self.config.interval_seconds,
        })
    }

    async fn verify_user_code(
        &self,
        user_code: String,
        user_id: uuid::Uuid,
    ) -> Result<(), DeviceFlowError> {
        let session = self
            .device_auth_repository
            .find_by_user_code(user_code)
            .await?
            .ok_or(DeviceFlowError::InvalidUserCode)?;

        match session.status {
            DeviceAuthStatus::Denied => return Err(DeviceFlowError::AccessDenied),
            DeviceAuthStatus::Expired => return Err(DeviceFlowError::ExpiredToken),
            // Idempotent: re-approving an already approved session is a no-op.
            DeviceAuthStatus::Approved => return Ok(()),
            DeviceAuthStatus::Pending => {}
        }

        if session.is_expired() {
            return Err(DeviceFlowError::ExpiredToken);
        }

        self.device_auth_repository
            .update_status(
                session.device_code,
                DeviceAuthStatus::Approved,
                Some(user_id),
            )
            .await?;

        Ok(())
    }

    async fn deny(&self, user_code: String, user_id: uuid::Uuid) -> Result<(), DeviceFlowError> {
        let session = self
            .device_auth_repository
            .find_by_user_code(user_code)
            .await?
            .ok_or(DeviceFlowError::InvalidUserCode)?;

        let session = self
            .device_auth_repository
            .update_status(session.device_code, DeviceAuthStatus::Denied, Some(user_id))
            .await?;

        self.fire_event(&session, WebhookTrigger::AuthDeviceFlowDenied)
            .await;

        Ok(())
    }

    async fn poll(&self, params: PollDeviceTokenParams) -> Result<JwtToken, DeviceFlowError> {
        let session = self
            .device_auth_repository
            .find_by_device_code(params.device_code)
            .await?
            .ok_or(DeviceFlowError::InvalidDeviceCode)?;

        // The device_code is bound to the client that initiated the flow.
        if session.client_id != params.client_id {
            return Err(DeviceFlowError::InvalidClient);
        }

        match session.status {
            DeviceAuthStatus::Approved => {
                let user_id = session.user_id.ok_or_else(|| {
                    DeviceFlowError::TokenIssuance(
                        "approved session has no associated user".to_string(),
                    )
                })?;

                self.token_issuer
                    .issue_tokens_for_user(GenerateTokensForUserInput {
                        user_id,
                        realm_id: session.realm_id.into(),
                        base_url: params.base_url,
                        client_id: Some(session.client_id),
                    })
                    .await
                    .map_err(|err| DeviceFlowError::TokenIssuance(err.to_string()))
            }
            DeviceAuthStatus::Denied => Err(DeviceFlowError::AccessDenied),
            DeviceAuthStatus::Expired => Err(DeviceFlowError::ExpiredToken),
            DeviceAuthStatus::Pending => {
                // Time-based expiry takes precedence: terminate the session.
                if session.is_expired() {
                    let session = self
                        .device_auth_repository
                        .update_status(session.device_code, DeviceAuthStatus::Expired, None)
                        .await?;

                    self.fire_event(&session, WebhookTrigger::AuthDeviceFlowExpired)
                        .await;

                    return Err(DeviceFlowError::ExpiredToken);
                }

                // Anti-bruteforce: reject polls arriving faster than `interval`.
                if let Some(last_polled_at) = session.last_polled_at {
                    let next_allowed = last_polled_at + Duration::seconds(session.interval);
                    if Utc::now() < next_allowed {
                        return Err(DeviceFlowError::SlowDown);
                    }
                }

                self.device_auth_repository
                    .mark_polled(session.device_code)
                    .await?;

                Err(DeviceFlowError::AuthorizationPending)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::authentication::device_flow::entities::{
        DeviceAuthSession, DeviceAuthSessionConfig, DeviceAuthStatus, DeviceFlowEventPayload,
    };
    use crate::domain::authentication::device_flow::ports::{
        MockDeviceAuthRepository, MockDeviceTokenIssuer,
    };
    use crate::domain::authentication::device_flow::value_objects::{
        InitiateDeviceFlowParams, PollDeviceTokenParams,
    };
    use crate::domain::common::entities::app_errors::CoreError;
    use crate::domain::realm::entities::RealmId;
    use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
    use crate::domain::webhook::ports::MockWebhookRepository;

    fn pending_session(client_id: Uuid) -> DeviceAuthSession {
        DeviceAuthSession::new(DeviceAuthSessionConfig {
            realm_id: RealmId::from(Uuid::new_v4()),
            client_id,
            scope: None,
            interval: 5,
            expires_in: 600,
        })
    }

    fn dummy_token() -> JwtToken {
        JwtToken::new(
            "access".to_string(),
            "Bearer".to_string(),
            "refresh".to_string(),
            300,
            3600,
            None,
            None,
        )
    }

    fn build(
        device_repo: MockDeviceAuthRepository,
        webhook_repo: MockWebhookRepository,
        issuer: MockDeviceTokenIssuer,
    ) -> DeviceFlowServiceImpl<MockDeviceAuthRepository, MockWebhookRepository, MockDeviceTokenIssuer>
    {
        DeviceFlowServiceImpl::new(
            Arc::new(device_repo),
            Arc::new(webhook_repo),
            Arc::new(issuer),
            DeviceFlowConfig::default(),
        )
    }

    #[tokio::test]
    async fn initiate_generates_session_and_fires_webhook() {
        let mut device_repo = MockDeviceAuthRepository::new();
        let mut webhook_repo = MockWebhookRepository::new();
        let issuer = MockDeviceTokenIssuer::new();

        device_repo
            .expect_find_by_user_code()
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));
        device_repo
            .expect_create()
            .times(1)
            .returning(|s: &DeviceAuthSession| {
                let s = s.clone();
                Box::pin(async move { Ok(s) })
            });
        webhook_repo
            .expect_notify::<DeviceFlowEventPayload>()
            .times(1)
            .returning(|_, _: WebhookPayload<DeviceFlowEventPayload>| {
                Box::pin(async move { Ok(()) })
            });

        let service = build(device_repo, webhook_repo, issuer);
        let out = service
            .initiate(InitiateDeviceFlowParams {
                realm_id: RealmId::from(Uuid::new_v4()),
                client_id: Uuid::new_v4(),
                scope: Some("openid".to_string()),
                oauth_device_code_grant_enabled: true,
                verification_uri: "https://auth.example.com/realms/master/device".to_string(),
            })
            .await
            .expect("initiate should succeed");

        assert_eq!(out.interval, DEFAULT_INTERVAL_SECONDS);
        assert_eq!(out.expires_in, DEFAULT_EXPIRES_IN_SECONDS);
        assert!(out.verification_uri_complete.contains("user_code="));
        assert!(out.verification_uri_complete.contains(&out.user_code));
        // device_code is a UUID rendered as a string.
        assert!(Uuid::parse_str(&out.device_code).is_ok());
    }

    #[tokio::test]
    async fn initiate_retries_on_user_code_collision() {
        let mut device_repo = MockDeviceAuthRepository::new();
        let mut webhook_repo = MockWebhookRepository::new();
        let issuer = MockDeviceTokenIssuer::new();

        // First candidate collides, second is free.
        let mut calls = 0;
        device_repo
            .expect_find_by_user_code()
            .times(2)
            .returning(move |_| {
                calls += 1;
                let collide = calls == 1;
                Box::pin(async move {
                    if collide {
                        Ok(Some(pending_session(Uuid::new_v4())))
                    } else {
                        Ok(None)
                    }
                })
            });
        device_repo
            .expect_create()
            .times(1)
            .returning(|s: &DeviceAuthSession| {
                let s = s.clone();
                Box::pin(async move { Ok(s) })
            });
        webhook_repo
            .expect_notify::<DeviceFlowEventPayload>()
            .returning(|_, _: WebhookPayload<DeviceFlowEventPayload>| {
                Box::pin(async move { Ok(()) })
            });

        let service = build(device_repo, webhook_repo, issuer);
        let res = service
            .initiate(InitiateDeviceFlowParams {
                realm_id: RealmId::from(Uuid::new_v4()),
                client_id: Uuid::new_v4(),
                scope: None,
                oauth_device_code_grant_enabled: true,
                verification_uri: "https://auth.example.com/device".to_string(),
            })
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn initiate_returns_unauthorized_when_grant_disabled_on_client() {
        // Repo / webhook should never be touched: the gate fires first.
        let device_repo = MockDeviceAuthRepository::new();
        let webhook_repo = MockWebhookRepository::new();
        let issuer = MockDeviceTokenIssuer::new();

        let service = build(device_repo, webhook_repo, issuer);
        let err = service
            .initiate(InitiateDeviceFlowParams {
                realm_id: RealmId::from(Uuid::new_v4()),
                client_id: Uuid::new_v4(),
                scope: None,
                oauth_device_code_grant_enabled: false,
                verification_uri: "https://auth.example.com/device".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::UnauthorizedClient));
    }

    #[tokio::test]
    async fn verify_user_code_marks_session_approved() {
        let user_id = Uuid::new_v4();
        let session = pending_session(Uuid::new_v4());
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        let webhook_repo = MockWebhookRepository::new();
        let issuer = MockDeviceTokenIssuer::new();

        device_repo
            .expect_find_by_user_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        device_repo
            .expect_update_status()
            .withf(move |dc, status, uid| {
                *dc == device_code && *status == DeviceAuthStatus::Approved && *uid == Some(user_id)
            })
            .times(1)
            .return_once(move |_, _, _| {
                let mut s = pending_session(Uuid::new_v4());
                s.status = DeviceAuthStatus::Approved;
                s.user_id = Some(user_id);
                Box::pin(async move { Ok(s) })
            });

        let service = build(device_repo, webhook_repo, issuer);
        service
            .verify_user_code("BCDF-GHJK".to_string(), user_id)
            .await
            .expect("verify should succeed");
    }

    #[tokio::test]
    async fn verify_unknown_user_code_is_invalid() {
        let mut device_repo = MockDeviceAuthRepository::new();
        device_repo
            .expect_find_by_user_code()
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));

        let service = build(
            device_repo,
            MockWebhookRepository::new(),
            MockDeviceTokenIssuer::new(),
        );
        let err = service
            .verify_user_code("ZZZZ-ZZZZ".to_string(), Uuid::new_v4())
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::InvalidUserCode));
    }

    #[tokio::test]
    async fn deny_marks_denied_and_fires_webhook() {
        let user_id = Uuid::new_v4();
        let session = pending_session(Uuid::new_v4());
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        let mut webhook_repo = MockWebhookRepository::new();
        let issuer = MockDeviceTokenIssuer::new();

        device_repo
            .expect_find_by_user_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        device_repo
            .expect_update_status()
            .withf(move |dc, status, _| *dc == device_code && *status == DeviceAuthStatus::Denied)
            .times(1)
            .return_once(move |_, _, _| {
                let mut s = pending_session(Uuid::new_v4());
                s.status = DeviceAuthStatus::Denied;
                Box::pin(async move { Ok(s) })
            });
        webhook_repo
            .expect_notify::<DeviceFlowEventPayload>()
            .times(1)
            .returning(|_, _: WebhookPayload<DeviceFlowEventPayload>| {
                Box::pin(async move { Ok(()) })
            });

        let service = build(device_repo, webhook_repo, issuer);
        service
            .deny("BCDF-GHJK".to_string(), user_id)
            .await
            .expect("deny should succeed");
    }

    #[tokio::test]
    async fn poll_returns_token_when_approved() {
        let user_id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let mut session = pending_session(client_id);
        session.status = DeviceAuthStatus::Approved;
        session.user_id = Some(user_id);
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        let mut issuer = MockDeviceTokenIssuer::new();

        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        issuer
            .expect_issue_tokens_for_user()
            .withf(move |input| input.user_id == user_id && input.client_id == Some(client_id))
            .times(1)
            .return_once(|_| Box::pin(async move { Ok(dummy_token()) }));

        let service = build(device_repo, MockWebhookRepository::new(), issuer);
        let token = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id,
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .expect("poll should issue a token");

        assert_eq!(token, dummy_token());
    }

    #[tokio::test]
    async fn poll_pending_returns_authorization_pending_and_records_poll() {
        let client_id = Uuid::new_v4();
        let session = pending_session(client_id); // last_polled_at = None
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        device_repo
            .expect_mark_polled()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(()) }));

        let service = build(
            device_repo,
            MockWebhookRepository::new(),
            MockDeviceTokenIssuer::new(),
        );
        let err = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id,
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::AuthorizationPending));
    }

    #[tokio::test]
    async fn poll_too_fast_returns_slow_down() {
        let client_id = Uuid::new_v4();
        let mut session = pending_session(client_id);
        // Just polled — interval has not yet elapsed.
        session.last_polled_at = Some(Utc::now());
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        // mark_polled must NOT be called on a slow_down.

        let service = build(
            device_repo,
            MockWebhookRepository::new(),
            MockDeviceTokenIssuer::new(),
        );
        let err = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id,
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::SlowDown));
    }

    #[tokio::test]
    async fn poll_after_interval_is_accepted_again() {
        let client_id = Uuid::new_v4();
        let mut session = pending_session(client_id);
        // Last poll was long enough ago (interval = 5s).
        session.last_polled_at = Some(Utc::now() - Duration::seconds(30));
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        device_repo
            .expect_mark_polled()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(()) }));

        let service = build(
            device_repo,
            MockWebhookRepository::new(),
            MockDeviceTokenIssuer::new(),
        );
        let err = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id,
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::AuthorizationPending));
    }

    #[tokio::test]
    async fn poll_expired_marks_expired_and_fires_webhook() {
        let client_id = Uuid::new_v4();
        let mut session = pending_session(client_id);
        session.expires_at = Utc::now() - Duration::seconds(10);
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        let mut webhook_repo = MockWebhookRepository::new();

        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        device_repo
            .expect_update_status()
            .withf(move |dc, status, _| *dc == device_code && *status == DeviceAuthStatus::Expired)
            .times(1)
            .return_once(move |_, _, _| {
                let mut s = pending_session(client_id);
                s.status = DeviceAuthStatus::Expired;
                Box::pin(async move { Ok(s) })
            });
        webhook_repo
            .expect_notify::<DeviceFlowEventPayload>()
            .times(1)
            .returning(|_, _: WebhookPayload<DeviceFlowEventPayload>| {
                Box::pin(async move { Ok(()) })
            });

        let service = build(device_repo, webhook_repo, MockDeviceTokenIssuer::new());
        let err = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id,
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::ExpiredToken));
    }

    #[tokio::test]
    async fn poll_denied_returns_access_denied() {
        let client_id = Uuid::new_v4();
        let mut session = pending_session(client_id);
        session.status = DeviceAuthStatus::Denied;
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));

        let service = build(
            device_repo,
            MockWebhookRepository::new(),
            MockDeviceTokenIssuer::new(),
        );
        let err = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id,
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::AccessDenied));
    }

    #[tokio::test]
    async fn poll_unknown_device_code_is_invalid() {
        let mut device_repo = MockDeviceAuthRepository::new();
        device_repo
            .expect_find_by_device_code()
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));

        let service = build(
            device_repo,
            MockWebhookRepository::new(),
            MockDeviceTokenIssuer::new(),
        );
        let err = service
            .poll(PollDeviceTokenParams {
                device_code: Uuid::new_v4(),
                client_id: Uuid::new_v4(),
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::InvalidDeviceCode));
    }

    #[tokio::test]
    async fn poll_with_mismatched_client_is_rejected() {
        let session = pending_session(Uuid::new_v4());
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));

        let service = build(
            device_repo,
            MockWebhookRepository::new(),
            MockDeviceTokenIssuer::new(),
        );
        let err = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id: Uuid::new_v4(), // different client
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::InvalidClient));
    }

    #[tokio::test]
    async fn poll_token_issuance_failure_is_surfaced() {
        let user_id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let mut session = pending_session(client_id);
        session.status = DeviceAuthStatus::Approved;
        session.user_id = Some(user_id);
        let device_code = session.device_code;

        let mut device_repo = MockDeviceAuthRepository::new();
        let mut issuer = MockDeviceTokenIssuer::new();

        device_repo
            .expect_find_by_device_code()
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        issuer
            .expect_issue_tokens_for_user()
            .times(1)
            .return_once(|_| Box::pin(async move { Err(CoreError::InvalidClient) }));

        let service = build(device_repo, MockWebhookRepository::new(), issuer);
        let err = service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id,
                base_url: "https://auth.example.com".to_string(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, DeviceFlowError::TokenIssuance(_)));
    }
}
