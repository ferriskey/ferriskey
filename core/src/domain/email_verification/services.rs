use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use crate::domain::common::email::EmailPort;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::email_template::entities::interpolate_variables;
use crate::domain::email_template::ports::{EmailTemplateRepository, TemplateRenderer};
use crate::domain::realm::ports::{RealmRepository, SmtpConfigRepository};
use crate::domain::user::entities::RequiredAction;
use crate::domain::user::ports::{UserRepository, UserRequiredActionRepository};

use super::ports::*;

fn generate_token_hash(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug)]
pub struct EmailVerificationServiceImpl<EVRT, UR, RR, URA, ES, SC, ETR, TR>
where
    EVRT: EmailVerificationTokenRepository,
    UR: UserRepository,
    RR: RealmRepository,
    URA: UserRequiredActionRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
{
    pub(crate) email_verification_token_repository: Arc<EVRT>,
    pub(crate) user_repository: Arc<UR>,
    pub(crate) realm_repository: Arc<RR>,
    pub(crate) user_required_action_repository: Arc<URA>,
    pub(crate) email_port: Arc<ES>,
    pub(crate) smtp_config_repository: Arc<SC>,
    pub(crate) email_template_repository: Arc<ETR>,
    pub(crate) template_renderer: Arc<TR>,
}

impl<EVRT, UR, RR, URA, ES, SC, ETR, TR> Clone
    for EmailVerificationServiceImpl<EVRT, UR, RR, URA, ES, SC, ETR, TR>
where
    EVRT: EmailVerificationTokenRepository,
    UR: UserRepository,
    RR: RealmRepository,
    URA: UserRequiredActionRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
{
    fn clone(&self) -> Self {
        Self {
            email_verification_token_repository: self.email_verification_token_repository.clone(),
            user_repository: self.user_repository.clone(),
            realm_repository: self.realm_repository.clone(),
            user_required_action_repository: self.user_required_action_repository.clone(),
            email_port: self.email_port.clone(),
            smtp_config_repository: self.smtp_config_repository.clone(),
            email_template_repository: self.email_template_repository.clone(),
            template_renderer: self.template_renderer.clone(),
        }
    }
}

impl<EVRT, UR, RR, URA, ES, SC, ETR, TR>
    EmailVerificationServiceImpl<EVRT, UR, RR, URA, ES, SC, ETR, TR>
where
    EVRT: EmailVerificationTokenRepository,
    UR: UserRepository,
    RR: RealmRepository,
    URA: UserRequiredActionRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        email_verification_token_repository: Arc<EVRT>,
        user_repository: Arc<UR>,
        realm_repository: Arc<RR>,
        user_required_action_repository: Arc<URA>,
        email_port: Arc<ES>,
        smtp_config_repository: Arc<SC>,
        email_template_repository: Arc<ETR>,
        template_renderer: Arc<TR>,
    ) -> Self {
        Self {
            email_verification_token_repository,
            user_repository,
            realm_repository,
            user_required_action_repository,
            email_port,
            smtp_config_repository,
            email_template_repository,
            template_renderer,
        }
    }

    async fn render_email_template(
        &self,
        template_id: Uuid,
        user: &crate::domain::user::entities::User,
        extra_vars: &[(&str, &str)],
    ) -> Result<String, CoreError> {
        let template = self
            .email_template_repository
            .get_by_id(template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        let html = self.template_renderer.render_to_html(&template.mjml)?;

        let mut variables = HashMap::new();
        variables.insert("user.first_name".to_string(), user.firstname.clone());
        variables.insert("user.last_name".to_string(), user.lastname.clone());
        variables.insert("user.email".to_string(), user.email.clone());
        for (key, value) in extra_vars {
            variables.insert(key.to_string(), value.to_string());
        }

        Ok(interpolate_variables(&html, &variables))
    }
}

impl<EVRT, UR, RR, URA, ES, SC, ETR, TR> EmailVerificationService
    for EmailVerificationServiceImpl<EVRT, UR, RR, URA, ES, SC, ETR, TR>
where
    EVRT: EmailVerificationTokenRepository,
    UR: UserRepository,
    RR: RealmRepository,
    URA: UserRequiredActionRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
{
    async fn send_verification_email(
        &self,
        user_id: Uuid,
        realm_name: String,
        base_url: String,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let user = self.user_repository.get_by_id(user_id).await?;

        // Generate token (UUID v4 as raw token, SHA-256 hash for storage)
        let raw_token = Uuid::new_v4().to_string();
        let token_hash = generate_token_hash(&raw_token);
        let ttl_hours = realm
            .settings
            .as_ref()
            .map(|s| s.email_verification_ttl_hours)
            .unwrap_or(24);
        let expires_at = Utc::now() + Duration::hours(ttl_hours);

        self.email_verification_token_repository
            .create(CreateEmailVerificationTokenInput {
                user_id,
                realm_id: realm.id.into(),
                token_hash,
                expires_at,
            })
            .await?;

        // Build verification link (points to frontend page, which calls the API)
        let verification_link = format!(
            "{}/realms/{}/authentication/verify-email?token={}",
            base_url, realm_name, raw_token
        );

        // Get template_id and smtp_config
        let template_id = realm
            .settings
            .as_ref()
            .and_then(|s| s.email_verification_template_id);

        match (
            template_id,
            self.smtp_config_repository.get_by_realm_id(realm.id).await,
        ) {
            (Some(tid), Ok(Some(smtp_config))) => {
                let expiration_label = if ttl_hours == 1 {
                    "1 hour".to_string()
                } else {
                    format!("{} hours", ttl_hours)
                };

                let body = format!(
                    "Click the link below to verify your email address:\n{}\n\nThis link expires in {}.\n\nIf you did not register for this account, please ignore this email.",
                    verification_link, expiration_label
                );

                let html_body = self
                    .render_email_template(
                        tid,
                        &user,
                        &[
                            ("verification_link", verification_link.as_str()),
                            ("expiration", expiration_label.as_str()),
                        ],
                    )
                    .await
                    .ok();

                match self
                    .email_port
                    .send_email(
                        &smtp_config,
                        &user.email,
                        "Verify your email address",
                        &body,
                        html_body,
                    )
                    .await
                {
                    Ok(()) => {
                        tracing::info!(
                            "Verification email sent to {} for user {}",
                            user.email,
                            user.id
                        );
                    }
                    Err(e) => {
                        warn!("Failed to send verification email: {}", e);
                    }
                }
            }
            (None, _) => {
                warn!(
                    "No email verification template configured for realm {}",
                    realm.name
                );
            }
            (_, _) => {
                warn!("SMTP not configured for realm {}", realm.name);
            }
        }

        Ok(())
    }

    async fn verify_email(&self, token: String) -> Result<VerifyEmailResult, CoreError> {
        let token_hash = generate_token_hash(&token);

        let token_record = self
            .email_verification_token_repository
            .find_valid_by_hash(&token_hash)
            .await?
            .ok_or(CoreError::InvalidOrExpiredToken)?;

        if !token_record.is_valid() {
            return Err(CoreError::InvalidOrExpiredToken);
        }

        // Mark token as used
        self.email_verification_token_repository
            .mark_used(token_record.id)
            .await?;

        // Get user and update email_verified = true
        let user = self.user_repository.get_by_id(token_record.user_id).await?;

        self.user_repository
            .update_user(
                token_record.user_id,
                crate::domain::user::value_objects::UpdateUserRequest {
                    firstname: user.firstname,
                    lastname: user.lastname,
                    email: user.email,
                    email_verified: true,
                    enabled: user.enabled,
                    required_actions: None,
                },
            )
            .await?;

        // Remove verify_email required action
        let _ = self
            .user_required_action_repository
            .remove_required_action(token_record.user_id, RequiredAction::VerifyEmail)
            .await;

        // Clean up other tokens for this user
        let _ = self
            .email_verification_token_repository
            .delete_by_user_id(token_record.user_id)
            .await;

        Ok(VerifyEmailResult {
            user_id: token_record.user_id,
            verified: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        common::{email::MockEmailPort, entities::app_errors::CoreError},
        email_template::{
            entities::{EmailTemplate, EmailType},
            ports::{MockEmailTemplateRepository, TemplateRenderer},
        },
        email_verification::entities::EmailVerificationToken,
        realm::{
            entities::{Realm, RealmSetting, SmtpConfig, SmtpEncryption},
            ports::{MockRealmRepository, MockSmtpConfigRepository},
        },
        user::{
            entities::{User, UserConfig},
            ports::{MockUserRepository, MockUserRequiredActionRepository},
        },
    };
    use ferriskey_domain::realm::RealmId;
    use mockall::predicate::*;

    struct TestRenderer;

    impl TemplateRenderer for TestRenderer {
        fn render_to_intermediate(
            &self,
            _structure: &serde_json::Value,
        ) -> Result<String, CoreError> {
            Ok("<mjml><mj-body>Test</mjml>".to_string())
        }

        fn render_to_html(&self, _intermediate: &str) -> Result<String, CoreError> {
            Ok("<html><body>Test</body></html>".to_string())
        }
    }

    fn test_realm() -> Realm {
        Realm {
            id: RealmId::new(Uuid::new_v4()),
            name: "test-realm".to_string(),
            settings: Some(RealmSetting::new(
                RealmId::new(Uuid::new_v4()),
                Some("RS256".to_string()),
            )),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_user(realm: &Realm) -> User {
        User::new(UserConfig {
            realm_id: realm.id,
            client_id: None,
            username: "testuser".to_string(),
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            email: "test@example.com".to_string(),
            email_verified: false,
            enabled: true,
        })
    }

    fn test_smtp_config(realm: &Realm) -> SmtpConfig {
        SmtpConfig {
            id: Uuid::new_v4(),
            realm_id: realm.id.into(),
            host: "smtp.example.com".to_string(),
            port: 587,
            username: "smtp_user".to_string(),
            password: "smtp_pass".to_string(),
            from_email: "noreply@example.com".to_string(),
            from_name: "Test".to_string(),
            encryption: SmtpEncryption::Tls,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_template(realm: &Realm) -> EmailTemplate {
        EmailTemplate {
            id: Uuid::new_v4(),
            realm_id: realm.id.into(),
            name: "Verify Email".to_string(),
            email_type: EmailType::EmailVerification,
            structure: serde_json::json!({"type": "root", "children": []}),
            mjml: "<mjml></mjml>".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_token(user_id: Uuid, realm_id: RealmId, token_hash: String) -> EmailVerificationToken {
        EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id,
            realm_id: realm_id.into(),
            token_hash,
            expires_at: Utc::now() + Duration::hours(24),
            created_at: Utc::now(),
            used_at: None,
        }
    }

    fn build_service(
        evrt: MockEmailVerificationTokenRepository,
        user_repo: MockUserRepository,
        realm_repo: MockRealmRepository,
        ura_repo: MockUserRequiredActionRepository,
        email_port: MockEmailPort,
        smtp_repo: MockSmtpConfigRepository,
        et_repo: MockEmailTemplateRepository,
    ) -> EmailVerificationServiceImpl<
        MockEmailVerificationTokenRepository,
        MockUserRepository,
        MockRealmRepository,
        MockUserRequiredActionRepository,
        MockEmailPort,
        MockSmtpConfigRepository,
        MockEmailTemplateRepository,
        TestRenderer,
    > {
        EmailVerificationServiceImpl::new(
            Arc::new(evrt),
            Arc::new(user_repo),
            Arc::new(realm_repo),
            Arc::new(ura_repo),
            Arc::new(email_port),
            Arc::new(smtp_repo),
            Arc::new(et_repo),
            Arc::new(TestRenderer),
        )
    }

    #[test]
    fn test_generate_token_hash_deterministic() {
        let token = "test-token-value";
        let hash1 = generate_token_hash(token);
        let hash2 = generate_token_hash(token);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_generate_token_hash_different_inputs() {
        let hash1 = generate_token_hash("token-a");
        let hash2 = generate_token_hash("token-b");
        assert_ne!(hash1, hash2);
    }

    // Restructured test
    #[tokio::test]
    async fn test_verify_email_success() {
        let realm = test_realm();
        let user = test_user(&realm);
        let user_id = user.id;
        let raw_token = "valid-token-123";
        let token_hash = generate_token_hash(raw_token);
        let token_record = test_token(user_id, realm.id, token_hash.clone());

        let mut evrt = MockEmailVerificationTokenRepository::new();
        let tc = token_record.clone();
        evrt.expect_find_valid_by_hash()
            .return_once(move |_| Box::pin(async move { Ok(Some(tc)) }));
        evrt.expect_mark_used()
            .return_once(|_| Box::pin(async move { Ok(()) }));
        evrt.expect_delete_by_user_id()
            .return_once(|_| Box::pin(async move { Ok(0) }));

        let user_clone = user.clone();
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_get_by_id()
            .return_once(move |_| Box::pin(async move { Ok(user_clone) }));
        user_repo
            .expect_update_user()
            .returning(|_, _| Box::pin(async move { Ok(test_user(&test_realm())) }));

        let mut ura_repo = MockUserRequiredActionRepository::new();
        ura_repo
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        let service = build_service(
            evrt,
            user_repo,
            MockRealmRepository::new(),
            ura_repo,
            MockEmailPort::new(),
            MockSmtpConfigRepository::new(),
            MockEmailTemplateRepository::new(),
        );

        let result = service.verify_email(raw_token.to_string()).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!(res.user_id, user_id);
        assert!(res.verified);
    }

    #[tokio::test]
    async fn test_verify_email_invalid_token() {
        let mut evrt = MockEmailVerificationTokenRepository::new();
        evrt.expect_find_valid_by_hash()
            .return_once(|_| Box::pin(async move { Ok(None) }));

        let service = build_service(
            evrt,
            MockUserRepository::new(),
            MockRealmRepository::new(),
            MockUserRequiredActionRepository::new(),
            MockEmailPort::new(),
            MockSmtpConfigRepository::new(),
            MockEmailTemplateRepository::new(),
        );

        let result = service.verify_email("invalid-token".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::InvalidOrExpiredToken
        ));
    }

    #[tokio::test]
    async fn test_verify_email_expired_token() {
        let realm = test_realm();
        let user = test_user(&realm);
        let raw_token = "expired-token";
        let token_hash = generate_token_hash(raw_token);

        let _expired_token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            realm_id: realm.id.into(),
            token_hash: token_hash.clone(),
            expires_at: Utc::now() - Duration::hours(1),
            created_at: Utc::now() - Duration::hours(25),
            used_at: None,
        };

        let mut evrt = MockEmailVerificationTokenRepository::new();
        // DB already filters expired tokens, so find_valid_by_hash returns None
        evrt.expect_find_valid_by_hash()
            .return_once(move |_| Box::pin(async move { Ok(None) }));

        let service = build_service(
            evrt,
            MockUserRepository::new(),
            MockRealmRepository::new(),
            MockUserRequiredActionRepository::new(),
            MockEmailPort::new(),
            MockSmtpConfigRepository::new(),
            MockEmailTemplateRepository::new(),
        );

        let result = service.verify_email(raw_token.to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::InvalidOrExpiredToken
        ));
    }

    #[tokio::test]
    async fn test_verify_email_already_used_token() {
        let realm = test_realm();
        let user = test_user(&realm);
        let raw_token = "used-token";
        let token_hash = generate_token_hash(raw_token);

        let _used_token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: user.id,
            realm_id: realm.id.into(),
            token_hash: token_hash.clone(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            used_at: Some(Utc::now() - Duration::minutes(5)),
        };

        let mut evrt = MockEmailVerificationTokenRepository::new();
        // find_valid_by_hash filters used_at IS NULL, so returns None for used tokens
        evrt.expect_find_valid_by_hash()
            .return_once(move |_| Box::pin(async move { Ok(None) }));

        let service = build_service(
            evrt,
            MockUserRepository::new(),
            MockRealmRepository::new(),
            MockUserRequiredActionRepository::new(),
            MockEmailPort::new(),
            MockSmtpConfigRepository::new(),
            MockEmailTemplateRepository::new(),
        );

        let result = service.verify_email(raw_token.to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoreError::InvalidOrExpiredToken
        ));
    }

    #[tokio::test]
    async fn test_send_verification_email_realm_not_found() {
        let mut realm_repo = MockRealmRepository::new();
        realm_repo
            .expect_get_by_name()
            .return_once(|_| Box::pin(async move { Ok(None) }));

        let service = build_service(
            MockEmailVerificationTokenRepository::new(),
            MockUserRepository::new(),
            realm_repo,
            MockUserRequiredActionRepository::new(),
            MockEmailPort::new(),
            MockSmtpConfigRepository::new(),
            MockEmailTemplateRepository::new(),
        );

        let result = service
            .send_verification_email(
                Uuid::new_v4(),
                "nonexistent-realm".to_string(),
                "http://localhost".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::InvalidRealm));
    }

    #[tokio::test]
    async fn test_send_verification_email_success_with_template() {
        let mut realm = test_realm();
        let template = test_template(&realm);
        let smtp_config = test_smtp_config(&realm);

        // Configure realm settings with template
        let mut settings = RealmSetting::new(realm.id, Some("RS256".to_string()));
        settings.email_verification_template_id = Some(template.id);
        settings.email_verification_ttl_hours = 24;
        realm.settings = Some(settings);

        let user = test_user(&realm);
        let realm_clone = realm.clone();
        let user_clone = user.clone();
        let template_clone = template.clone();
        let smtp_clone = smtp_config.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user_clone.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut evrt = MockEmailVerificationTokenRepository::new();
        evrt.expect_create().returning(|_| {
            let token = EmailVerificationToken {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                realm_id: Uuid::new_v4().into(),
                token_hash: "hash".to_string(),
                expires_at: Utc::now() + Duration::hours(24),
                created_at: Utc::now(),
                used_at: None,
            };
            Box::pin(async move { Ok(token) })
        });

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |_| {
            let t = template_clone.clone();
            Box::pin(async move { Ok(Some(t)) })
        });

        let mut smtp_repo = MockSmtpConfigRepository::new();
        smtp_repo.expect_get_by_realm_id().returning(move |_| {
            let s = smtp_clone.clone();
            Box::pin(async move { Ok(Some(s)) })
        });

        let mut email_port = MockEmailPort::new();
        email_port
            .expect_send_email()
            .returning(|_, _, _, _, _| Box::pin(async move { Ok(()) }));

        let service = build_service(
            evrt,
            user_repo,
            realm_repo,
            MockUserRequiredActionRepository::new(),
            email_port,
            smtp_repo,
            et_repo,
        );

        let result = service
            .send_verification_email(
                user.id,
                "test-realm".to_string(),
                "http://localhost".to_string(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_verification_email_no_template() {
        let mut realm = test_realm();
        // Settings with no template configured
        let settings = RealmSetting::new(realm.id, Some("RS256".to_string()));
        realm.settings = Some(settings);

        let user = test_user(&realm);
        let realm_clone = realm.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut evrt = MockEmailVerificationTokenRepository::new();
        evrt.expect_create().returning(|_| {
            let token = EmailVerificationToken {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                realm_id: Uuid::new_v4().into(),
                token_hash: "hash".to_string(),
                expires_at: Utc::now() + Duration::hours(24),
                created_at: Utc::now(),
                used_at: None,
            };
            Box::pin(async move { Ok(token) })
        });

        // SMTP repo is still called even when template_id is None (match on tuple)
        let mut smtp_repo = MockSmtpConfigRepository::new();
        smtp_repo
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async move { Ok(None) }));

        let service = build_service(
            evrt,
            user_repo,
            realm_repo,
            MockUserRequiredActionRepository::new(),
            MockEmailPort::new(),
            smtp_repo,
            MockEmailTemplateRepository::new(),
        );

        // Should succeed even without template (just logs warning)
        let result = service
            .send_verification_email(
                Uuid::new_v4(),
                "test-realm".to_string(),
                "http://localhost".to_string(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_verification_email_no_smtp() {
        let mut realm = test_realm();
        let template = test_template(&realm);

        let mut settings = RealmSetting::new(realm.id, Some("RS256".to_string()));
        settings.email_verification_template_id = Some(template.id);
        realm.settings = Some(settings);

        let user = test_user(&realm);
        let realm_clone = realm.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut evrt = MockEmailVerificationTokenRepository::new();
        evrt.expect_create().returning(|_| {
            Box::pin(async move {
                Ok(EmailVerificationToken {
                    id: Uuid::new_v4(),
                    user_id: Uuid::new_v4(),
                    realm_id: Uuid::new_v4().into(),
                    token_hash: "hash".to_string(),
                    expires_at: Utc::now() + Duration::hours(24),
                    created_at: Utc::now(),
                    used_at: None,
                })
            })
        });

        // SMTP repo returns None (no config)
        let mut smtp_repo = MockSmtpConfigRepository::new();
        smtp_repo
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async move { Ok(None) }));

        let service = build_service(
            evrt,
            user_repo,
            realm_repo,
            MockUserRequiredActionRepository::new(),
            MockEmailPort::new(),
            smtp_repo,
            MockEmailTemplateRepository::new(),
        );

        // Should succeed even without SMTP (just logs warning)
        let result = service
            .send_verification_email(
                Uuid::new_v4(),
                "test-realm".to_string(),
                "http://localhost".to_string(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_verification_email_custom_ttl() {
        let mut realm = test_realm();
        let template = test_template(&realm);
        let smtp_config = test_smtp_config(&realm);

        let mut settings = RealmSetting::new(realm.id, Some("RS256".to_string()));
        settings.email_verification_template_id = Some(template.id);
        settings.email_verification_ttl_hours = 1; // 1 hour TTL
        realm.settings = Some(settings);

        let user = test_user(&realm);
        let realm_clone = realm.clone();
        let template_clone = template.clone();
        let smtp_clone = smtp_config.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut evrt = MockEmailVerificationTokenRepository::new();
        evrt.expect_create()
            .returning(|input: CreateEmailVerificationTokenInput| {
                // Verify TTL is approximately 1 hour
                let expected_expiry = Utc::now() + Duration::hours(1);
                let diff = (input.expires_at - expected_expiry).num_seconds().abs();
                assert!(diff < 5, "TTL should be approximately 1 hour");
                Box::pin(async move {
                    Ok(EmailVerificationToken {
                        id: Uuid::new_v4(),
                        user_id: input.user_id,
                        realm_id: input.realm_id,
                        token_hash: input.token_hash,
                        expires_at: input.expires_at,
                        created_at: Utc::now(),
                        used_at: None,
                    })
                })
            });

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |_| {
            let t = template_clone.clone();
            Box::pin(async move { Ok(Some(t)) })
        });

        let mut smtp_repo = MockSmtpConfigRepository::new();
        smtp_repo.expect_get_by_realm_id().returning(move |_| {
            let s = smtp_clone.clone();
            Box::pin(async move { Ok(Some(s)) })
        });

        let mut email_port = MockEmailPort::new();
        email_port
            .expect_send_email()
            .returning(|_, _, subject, body, _| {
                // Verify "1 hour" singular form in the body
                assert_eq!(subject, "Verify your email address");
                assert!(
                    body.contains("1 hour"),
                    "Body should contain '1 hour' singular form"
                );
                assert!(
                    !body.contains("hours"),
                    "Body should not contain 'hours' for 1 hour TTL"
                );
                Box::pin(async move { Ok(()) })
            });

        let service = build_service(
            evrt,
            user_repo,
            realm_repo,
            MockUserRequiredActionRepository::new(),
            email_port,
            smtp_repo,
            et_repo,
        );

        let result = service
            .send_verification_email(
                Uuid::new_v4(),
                "test-realm".to_string(),
                "http://localhost".to_string(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_verification_email_send_failure_is_non_fatal() {
        let mut realm = test_realm();
        let template = test_template(&realm);
        let smtp_config = test_smtp_config(&realm);

        let mut settings = RealmSetting::new(realm.id, Some("RS256".to_string()));
        settings.email_verification_template_id = Some(template.id);
        realm.settings = Some(settings);

        let user = test_user(&realm);
        let realm_clone = realm.clone();
        let template_clone = template.clone();
        let smtp_clone = smtp_config.clone();

        let mut realm_repo = MockRealmRepository::new();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_by_id().returning(move |_| {
            let u = user.clone();
            Box::pin(async move { Ok(u) })
        });

        let mut evrt = MockEmailVerificationTokenRepository::new();
        evrt.expect_create().returning(|_| {
            Box::pin(async move {
                Ok(EmailVerificationToken {
                    id: Uuid::new_v4(),
                    user_id: Uuid::new_v4(),
                    realm_id: Uuid::new_v4().into(),
                    token_hash: "hash".to_string(),
                    expires_at: Utc::now() + Duration::hours(24),
                    created_at: Utc::now(),
                    used_at: None,
                })
            })
        });

        let mut et_repo = MockEmailTemplateRepository::new();
        et_repo.expect_get_by_id().returning(move |_| {
            let t = template_clone.clone();
            Box::pin(async move { Ok(Some(t)) })
        });

        let mut smtp_repo = MockSmtpConfigRepository::new();
        smtp_repo.expect_get_by_realm_id().returning(move |_| {
            let s = smtp_clone.clone();
            Box::pin(async move { Ok(Some(s)) })
        });

        let mut email_port = MockEmailPort::new();
        email_port.expect_send_email().returning(|_, _, _, _, _| {
            Box::pin(async move { Err(CoreError::InternalServerError) })
        });

        let service = build_service(
            evrt,
            user_repo,
            realm_repo,
            MockUserRequiredActionRepository::new(),
            email_port,
            smtp_repo,
            et_repo,
        );

        // Email send failure should NOT propagate - it's fire-and-forget
        let result = service
            .send_verification_email(
                Uuid::new_v4(),
                "test-realm".to_string(),
                "http://localhost".to_string(),
            )
            .await;

        assert!(result.is_ok());
    }
}
