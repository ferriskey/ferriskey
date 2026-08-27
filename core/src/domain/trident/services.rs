use std::{sync::Arc, time::SystemTime, time::UNIX_EPOCH};

use chrono::{Duration, Utc};
use ferriskey_domain::generate_uuid_v7;
use futures::future::try_join_all;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha1::Sha1;
use tracing::{debug, error, warn};
use uuid::Uuid;
use webauthn_rs::prelude::*;
use zeroize::Zeroize;

use crate::{
    domain::{
        authentication::{
            entities::{AuthSession, WebAuthnChallenge},
            ports::AuthSessionRepository,
            services::lockout_compute_locked_until,
            value_objects::Identity,
        },
        common::{
            email::EmailPort, entities::app_errors::CoreError, generate_random_string,
            generate_random_token,
        },
        credential::{
            entities::{Credential, CredentialData, CredentialOverview, CredentialType},
            ports::CredentialRepository,
        },
        crypto::HasherRepository,
        email_template::{
            entities::interpolate_variables,
            ports::{EmailTemplateRepository, TemplateRenderer},
        },
        password_policy::{
            entity::PasswordPolicy, repository::PasswordPolicyRepository,
            service::violations_to_core_error, validator,
        },
        realm::{
            entities::RealmId,
            ports::{RealmRepository, SmtpConfigRepository},
        },
        seawatch::{
            entities::{EventStatus, SecurityEvent, SecurityEventType},
            ports::SecurityEventRepository,
        },
        session::ports::TokenRevocationPort,
        trident::{
            entities::{MfaRecoveryCode, PasswordResetToken, TotpSecret},
            mfa_policy::{PendingAuthStep, pending_auth_step},
            ports::{
                BurnRecoveryCodeInput, BurnRecoveryCodeOutput, ChallengeOtpInput,
                ChallengeOtpOutput, CompletePasswordResetInput, CompletePasswordResetOutput,
                CompletePasswordResetWithRecoveryCodeInput, GenerateRecoveryCodeInput,
                GenerateRecoveryCodeOutput, MagicLinkInput, MagicLinkRepository,
                OtpEnrollmentRepository, PasskeyAuthenticateInput, PasskeyAuthenticateOutput,
                PasskeyRegisterOptionsSelfServiceInput, PasskeyRegisterSelfServiceInput,
                PasskeyRequestOptionsInput, PasswordResetTokenRepository, ReauthenticateInput,
                ReauthenticateOutput, RecoveryCodeFormatter, RecoveryCodeRepository,
                RequestPasswordResetInput, SetupOtpInput, SetupOtpOutput, StepUpTokenRecord,
                StepUpTokenRepository, TridentService, UpdatePasswordInput, VerifyMagicLinkInput,
                VerifyOtpInput, VerifyOtpOutput, VerifyResetTokenInput, WebAuthnChallengeRecord,
                WebAuthnChallengeRepository, WebAuthnPublicKeyAuthenticateInput,
                WebAuthnPublicKeyAuthenticateOutput, WebAuthnPublicKeyCreateOptionsInput,
                WebAuthnPublicKeyCreateOptionsOutput, WebAuthnPublicKeyRequestOptionsInput,
                WebAuthnPublicKeyRequestOptionsOutput, WebAuthnRpInfo,
                WebAuthnValidatePublicKeyInput, WebAuthnValidatePublicKeyOutput,
            },
        },
        user::{
            entities::{RequiredAction, RequiredActionError},
            ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
        },
        webhook::{
            entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
            ports::WebhookRepository,
        },
    },
    infrastructure::recovery_code::formatters::{
        B32Split4RecoveryCodeFormatter, RecoveryCodeFormat,
    },
};

type HmacSha1 = Hmac<Sha1>;

/// How long a candidate TOTP secret handed out by `setup_otp` stays claimable by
/// `verify_otp`. Short enough that an abandoned enrolment is not left standing as a
/// second, silently-valid factor.
const OTP_ENROLLMENT_TTL_MINUTES: i64 = 5;

/// How long a WebAuthn registration challenge stays usable. The challenge was already
/// stamped with `webauthn_challenge_issued_at` but nothing ever read it back.
const WEBAUTHN_CHALLENGE_TTL_MINUTES: i64 = 5;

/// How long a self-service passkey registration challenge stays valid.
const PENDING_REGISTRATION_TTL: chrono::Duration = chrono::Duration::seconds(600);

/// How long a step-up token minted by `/me/reauthenticate` stays valid.
const STEP_UP_TOKEN_TTL: chrono::Duration = chrono::Duration::seconds(300);

async fn store_pending_registration<WCR: WebAuthnChallengeRepository>(
    webauthn_challenge_repository: &WCR,
    user_id: Uuid,
    challenge: WebAuthnChallenge,
) -> Result<(), CoreError> {
    let expires_at = Utc::now() + PENDING_REGISTRATION_TTL;
    webauthn_challenge_repository
        .save(WebAuthnChallengeRecord {
            user_id,
            challenge,
            expires_at,
        })
        .await
        .map_err(|e| {
            error!("Failed to persist self-service passkey challenge: {e:?}");
            CoreError::InternalServerError
        })
}

async fn take_pending_registration<WCR: WebAuthnChallengeRepository>(
    webauthn_challenge_repository: &WCR,
    user_id: Uuid,
) -> Result<WebAuthnChallenge, CoreError> {
    webauthn_challenge_repository
        .take(user_id)
        .await
        .map_err(|e| {
            error!("Failed to load self-service passkey challenge: {e:?}");
            CoreError::InternalServerError
        })?
        .ok_or(CoreError::WebAuthnMissingChallenge)
}

fn generate_secret() -> Result<TotpSecret, CoreError> {
    let mut bytes = [0u8; 20];
    rand::thread_rng()
        .try_fill_bytes(&mut bytes)
        .map_err(|_| CoreError::InternalServerError)?;

    let base32 = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes);

    Ok(TotpSecret::from_base32(&base32))
}

fn generate_otpauth_uri(issuer: &str, user_email: &str, secret: &TotpSecret) -> String {
    let encoded_secret = secret.base32_encoded();

    let issuer_encoded = urlencoding::encode(issuer);
    let label_encoded = urlencoding::encode(user_email);

    format!(
        "otpauth://totp/{label_encoded}?secret={encoded_secret}&issuer={issuer_encoded}&algorithm=SHA1&digits=6&period=30"
    )
}

fn generate_totp_code(secret: &[u8], counter: u64, digits: u32) -> Result<u32, CoreError> {
    let mut mac = HmacSha1::new_from_slice(secret).map_err(|_| CoreError::InternalServerError)?;

    let mut counter_bytes = [0u8; 8];

    counter_bytes.copy_from_slice(&counter.to_be_bytes());

    mac.update(&counter_bytes);

    let hmac_result = mac.finalize().into_bytes();

    let offset = (hmac_result[19] & 0x0f) as usize;
    let code = ((hmac_result[offset] as u32 & 0x7f) << 24)
        | ((hmac_result[offset + 1] as u32) << 16)
        | ((hmac_result[offset + 2] as u32) << 8)
        | (hmac_result[offset + 3] as u32);

    Ok(code % 10u32.pow(digits))
}

fn verify(secret: &TotpSecret, code: &str) -> Result<bool, CoreError> {
    let Ok(expected_code) = code.parse::<u32>() else {
        error!("failed to parse code: {}", code);
        return Ok(false);
    };

    let Ok(secret_bytes) = secret.to_bytes() else {
        error!("failed to convert secret to bytes");
        return Ok(false);
    };

    let time_step = 30;
    let digits = 6;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_secs();

    let counter = now / time_step;

    let counters_to_check = [counter.saturating_sub(1), counter, counter + 1];

    for &check_counter in counters_to_check.iter() {
        let generated = generate_totp_code(&secret_bytes, check_counter, digits)?;

        if generated == expected_code {
            return Ok(true);
        }
    }

    Ok(false)
}

fn format_code(code: &MfaRecoveryCode, format: RecoveryCodeFormat) -> String {
    match format {
        RecoveryCodeFormat::B32Split4 => B32Split4RecoveryCodeFormatter::format(code),
    }
}

fn decode_string(code: String, format: RecoveryCodeFormat) -> Result<MfaRecoveryCode, CoreError> {
    match format {
        RecoveryCodeFormat::B32Split4 => B32Split4RecoveryCodeFormatter::decode(code),
    }
}

fn build_webauthn_client(rp_info: WebAuthnRpInfo) -> Result<Webauthn, CoreError> {
    let rp_url = Url::parse(&rp_info.allowed_origin).map_err(|e| {
        error!("Failed to parse server_host as URL: {e}");
        CoreError::InternalServerError
    })?;

    WebauthnBuilder::new(&rp_info.rp_id, &rp_url)
        .map_err(|e| {
            error!("Failed to build Webauthn client: {e:?}");
            CoreError::InternalServerError
        })?
        .build()
        .map_err(|e| {
            error!("Failed to build Webauthn client: {e:?}");
            CoreError::InternalServerError
        })
}

#[derive(Clone, Debug)]
pub struct TridentServiceImpl<
    CR,
    RC,
    AS,
    H,
    URA,
    ML,
    UR,
    RR,
    ES,
    SC,
    PRT,
    SE,
    WH,
    ETR,
    TR,
    PPR,
    OER,
    URR,
    TRV,
    WCR,
    SUT,
> where
    CR: CredentialRepository,
    RC: RecoveryCodeRepository,
    AS: AuthSessionRepository,
    H: HasherRepository,
    URA: UserRequiredActionRepository,
    ML: MagicLinkRepository,
    UR: UserRepository,
    RR: RealmRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    PRT: PasswordResetTokenRepository,
    SE: SecurityEventRepository,
    WH: WebhookRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
    PPR: PasswordPolicyRepository,
    OER: OtpEnrollmentRepository,
    URR: UserRoleRepository,
    TRV: TokenRevocationPort,
    WCR: WebAuthnChallengeRepository,
    SUT: StepUpTokenRepository,
{
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) recovery_code_repository: Arc<RC>,
    pub(crate) auth_session_repository: Arc<AS>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) user_required_action_repository: Arc<URA>,
    pub(crate) magic_link_repository: Arc<ML>,
    pub(crate) user_repository: Arc<UR>,
    pub(crate) realm_repository: Arc<RR>,
    pub(crate) email_port: Arc<ES>,
    pub(crate) smtp_config_repository: Arc<SC>,
    pub(crate) password_reset_token_repository: Arc<PRT>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) webhook_repository: Arc<WH>,
    pub(crate) email_template_repository: Arc<ETR>,
    pub(crate) template_renderer: Arc<TR>,
    pub(crate) password_policy_repository: Arc<PPR>,
    pub(crate) otp_enrollment_repository: Arc<OER>,
    pub(crate) user_role_repository: Arc<URR>,
    pub(crate) token_revocation: Arc<TRV>,
    pub(crate) webauthn_challenge_repository: Arc<WCR>,
    pub(crate) step_up_token_repository: Arc<SUT>,
}

impl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH, ETR, TR, PPR, OER, URR, TRV, WCR, SUT>
    TridentServiceImpl<
        CR,
        RC,
        AS,
        H,
        URA,
        ML,
        UR,
        RR,
        ES,
        SC,
        PRT,
        SE,
        WH,
        ETR,
        TR,
        PPR,
        OER,
        URR,
        TRV,
        WCR,
        SUT,
    >
where
    CR: CredentialRepository,
    RC: RecoveryCodeRepository,
    AS: AuthSessionRepository,
    H: HasherRepository,
    URA: UserRequiredActionRepository,
    ML: MagicLinkRepository,
    UR: UserRepository,
    RR: RealmRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    PRT: PasswordResetTokenRepository,
    SE: SecurityEventRepository,
    WH: WebhookRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
    PPR: PasswordPolicyRepository,
    OER: OtpEnrollmentRepository,
    URR: UserRoleRepository,
    TRV: TokenRevocationPort,
    WCR: WebAuthnChallengeRepository,
    SUT: StepUpTokenRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        credential_repository: Arc<CR>,
        recovery_code_repository: Arc<RC>,
        auth_session_repository: Arc<AS>,
        hasher_repository: Arc<H>,
        user_required_action_repository: Arc<URA>,
        magic_link_repository: Arc<ML>,
        user_repository: Arc<UR>,
        realm_repository: Arc<RR>,
        email_port: Arc<ES>,
        smtp_config_repository: Arc<SC>,
        password_reset_token_repository: Arc<PRT>,
        security_event_repository: Arc<SE>,
        webhook_repository: Arc<WH>,
        email_template_repository: Arc<ETR>,
        template_renderer: Arc<TR>,
        password_policy_repository: Arc<PPR>,
        otp_enrollment_repository: Arc<OER>,
        user_role_repository: Arc<URR>,
        token_revocation: Arc<TRV>,
        webauthn_challenge_repository: Arc<WCR>,
        step_up_token_repository: Arc<SUT>,
    ) -> Self {
        Self {
            credential_repository,
            recovery_code_repository,
            auth_session_repository,
            hasher_repository,
            user_required_action_repository,
            magic_link_repository,
            user_repository,
            realm_repository,
            email_port,
            smtp_config_repository,
            password_reset_token_repository,
            security_event_repository,
            webhook_repository,
            email_template_repository,
            template_renderer,
            password_policy_repository,
            otp_enrollment_repository,
            user_role_repository,
            token_revocation,
            webauthn_challenge_repository,
            step_up_token_repository,
        }
    }

    /// Mint a short-lived, single-use, user-bound step-up token after a
    /// successful re-authentication. The raw token is returned to the caller
    /// and only its hash is persisted, so a leaked database never exposes
    /// usable tokens.
    async fn mint_step_up_token(&self, user_id: Uuid) -> Result<String, CoreError> {
        let raw = generate_random_token();
        let hash = self
            .hasher_repository
            .hash_magic_token(&raw)
            .await
            .map_err(|e| {
                error!("Failed to hash step-up token: {e:?}");
                CoreError::InternalServerError
            })?;

        let expires_at = Utc::now() + STEP_UP_TOKEN_TTL;
        self.step_up_token_repository
            .save(StepUpTokenRecord {
                id: generate_uuid_v7(),
                user_id,
                token_hash: hash.hash,
                expires_at,
            })
            .await
            .map_err(|e| {
                error!("Failed to persist step-up token: {e:?}");
                CoreError::InternalServerError
            })?;

        Ok(raw)
    }

    /// Consume a step-up token presented by the caller, atomically removing it
    /// so it cannot be reused. Returns `StepUpTokenInvalid` when the token is
    /// missing, expired, or does not match.
    async fn consume_step_up_token(&self, user_id: Uuid, presented: &str) -> Result<(), CoreError> {
        let candidates = self
            .step_up_token_repository
            .find_active(user_id)
            .await
            .map_err(|e| {
                error!("Failed to load active step-up tokens: {e:?}");
                CoreError::InternalServerError
            })?;

        for candidate in candidates {
            let valid = self
                .hasher_repository
                .verify_magic_token(presented, &candidate.token_hash)
                .await
                .map_err(|e| {
                    error!("Failed to verify step-up token: {e:?}");
                    CoreError::InternalServerError
                })?;

            if !valid {
                continue;
            }

            let deleted = self
                .step_up_token_repository
                .delete_by_id(candidate.id)
                .await
                .map_err(|e| {
                    error!("Failed to delete matched step-up token: {e:?}");
                    CoreError::InternalServerError
                })?;

            if deleted {
                return Ok(());
            }

            return Err(CoreError::StepUpTokenInvalid);
        }

        Err(CoreError::StepUpTokenInvalid)
    }

    async fn verify_recovery_code_candidate(
        &self,
        user_code: &MfaRecoveryCode,
        code_cred: &Credential,
    ) -> Result<bool, CoreError> {
        let (hash_iterations, algorithm) = match &code_cred.credential_data {
            CredentialData::Hash {
                hash_iterations,
                algorithm,
            } => (*hash_iterations, algorithm.clone()),
            _ => {
                error!(
                    "A recovery code credential has no Hash credential data. This is a server bug."
                );
                return Err(CoreError::InternalServerError);
            }
        };
        let salt = code_cred
            .salt
            .as_ref()
            .ok_or(CoreError::InternalServerError)?;

        self.recovery_code_repository
            .verify(
                user_code,
                &code_cred.secret_data,
                hash_iterations,
                &algorithm,
                salt,
            )
            .await
    }

    async fn find_matching_recovery_code(
        &self,
        user_id: Uuid,
        user_code: &MfaRecoveryCode,
        lookup: &str,
    ) -> Result<Option<Credential>, CoreError> {
        if let Some(candidate) = self
            .credential_repository
            .find_recovery_code_by_lookup(user_id, lookup)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?
        {
            return Ok(self
                .verify_recovery_code_candidate(user_code, &candidate)
                .await?
                .then_some(candidate));
        }

        // Rows created before the lookup column existed have NULL in
        // `recovery_code_lookup`. Keep them working by scanning only those
        // legacy rows as a bounded fallback until they are rotated away.
        let legacy_candidates = self
            .credential_repository
            .get_credentials_by_user_id(user_id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        for candidate in legacy_candidates.into_iter().filter(|credential| {
            credential.credential_type == CredentialType::RecoveryCode
                && credential.recovery_code_lookup.is_none()
        }) {
            if self
                .verify_recovery_code_candidate(user_code, &candidate)
                .await?
            {
                return Ok(Some(candidate));
            }
        }

        Ok(None)
    }

    /// Emit a `ReauthenticationFailed` security event so SeaWatch can detect
    /// brute-force attempts on the step-up endpoint. The caller passes the
    /// realm it already resolved from the authenticated user: a nil fallback
    /// would violate the `security_events.realm_id` foreign key and silently
    /// drop the event.
    async fn emit_reauthentication_failed(&self, realm_id: RealmId, user_id: Uuid) {
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::ReauthenticationFailed,
                    EventStatus::Failure,
                    user_id,
                )
                .with_target("user".to_string(), user_id, None),
            )
            .await
            .inspect_err(|e| warn!("Failed to log reauthentication failure event: {e}"));
    }

    /// Notify the account owner by email when an authentication factor is
    /// added or removed. This is the compensating control for the self-service
    /// MFA endpoints: even if a stolen access token passes the step-up check,
    /// the legitimate user is alerted that a factor changed on their account.
    ///
    /// The email is best-effort: if SMTP is not configured for the realm we log
    /// an `EmailNotSent` event and continue, so a misconfigured realm never
    /// blocks the factor operation itself.
    async fn notify_factor_change(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        factor: &str,
        action: &str,
    ) {
        let user = match self.user_repository.get_by_id(user_id).await {
            Ok(u) => u,
            Err(e) => {
                warn!("Failed to load user for factor-change email: {e:?}");
                return;
            }
        };
        let realm = match self.realm_repository.get_by_id(realm_id).await {
            Ok(Some(r)) => r,
            _ => {
                warn!("Failed to load realm for factor-change email");
                return;
            }
        };

        let subject = match action {
            "enrolled" => "A new sign-in method was added to your account",
            "removed" => "A sign-in method was removed from your account",
            _ => "Your account sign-in methods changed",
        };
        let body = format!(
            "Hello,\n\nA {factor} sign-in method was {action} on your {} account.\n\n\
             If this was you, no further action is needed. If you did not make this change, \
             please contact your administrator immediately and review your account security.\n",
            realm.name
        );

        match self.smtp_config_repository.get_by_realm_id(realm.id).await {
            Ok(Some(smtp_config)) => {
                match self
                    .email_port
                    .send_email(
                        &smtp_config,
                        user.email.as_deref().unwrap_or(""),
                        subject,
                        &body,
                        None,
                    )
                    .await
                {
                    Ok(()) => {
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailSent,
                                    EventStatus::Success,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "email_type": "factor_change",
                                    "factor": factor,
                                    "action": action,
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| {
                                warn!("Failed to log factor-change email sent event: {e}")
                            });
                    }
                    Err(e) => {
                        warn!("Failed to send factor-change email: {e}");
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailNotSent,
                                    EventStatus::Failure,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "reason": format!("Failed to send factor-change email: {e}"),
                                    "email_type": "factor_change",
                                    "factor": factor,
                                    "action": action,
                                    "error_code": "SMTP_SEND_FAILED",
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| {
                                warn!("Failed to log factor-change email not sent event: {e}")
                            });
                    }
                }
            }
            _ => {
                warn!(
                    "SMTP not configured for realm {}, skipping factor-change email",
                    realm.name
                );
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::EmailNotSent,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_details(serde_json::json!({
                            "reason": format!("SMTP not configured for realm {}", realm.name),
                            "email_type": "factor_change",
                            "factor": factor,
                            "action": action,
                            "error_code": "SMTP_NOT_CONFIGURED",
                            "user_id": user.id.to_string(),
                        })),
                    )
                    .await
                    .inspect_err(|e| {
                        warn!("Failed to log factor-change email not sent event: {e}")
                    });
            }
        }
    }

    async fn pending_auth_step_for(
        &self,
        user_id: Uuid,
        actions_satisfied_by_path: &[RequiredAction],
    ) -> Result<Option<PendingAuthStep>, CoreError> {
        let user = self.user_repository.get_by_id(user_id).await?;

        let credentials = self
            .credential_repository
            .get_credentials_by_user_id(user_id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        let has_otp_credential = credentials
            .iter()
            .any(|credential| credential.credential_type == CredentialType::Otp);
        let has_temporary_password = credentials.iter().any(|credential| credential.temporary);

        let roles = self.user_role_repository.get_user_roles(user_id).await?;

        let settings = self
            .realm_repository
            .get_realm_settings(user.realm_id)
            .await?;

        let persisted_actions = user
            .required_actions
            .into_iter()
            .filter(|action| !actions_satisfied_by_path.contains(action))
            .collect::<Vec<RequiredAction>>();

        Ok(pending_auth_step(
            &persisted_actions,
            settings.as_ref(),
            &roles,
            has_otp_credential,
            has_temporary_password,
        ))
    }

    async fn store_auth_code_and_generate_login_url(
        &self,
        auth_session: &AuthSession,
        user_id: Uuid,
        actions_satisfied_by_path: &[RequiredAction],
    ) -> Result<String, CoreError> {
        if let Some(step) = self
            .pending_auth_step_for(user_id, actions_satisfied_by_path)
            .await?
        {
            warn!(
                user_id = %user_id,
                ?step,
                "refusing to issue an authorization code while an authentication step is due"
            );

            return Err(CoreError::Forbidden(
                "an authentication step is still due for this user".to_string(),
            ));
        }

        let authorization_code = generate_random_string();

        self.auth_session_repository
            .update_code_and_user_id(auth_session.id, authorization_code.clone(), user_id)
            .await
            .map_err(|_| CoreError::AuthorizationCodeStorageFailed)?;

        let current_state = auth_session
            .state
            .as_ref()
            .ok_or(CoreError::AuthSessionExpectedState)?;

        Ok(format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, authorization_code, current_state
        ))
    }

    async fn render_email_template(
        &self,
        realm_id: Uuid,
        template_id: Uuid,
        user: &crate::domain::user::entities::User,
        extra_vars: &[(&str, &str)],
    ) -> Result<String, CoreError> {
        let template = self
            .email_template_repository
            .get_by_id(realm_id, template_id)
            .await?
            .ok_or(CoreError::EmailTemplateNotFound)?;

        let html = self.template_renderer.render_to_html(&template.mjml)?;

        let mut variables = std::collections::HashMap::new();
        variables.insert(
            "user.first_name".to_string(),
            user.firstname.clone().unwrap_or_default(),
        );
        variables.insert(
            "user.last_name".to_string(),
            user.lastname.clone().unwrap_or_default(),
        );
        variables.insert(
            "user.email".to_string(),
            user.email.clone().unwrap_or_default(),
        );
        for (key, value) in extra_vars {
            variables.insert(key.to_string(), value.to_string());
        }

        Ok(interpolate_variables(&html, &variables))
    }
}

impl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH, ETR, TR, PPR, OER, URR, TRV, WCR, SUT>
    TridentService
    for TridentServiceImpl<
        CR,
        RC,
        AS,
        H,
        URA,
        ML,
        UR,
        RR,
        ES,
        SC,
        PRT,
        SE,
        WH,
        ETR,
        TR,
        PPR,
        OER,
        URR,
        TRV,
        WCR,
        SUT,
    >
where
    CR: CredentialRepository,
    RC: RecoveryCodeRepository,
    AS: AuthSessionRepository,
    H: HasherRepository,
    URA: UserRequiredActionRepository,
    ML: MagicLinkRepository,
    UR: UserRepository,
    RR: RealmRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    PRT: PasswordResetTokenRepository,
    SE: SecurityEventRepository,
    WH: WebhookRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
    PPR: PasswordPolicyRepository,
    OER: OtpEnrollmentRepository,
    URR: UserRoleRepository,
    TRV: TokenRevocationPort,
    WCR: WebAuthnChallengeRepository,
    SUT: StepUpTokenRepository,
{
    async fn generate_recovery_code(
        &self,
        identity: Identity,
        input: GenerateRecoveryCodeInput,
    ) -> Result<GenerateRecoveryCodeOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        // Regenerating recovery codes invalidates the victim's real codes and
        // hands the caller a fresh set, so it is the fourth operation of the
        // same sensitivity class as enrolling/removing factors: require the
        // same proof of knowledge (a valid, single-use step-up token) instead
        // of trusting the bearer token alone.
        let step_up_token = input.step_up_token.ok_or_else(|| {
            CoreError::Forbidden(
                "re-authentication is required to generate recovery codes".to_string(),
            )
        })?;
        self.consume_step_up_token(user.id, &step_up_token).await?;

        let format =
            RecoveryCodeFormat::try_from(input.format).map_err(CoreError::RecoveryCodeGenError)?;

        let stored_codes = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .into_iter()
            .filter(|cred| cred.credential_type.as_str() == "recovery-code")
            .collect::<Vec<Credential>>();

        let codes = self
            .recovery_code_repository
            .generate_n_recovery_code(input.amount as usize);

        // These are probably not concurrent jobs !
        // They should be parallelized with threads instead of IO tasks for faster operation
        let futures = codes
            .iter()
            .map(|code| self.recovery_code_repository.secure_for_storage(code));
        let secure_codes = try_join_all(futures).await?;

        self.credential_repository
            .create_recovery_code_credentials(user.id, secure_codes)
            .await
            .map_err(|e| {
                error!("{e}");
                CoreError::InternalServerError
            })?;

        // Once new codes stored it's now safe to invalidate the previous recovery codes
        let _ = {
            let futures = stored_codes
                .into_iter()
                .map(|c| self.credential_repository.delete_by_id(c.id));
            try_join_all(futures).await
        }
        .map_err(|e| {
            error!("Failed to delete previously fetched credentials: {e}");
            CoreError::InternalServerError
        })?;

        // Now format the codes into human-readable format for
        // distribution to the user
        let codes = codes
            .into_iter()
            .map(|c| format_code(&c, format.clone()))
            .collect::<Vec<String>>();

        Ok(GenerateRecoveryCodeOutput { codes })
    }

    async fn burn_recovery_code(
        &self,
        identity: Identity,
        input: BurnRecoveryCodeInput,
    ) -> Result<BurnRecoveryCodeOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("Is not an user".to_string())),
        };

        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let format =
            RecoveryCodeFormat::try_from(input.format).map_err(CoreError::RecoveryCodeBurnError)?;

        let user_code = decode_string(input.code, format)?;

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::SessionNotFound)?;

        // Enforce account lockout so the MFA login fallback cannot be used as
        // an unlimited recovery-code oracle (same policy as the unauthenticated
        // recovery-code reset endpoint).
        let realm_settings = self
            .realm_repository
            .get_realm_settings(user.realm_id)
            .await?;
        let lockout_threshold = realm_settings
            .as_ref()
            .map(|s| s.lockout_threshold)
            .unwrap_or(10);
        let lockout_duration_seconds = realm_settings
            .as_ref()
            .map(|s| s.lockout_duration_seconds)
            .unwrap_or(900);
        let now = Utc::now();
        if user.is_locked(now) {
            return Err(CoreError::AccountLocked);
        }

        // Locate the single candidate recovery-code row via the fast lookup key
        // and run Argon2 only once, instead of scanning every stored code.
        let lookup = self.recovery_code_repository.lookup_of(&user_code);
        let burnt_code = self
            .find_matching_recovery_code(user.id, &user_code, &lookup)
            .await?;

        // This doesn't check if there are multiple matches because it is not necessarly a bug
        // It is highly unlikely but a user may have multiple identical recovery codes
        // or it could also be a duplicate storage bug.
        // Anyway, this is not the place to check such a bug
        let burnt_code = match burnt_code {
            Some(code) => code,
            None => {
                // Failed attempt: bump the lockout counter and emit a failure
                // event so SeaWatch can detect brute-force guessing (mirrors
                // the recovery-code reset endpoint).
                let locked_until = lockout_compute_locked_until(
                    user.failed_login_attempts + 1,
                    lockout_threshold,
                    lockout_duration_seconds,
                    now,
                );
                let _ = self
                    .user_repository
                    .increment_failed_login_attempts(user.id, locked_until)
                    .await;
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            user.realm_id,
                            SecurityEventType::RecoveryCodeBurned,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_target("user".to_string(), user.id, None),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log recovery code burn failure event: {e}"));
                return Err(CoreError::RecoveryCodeBurnError(
                    "The provided code is invalid or has already been used".to_string(),
                ));
            }
        };

        self
            .credential_repository
            .delete_by_id(burnt_code.id)
            .await
            .map_err(|e| {
                error!("Failed to delete a credential even though it was just fetched with the same repository: {e}");
                CoreError::InternalServerError
            })?;

        // A valid code proves possession, so clear any accumulated lockout
        // counter from prior failed guesses.
        let _ = self
            .user_repository
            .reset_failed_login_attempts(user.id)
            .await;

        // Audit the burn so a recovery code used to bypass MFA is visible to
        // SeaWatch and the user can be notified.
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    user.realm_id,
                    SecurityEventType::RecoveryCodeBurned,
                    EventStatus::Success,
                    user.id,
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await
            .inspect_err(|e| warn!("Failed to log recovery code burned event: {e}"));

        let authorization_code = generate_random_string();

        self.auth_session_repository
            .update_code_and_user_id(session_code, authorization_code.clone(), user.id)
            .await
            .map_err(|e| CoreError::TotpVerificationFailed(e.to_string()))?;

        let current_state = auth_session.state.ok_or(CoreError::RecoveryCodeBurnError(
            "Invalid session state".to_string(),
        ))?;

        let login_url = format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, authorization_code, current_state
        );

        Ok(BurnRecoveryCodeOutput { login_url })
    }

    async fn webauthn_public_key_create_options(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyCreateOptionsInput,
    ) -> Result<WebAuthnPublicKeyCreateOptionsOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let credentials = self
            .credential_repository
            .get_webauthn_public_key_credentials(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let credentials = {
            let filtered = credentials
                .into_iter()
                .filter_map(|v| v.webauthn_credential_id)
                .collect::<Vec<CredentialID>>();
            if filtered.is_empty() {
                None
            } else {
                // User already has passkeys — clear the required action if present
                let _ = self
                    .user_required_action_repository
                    .remove_required_action(user.id, RequiredAction::ConfigurePasskey)
                    .await;
                Some(filtered)
            }
        };

        let (ccr, pr) = webauthn
            .start_passkey_registration(
                user.id,
                user.email.as_deref().unwrap_or(""),
                &user.username,
                credentials,
            )
            .map_err(|e| {
                error!("Failed to generate webauthn challenge: {e:?}");
                CoreError::InternalServerError
            })?;

        let _ = self
            .auth_session_repository
            .save_webauthn_challenge(session_code, WebAuthnChallenge::Registration(pr))
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(WebAuthnPublicKeyCreateOptionsOutput(ccr))
    }

    async fn webauthn_public_key_create(
        &self,
        identity: Identity,
        input: WebAuthnValidatePublicKeyInput,
    ) -> Result<WebAuthnValidatePublicKeyOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        // Same reasoning as `verify_otp`: this route is reachable with a temporary token
        // obtained from the password alone, so enrolling a passkey must be something the
        // server asked for, not something a caller can decide to do.
        let required_actions = self
            .user_required_action_repository
            .get_required_actions(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if !required_actions.contains(&RequiredAction::ConfigurePasskey) {
            warn!(
                user_id = %user.id,
                "Refused passkey enrolment: user carries no ConfigurePasskey required action"
            );
            return Err(CoreError::Forbidden(
                "passkey enrollment was not requested for this user".to_string(),
            ));
        }

        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        // `webauthn_challenge_issued_at` was already being written on every challenge but
        // never read, so a challenge stayed valid for the whole life of the auth session.
        let issued_at = auth_session
            .webauthn_challenge_issued_at
            .ok_or(CoreError::WebAuthnMissingChallenge)?;

        if Utc::now() - issued_at > Duration::minutes(WEBAUTHN_CHALLENGE_TTL_MINUTES) {
            warn!(user_id = %user.id, "Refused passkey enrolment: registration challenge is stale");
            return Err(CoreError::WebAuthnChallengeFailed);
        }

        let passkey = match auth_session.webauthn_challenge {
            Some(WebAuthnChallenge::Registration(ref pk)) => webauthn
                .finish_passkey_registration(&input.credential, pk)
                .map_err(|e| {
                    debug!("Failed to complete passkey registration: {e:?}");
                    CoreError::Invalid
                }),
            _ => Err(CoreError::Invalid),
        }?;

        self.credential_repository
            .create_webauthn_credential(user.id, passkey)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        // Clearing the action is what closes the enrolment window checked above, so a
        // failure here is worth a trace rather than being swallowed outright.
        if let Err(e) = self
            .user_required_action_repository
            .remove_required_action(user.id, RequiredAction::ConfigurePasskey)
            .await
        {
            warn!(
                user_id = %user.id,
                "Failed to remove ConfigurePasskey required action after passkey enrolment: {e:?}"
            );
        }

        // Audit the enrolment and notify the account owner, matching the
        // self-service passkey route: a compromised temporary login token must
        // not be able to add a factor invisibly.
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    user.realm_id,
                    SecurityEventType::MfaEnrolled,
                    EventStatus::Success,
                    user.id,
                )
                .with_target(
                    "user".to_string(),
                    user.id,
                    Some("passkey".to_string()),
                ),
            )
            .await
            .inspect_err(|e| warn!("Failed to log MFA enrollment event: {e}"));

        self.notify_factor_change(user.id, user.realm_id, "passkey", "enrolled")
            .await;

        Ok(WebAuthnValidatePublicKeyOutput {})
    }

    async fn webauthn_public_key_request_options(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyRequestOptionsInput,
    ) -> Result<WebAuthnPublicKeyRequestOptionsOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let creds = self
            .credential_repository
            .get_webauthn_public_key_credentials(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let creds = creds
            .into_iter()
            .map(|v|
                match v.credential_data {
                    CredentialData::WebAuthn {credential} => {
                        Ok(Passkey::from(*credential))
                    },
                    _ => {
                        error!("A Webauthn credential doesn't hold WebAuthn credential data ! Something went wrong during creation...");
                        Err(CoreError::InternalServerError)
                    }
                }
            )
            .collect::<Result<Vec<Passkey>, CoreError>>()?;

        let (rcr, pa) = webauthn.start_passkey_authentication(&creds).map_err(|e| {
            error!("Failed to generate webauthn challenge: {e:?}");
            CoreError::InternalServerError
        })?;

        let _ = self
            .auth_session_repository
            .save_webauthn_challenge(session_code, WebAuthnChallenge::Authentication(pa))
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(WebAuthnPublicKeyRequestOptionsOutput(rcr))
    }

    async fn webauthn_public_key_authenticate(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyAuthenticateInput,
    ) -> Result<WebAuthnPublicKeyAuthenticateOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let auth_result = match auth_session.webauthn_challenge {
            Some(WebAuthnChallenge::Authentication(ref pa)) => webauthn
                .finish_passkey_authentication(&input.credential, pa)
                .map_err(|e| {
                    error!("Error during webauthn verification: {e:?}");
                    CoreError::WebAuthnChallengeFailed
                }),
            _ => Err(CoreError::WebAuthnMissingChallenge),
        }?;

        if auth_result.needs_update() {
            let _ = self
                .credential_repository
                .update_webauthn_credential(&auth_result)
                .await
                .map_err(|e| {
                    debug!("{e:?}");
                    CoreError::InternalServerError
                })?;
        }

        if !auth_result.user_verified() {
            return Err(CoreError::WebAuthnChallengeFailed);
        }

        let login_url = self
            .store_auth_code_and_generate_login_url(&auth_session, user.id, &[])
            .await?;

        Ok(WebAuthnPublicKeyAuthenticateOutput { login_url })
    }

    async fn passkey_request_options(
        &self,
        input: PasskeyRequestOptionsInput,
    ) -> Result<WebAuthnPublicKeyRequestOptionsOutput, CoreError> {
        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        if let Some(username) = input.username {
            // Non-discoverable: we know the user, fetch their passkeys
            let user = self
                .user_repository
                .get_by_username(username, realm.id)
                .await
                .map_err(|_| CoreError::WebAuthnChallengeFailed)?;

            let creds = self
                .credential_repository
                .get_webauthn_public_key_credentials(user.id)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            let creds = creds
                .into_iter()
                .map(|v| match v.credential_data {
                    CredentialData::WebAuthn { credential } => Ok(Passkey::from(*credential)),
                    _ => {
                        error!("A WebAuthn credential doesn't hold WebAuthn credential data");
                        Err(CoreError::InternalServerError)
                    }
                })
                .collect::<Result<Vec<Passkey>, CoreError>>()?;

            if creds.is_empty() {
                return Err(CoreError::WebAuthnChallengeFailed);
            }

            let (rcr, pa) = webauthn.start_passkey_authentication(&creds).map_err(|e| {
                error!("Failed to start passkey authentication: {e:?}");
                CoreError::InternalServerError
            })?;

            self.auth_session_repository
                .save_webauthn_challenge(session_code, WebAuthnChallenge::Authentication(pa))
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            Ok(WebAuthnPublicKeyRequestOptionsOutput(rcr))
        } else {
            // Discoverable: no user provided, browser will propose available passkeys
            let (rcr, da) = webauthn.start_discoverable_authentication().map_err(|e| {
                error!("Failed to start discoverable authentication: {e:?}");
                CoreError::InternalServerError
            })?;

            self.auth_session_repository
                .save_webauthn_challenge(
                    session_code,
                    WebAuthnChallenge::DiscoverableAuthentication(da),
                )
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            Ok(WebAuthnPublicKeyRequestOptionsOutput(rcr))
        }
    }

    async fn passkey_authenticate(
        &self,
        input: PasskeyAuthenticateInput,
    ) -> Result<PasskeyAuthenticateOutput, CoreError> {
        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let mut auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let webauthn_challenge = auth_session.webauthn_challenge.take();
        let auth_result = match webauthn_challenge {
            Some(WebAuthnChallenge::Authentication(ref pa)) => {
                // Non-discoverable: user was known at challenge time
                webauthn
                    .finish_passkey_authentication(&input.credential, pa)
                    .map_err(|e| {
                        error!("Error during passkey authentication: {e:?}");
                        CoreError::WebAuthnChallengeFailed
                    })?
            }
            Some(WebAuthnChallenge::DiscoverableAuthentication(da)) => {
                // Discoverable: resolve credential from the assertion response
                let user_handle = input
                    .credential
                    .get_user_unique_id()
                    .ok_or(CoreError::WebAuthnChallengeFailed)?;

                let user_uuid = Uuid::from_slice(user_handle)
                    .map_err(|_| CoreError::WebAuthnChallengeFailed)?;

                let cred_id_bytes: &[u8] = input.credential.raw_id.as_ref();

                let credential = self
                    .credential_repository
                    .get_webauthn_credential_by_credential_id_and_user(cred_id_bytes, user_uuid)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?
                    .ok_or(CoreError::WebAuthnChallengeFailed)?;

                let passkey = match credential.credential_data {
                    CredentialData::WebAuthn { credential } => Passkey::from(*credential),
                    _ => return Err(CoreError::WebAuthnChallengeFailed),
                };

                let dk: DiscoverableKey = passkey.into();
                webauthn
                    .finish_discoverable_authentication(&input.credential, da, &[dk])
                    .map_err(|e| {
                        error!("Error during discoverable authentication: {e:?}");
                        CoreError::WebAuthnChallengeFailed
                    })?
            }
            _ => return Err(CoreError::WebAuthnMissingChallenge),
        };

        if auth_result.needs_update() {
            let _ = self
                .credential_repository
                .update_webauthn_credential(&auth_result)
                .await
                .map_err(|e| {
                    debug!("{e:?}");
                    CoreError::InternalServerError
                })?;
        }

        if !auth_result.user_verified() {
            return Err(CoreError::WebAuthnChallengeFailed);
        }

        // Resolve the user from the assertion response
        let user_handle = input
            .credential
            .get_user_unique_id()
            .ok_or(CoreError::WebAuthnChallengeFailed)?;

        let user_uuid =
            Uuid::from_slice(user_handle).map_err(|_| CoreError::WebAuthnChallengeFailed)?;

        let user = self
            .user_repository
            .get_by_id(user_uuid)
            .await
            .map_err(|_| CoreError::WebAuthnChallengeFailed)?;

        if user.realm_id != realm.id {
            return Err(CoreError::WebAuthnChallengeFailed);
        }

        let login_url = self
            .store_auth_code_and_generate_login_url(&auth_session, user.id, &[])
            .await?;

        Ok(PasskeyAuthenticateOutput { login_url })
    }

    async fn challenge_otp(
        &self,
        identity: Identity,
        input: ChallengeOtpInput,
    ) -> Result<ChallengeOtpOutput, CoreError> {
        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::SessionNotFound)?;

        let user_credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        let otp_credential = user_credentials
            .iter()
            .find(|cred| cred.credential_type == CredentialType::Otp)
            .ok_or_else(|| {
                CoreError::TotpVerificationFailed("user has not OTP configured".to_string())
            })?;

        let secret = TotpSecret::from_base32(&otp_credential.secret_data);

        let is_valid = verify(&secret, &input.code)?;

        if !is_valid {
            error!(
                "invalid OTP code for user: {}",
                user.email.as_deref().unwrap_or("")
            );
            return Err(CoreError::TotpVerificationFailed(
                "failed to verify OTP".to_string(),
            ));
        }

        let required_actions = self
            .user_required_action_repository
            .get_required_actions(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if !required_actions.is_empty() {
            return Ok(ChallengeOtpOutput {
                login_url: None,
                required_actions,
                temporary_token: None,
            });
        }

        let authorization_code = generate_random_string();

        self.auth_session_repository
            .update_code_and_user_id(session_code, authorization_code.clone(), user.id)
            .await
            .map_err(|e| CoreError::TotpVerificationFailed(e.to_string()))?;

        let current_state = auth_session.state.ok_or(CoreError::TotpVerificationFailed(
            "invalid session state".to_string(),
        ))?;

        let login_url = format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, authorization_code, current_state
        );

        Ok(ChallengeOtpOutput {
            login_url: Some(login_url),
            required_actions: Vec::new(),
            temporary_token: None,
        })
    }

    async fn setup_otp(
        &self,
        identity: Identity,
        input: SetupOtpInput,
    ) -> Result<SetupOtpOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let secret = generate_secret()?;
        let otpauth_uri =
            generate_otpauth_uri(&input.issuer, user.email.as_deref().unwrap_or(""), &secret);
        let secret = secret.base32_encoded().to_string();

        // Handing the secret to the user is fine — they must type it into their
        // authenticator. What is not fine is taking it back from them on the next call,
        // so record it here and let `verify_otp` read it from this side of the wire.
        // Sweep consumed/expired enrolments first so rows holding a plaintext
        // candidate secret do not accumulate (same pattern as the WebAuthn
        // challenge and step-up token tables).
        let _ = self.otp_enrollment_repository.cleanup_expired().await;
        self.otp_enrollment_repository
            .start_enrollment(
                user.id,
                secret.clone(),
                Utc::now() + Duration::minutes(OTP_ENROLLMENT_TTL_MINUTES),
            )
            .await?;

        Ok(SetupOtpOutput {
            otpauth_uri,
            secret,
        })
    }

    async fn update_password(
        &self,
        identity: Identity,
        input: UpdatePasswordInput,
    ) -> Result<(), CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let policy = self
            .password_policy_repository
            .find_by_realm_id(user.realm_id.into())
            .await?
            .unwrap_or_else(|| PasswordPolicy::default(user.realm_id.into()));

        let email_local_buf = user
            .email
            .as_deref()
            .and_then(|e| e.split('@').next())
            .map(str::to_string);

        validator::validate(
            &input.value,
            &policy,
            Some(user.username.as_str()),
            email_local_buf.as_deref(),
        )
        .map_err(violations_to_core_error)?;

        let password_credential = self
            .credential_repository
            .get_password_credential(user.id)
            .await;

        if password_credential.is_ok() {
            self.credential_repository
                .delete_password_credential(user.id)
                .await
                .map_err(|_| CoreError::DeleteCredentialError)?;
        }

        let hash_result = self
            .hasher_repository
            .hash_password(&input.value)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(user.id, "password".into(), hash_result, "".into(), false)
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        self.user_required_action_repository
            .remove_required_action(user.id, RequiredAction::UpdatePassword)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.token_revocation
            .revoke_all_user_access(user.id, user.realm_id.into())
            .await?;

        Ok(())
    }

    async fn verify_otp(
        &self,
        identity: Identity,
        input: VerifyOtpInput,
    ) -> Result<VerifyOtpOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        // 1. Replacing an existing authenticator requires either the server to
        //    have asked for it (the `ConfigureOtp` required action, set by the
        //    login flow) or a fresh proof of knowledge (a step-up token minted
        //    by `/me/reauthenticate`, presented by the self-service flow). The
        //    login-flow `/login-actions/verify-otp` passes no token, so it can
        //    never opt out of this guard by omitting it.
        let existing_credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;
        let existing_otp_credentials = existing_credentials
            .iter()
            .filter(|credential| credential.credential_type == CredentialType::Otp)
            .collect::<Vec<_>>();

        if !existing_otp_credentials.is_empty()
            && !user
                .required_actions
                .contains(&RequiredAction::ConfigureOtp)
            && input.step_up_token.is_none()
        {
            return Err(CoreError::Forbidden(
                "OTP is already configured for this user".into(),
            ));
        }

        // 2. Consume the step-up token only after the guard: a rejected request
        //    (403) must not burn it, otherwise the user would have to redo
        //    `/me/reauthenticate` for every attempt. From here on the token is
        //    spent, so the replacement below is bound to a fresh proof of
        //    knowledge.
        if let Some(step_up_token) = &input.step_up_token {
            self.consume_step_up_token(user.id, step_up_token).await?;
        }

        // 3. Read the newest active enrollment persisted by `/me/totp/setup`.
        //    The caller can never supply their own secret, so an attacker
        //    cannot silently replace the victim's authenticator.
        let enrollment = self
            .otp_enrollment_repository
            .get_active_enrollment(user.id, Utc::now())
            .await
            .map_err(|e| {
                error!("Failed to load OTP enrollment: {e:?}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::PendingTotpSecretMissing)?;

        let secret = TotpSecret::from_base32(&enrollment.secret);

        // 4. Verify the code before claiming the enrollment. A mistyped digit
        //    must not destroy the user's enrollment attempt.
        let is_valid = verify(&secret, &input.code)?;
        if !is_valid {
            error!(user_id = %user.id, "invalid OTP code during TOTP enrollment");
            return Err(CoreError::InvalidOtpCode);
        }

        let claimed = self
            .otp_enrollment_repository
            .claim_enrollment(enrollment.id, Utc::now())
            .await
            .map_err(|e| {
                error!("Failed to claim OTP enrollment: {e:?}");
                CoreError::InternalServerError
            })?;
        if !claimed {
            return Err(CoreError::PendingTotpSecretMissing);
        }

        let credential_data = serde_json::json!({
          "subType": "totp",
          "digits": 6,
          "counter": 0,
          "period": 30,
          "algorithm": "SHA1",
        });

        // 4. Replace any existing OTP credential with the newly verified one.
        for cred in existing_otp_credentials {
            self.credential_repository
                .delete_by_id(cred.id)
                .await
                .map_err(|e| {
                    error!(
                        user_id = %user.id,
                        credential_id = %cred.id,
                        "Failed to delete existing OTP credential before re-enrollment: {e:?}"
                    );
                    CoreError::DeleteCredentialError
                })?;
        }

        self.credential_repository
            .create_custom_credential(
                user.id,
                "otp".to_string(),
                secret.base32_encoded().to_string(),
                None,
                credential_data,
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if let Err(e) = self
            .user_required_action_repository
            .remove_required_action(user.id, RequiredAction::ConfigureOtp)
            .await
        {
            match e {
                RequiredActionError::NotFound => {
                    debug!(
                        user_id = %user.id,
                        "ConfigureOtp required action was already absent after OTP setup"
                    );
                }
                other => {
                    warn!(
                        user_id = %user.id,
                        "Failed to remove ConfigureOtp required action after OTP setup: {other:?}"
                    );
                }
            }
        }

        // 5. Audit the enrollment so a compromised token cannot silently add
        //    a factor.
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    user.realm_id,
                    SecurityEventType::MfaEnrolled,
                    EventStatus::Success,
                    user.id,
                )
                .with_target("user".to_string(), user.id, Some("otp".to_string())),
            )
            .await
            .inspect_err(|e| warn!("Failed to log MFA enrollment event: {e}"));

        // 6. Notify the account owner that a new authenticator was added.
        self.notify_factor_change(user.id, user.realm_id, "TOTP", "enrolled")
            .await;

        Ok(VerifyOtpOutput {
            message: "OTP verified successfully".to_string(),
            user_id: user.id,
        })
    }

    async fn generate_magic_link(&self, input: MagicLinkInput) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InvalidRealm)?;

        let settings = self
            .realm_repository
            .get_realm_settings(realm.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::MagicLinkNotEnabled)?;

        if !settings.magic_link_enabled {
            return Err(CoreError::MagicLinkNotEnabled);
        }

        let user = match self
            .user_repository
            .get_by_email(&input.email, realm.id)
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!("User not found for magic link generation");
                return Ok(());
            }
            Err(e) => {
                error!("Failed to look up user during magic link generation: {}", e);
                return Ok(()); // Valid on purpose to avoid leaking email existence
            }
        };
        self.magic_link_repository
            .cleanup_expired(realm.id.into())
            .await?;
        let magic_token_id = generate_uuid_v7();
        let magic_token = generate_random_token();
        let magic_token_hash = self
            .hasher_repository
            .hash_magic_token(&magic_token)
            .await
            .map_err(|_| CoreError::InternalServerError)?;
        let ttl_minutes = settings.magic_link_ttl;
        let expires_at = Utc::now() + Duration::minutes(ttl_minutes as i64);
        let auth_session_code = input
            .session_code
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok());

        self.magic_link_repository
            .create_magic_link(
                user.id,
                realm.id.into(),
                magic_token_id,
                &magic_token_hash,
                expires_at,
                auth_session_code,
            )
            .await?;

        let template_id = realm
            .settings
            .as_ref()
            .and_then(|s| s.magic_link_template_id);

        match (
            template_id,
            self.smtp_config_repository.get_by_realm_id(realm.id).await,
        ) {
            (Some(tid), Ok(Some(smtp_config))) => {
                let magic_link_url = format!(
                    "{}/realms/{}/authentication/magic-link?token_id={}&magic_token={}",
                    input.base_url, realm.name, magic_token_id, magic_token
                );
                let body = format!(
                    "Click the link below to sign in:\n{magic_link_url}\n\nThis link expires in {ttl_minutes} minutes.\n\nIf you did not request this, please ignore this email.",
                );

                let html_body = self
                    .render_email_template(
                        realm.id.into(),
                        tid,
                        &user,
                        &[
                            ("magic_link", magic_link_url.as_str()),
                            ("expiration", &format!("{ttl_minutes} minutes")),
                        ],
                    )
                    .await
                    .ok();

                match self
                    .email_port
                    .send_email(
                        &smtp_config,
                        user.email.as_deref().unwrap_or(""),
                        "Your magic link",
                        &body,
                        html_body,
                    )
                    .await
                {
                    Ok(()) => {
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailSent,
                                    EventStatus::Success,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "template_id": tid.to_string(),
                                    "email_type": "magic_link",
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| warn!("Failed to log email sent event: {}", e));
                    }
                    Err(e) => {
                        warn!("Failed to send magic link email: {}", e);
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailNotSent,
                                    EventStatus::Failure,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "reason": format!("Failed to send magic link email: {}", e),
                                    "email_type": "magic_link",
                                    "error_code": "SMTP_SEND_FAILED",
                                    "template_id": tid.to_string(),
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
                    }
                }
            }
            (None, _) => {
                warn!(
                    "No magic link email template configured for realm {}",
                    realm.name
                );
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::EmailNotSent,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_details(serde_json::json!({
                            "reason": format!("No magic_link email template configured for realm {}", realm.name),
                            "email_type": "magic_link",
                            "error_code": "TEMPLATE_NOT_CONFIGURED",
                            "user_id": user.id.to_string(),
                        })),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
            }
            (_, _) => {
                warn!("SMTP not configured for realm, logging magic link instead");
                debug!(
                    "Magic link URL: {}/realms/{}/authentication/magic-link?token_id={}&magic_token={}",
                    input.base_url, realm.name, magic_token_id, magic_token
                );
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::EmailNotSent,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_details(serde_json::json!({
                            "reason": format!("SMTP not configured for realm {}", realm.name),
                            "email_type": "magic_link",
                            "error_code": "SMTP_NOT_CONFIGURED",
                            "user_id": user.id.to_string(),
                        })),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
            }
        }

        Ok(())
    }

    async fn verify_magic_link(&self, input: VerifyMagicLinkInput) -> Result<String, CoreError> {
        let magic_link = self
            .magic_link_repository
            .get_by_token_id(input.magic_token_id)
            .await
            .inspect_err(|e| error!("Failed to retrieve magic link: {}", e))?
            .ok_or_else(|| {
                warn!(
                    "Magic link not found for token_id: {}",
                    input.magic_token_id
                );
                CoreError::InvalidMagicLink
            })?;

        // Use the session code stored at send time so that the user is redirected to
        // the correct client (e.g. an external React app) and not to the fallback
        // `security-admin-console` that would be created by the OAuth redirect loop.
        let session_code = magic_link.auth_session_code.ok_or_else(|| {
            warn!("Magic link has no associated auth session code");
            CoreError::SessionNotFound
        })?;

        // Fetch the auth session
        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .inspect_err(|_| error!("Session not found for code: {}", session_code))
            .map_err(|_| CoreError::SessionNotFound)?;

        if magic_link.realm_id != Uuid::from(auth_session.realm_id) {
            warn!(
                "Magic link realm_id {} does not match auth session realm_id {}",
                magic_link.realm_id,
                Uuid::from(auth_session.realm_id)
            );
            return Err(CoreError::InvalidMagicLink);
        }

        if magic_link.is_expired() {
            warn!("Magic link has expired");
            self.magic_link_repository
                .delete_by_token_id(magic_link.magic_token_id)
                .await
                .inspect_err(|e| error!("Failed to delete magic link : {}", e))
                .map_err(|_| CoreError::InternalServerError)?;
            return Err(CoreError::MagicLinkExpired);
        }
        let is_valid = self
            .hasher_repository
            .verify_magic_token(&input.magic_token, &magic_link.magic_token_hash)
            .await
            .map_err(|e| {
                error!("Token verification failed: {}", e);
                CoreError::InternalServerError
            })?;
        if !is_valid {
            warn!("Magic token verification failed");
            let _ = self
                .magic_link_repository
                .delete_by_token_id(magic_link.magic_token_id)
                .await
                .inspect_err(|e| {
                    warn!(
                        "Failed to delete magic link after failed verification: {}",
                        e
                    )
                });
            return Err(CoreError::InvalidMagicLink);
        }

        // Generate authorization code and login URL
        let login_url = self
            .store_auth_code_and_generate_login_url(
                &auth_session,
                magic_link.user_id,
                &[RequiredAction::VerifyEmail],
            )
            .await
            .inspect_err(|e| error!("Failed to generate login URL: {}", e))?;

        // TODO: here an email should be sent to the user instead of logging it
        debug!("Magic link verified for user_id: {}", magic_link.user_id);
        // Delete the used magic link
        let _ = self
            .magic_link_repository
            .delete_by_token_id(magic_link.magic_token_id)
            .await
            .inspect_err(|e| warn!("Failed to delete used magic link: {}", e));

        Ok(login_url)
    }

    async fn request_password_reset(
        &self,
        input: RequestPasswordResetInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InvalidRealm)?;

        let settings = self
            .realm_repository
            .get_realm_settings(realm.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::NotFound)?;

        if !settings.forgot_password_enabled {
            return Err(CoreError::Forbidden(
                "Password reset is not enabled for this realm".to_string(),
            ));
        }

        let user = match self
            .user_repository
            .get_by_email(&input.email, realm.id)
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!("User not found for password reset request");
                return Ok(()); // Don't leak email existence
            }
            Err(e) => {
                error!("Failed to look up user during password reset: {}", e);
                return Ok(());
            }
        };

        // Rate limit: max 3 active tokens per user
        let active_count = self
            .password_reset_token_repository
            .count_active_by_user_id(user.id)
            .await?;

        if active_count >= 3 {
            warn!("Too many active password reset tokens for user {}", user.id);
            return Ok(());
        }

        // Cleanup expired tokens
        self.password_reset_token_repository
            .cleanup_expired()
            .await?;

        let token_id = generate_uuid_v7();
        let raw_token = generate_random_token();
        let token_hash = self
            .hasher_repository
            .hash_magic_token(&raw_token)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let ttl_minutes = 30i64;
        let expires_at = Utc::now() + Duration::minutes(ttl_minutes);

        let auth_session_code = input
            .session_code
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok());

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

        self.password_reset_token_repository.create(&prt).await?;

        let template_id = realm
            .settings
            .as_ref()
            .and_then(|s| s.reset_password_template_id);

        match (
            template_id,
            self.smtp_config_repository.get_by_realm_id(realm.id).await,
        ) {
            (Some(tid), Ok(Some(smtp_config))) => {
                let reset_link = format!(
                    "{}/realms/{}/authentication/reset-password?token_id={}&token={}",
                    input.base_url, realm.name, token_id, raw_token
                );
                let body = format!(
                    "A password reset was requested for your account.\n\nClick the link below to reset your password:\n{reset_link}\n\nThis link expires in {ttl_minutes} minutes.\n\nIf you did not request this, please ignore this email.",
                );

                let html_body = self
                    .render_email_template(
                        realm.id.into(),
                        tid,
                        &user,
                        &[
                            ("reset_link", reset_link.as_str()),
                            ("expiration", &format!("{ttl_minutes} minutes")),
                        ],
                    )
                    .await
                    .ok();

                match self
                    .email_port
                    .send_email(
                        &smtp_config,
                        user.email.as_deref().unwrap_or(""),
                        "Reset your password",
                        &body,
                        html_body,
                    )
                    .await
                {
                    Ok(()) => {
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailSent,
                                    EventStatus::Success,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "template_id": tid.to_string(),
                                    "email_type": "reset_password",
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| warn!("Failed to log email sent event: {}", e));
                    }
                    Err(e) => {
                        warn!("Failed to send password reset email: {}", e);
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailNotSent,
                                    EventStatus::Failure,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "reason": format!("Failed to send password reset email: {}", e),
                                    "email_type": "reset_password",
                                    "error_code": "SMTP_SEND_FAILED",
                                    "template_id": tid.to_string(),
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
                    }
                }
            }
            (None, _) => {
                warn!(
                    "No reset password email template configured for realm {}",
                    realm.name
                );
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::EmailNotSent,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_details(serde_json::json!({
                            "reason": format!("No reset_password email template configured for realm {}", realm.name),
                            "email_type": "reset_password",
                            "error_code": "TEMPLATE_NOT_CONFIGURED",
                            "user_id": user.id.to_string(),
                        })),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
            }
            (_, _) => {
                warn!("SMTP not configured for realm, logging password reset token instead");
                debug!(
                    "Password reset token_id: {}, token: {}",
                    token_id, raw_token
                );
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::EmailNotSent,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_details(serde_json::json!({
                            "reason": format!("SMTP not configured for realm {}", realm.name),
                            "email_type": "reset_password",
                            "error_code": "SMTP_NOT_CONFIGURED",
                            "user_id": user.id.to_string(),
                        })),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
            }
        }

        // Log SeaWatch event
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::PasswordResetRequested,
                    EventStatus::Success,
                    user.id,
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await
            .inspect_err(|e| warn!("Failed to log password reset requested event: {}", e));

        Ok(())
    }

    async fn complete_password_reset(
        &self,
        input: CompletePasswordResetInput,
    ) -> Result<CompletePasswordResetOutput, CoreError> {
        // 1. Get token by token_id
        let prt = self
            .password_reset_token_repository
            .get_by_token_id(input.token_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // 2. Verify not expired + Argon2 verify
        if prt.is_expired() {
            self.password_reset_token_repository
                .delete_by_token_id(input.token_id)
                .await?;
            return Err(CoreError::ExpiredToken);
        }

        let is_valid = self
            .hasher_repository
            .verify_magic_token(&input.token, &prt.token_hash)
            .await
            .map_err(|e| {
                error!("Token verification failed: {}", e);
                CoreError::InternalServerError
            })?;

        if !is_valid {
            let _ = self
                .password_reset_token_repository
                .delete_by_token_id(input.token_id)
                .await;
            return Err(CoreError::InvalidToken);
        }

        // 3. Enforce password policy before applying the new credential.
        let policy = self
            .password_policy_repository
            .find_by_realm_id(prt.realm_id)
            .await?
            .unwrap_or_else(|| PasswordPolicy::default(prt.realm_id));

        // Look up user context for the common-password check (username/email match).
        let target_user = self.user_repository.get_by_id(prt.user_id).await.ok();

        let (username_buf, email_local_buf);
        let (username_ref, email_local_ref) = if let Some(ref u) = target_user {
            username_buf = u.username.clone();
            email_local_buf = u
                .email
                .as_deref()
                .and_then(|e| e.split('@').next())
                .map(str::to_string);
            (Some(username_buf.as_str()), email_local_buf.as_deref())
        } else {
            (None, None)
        };

        validator::validate(&input.new_password, &policy, username_ref, email_local_ref)
            .map_err(violations_to_core_error)?;

        // 4. Delete old password credential
        let _ = self
            .credential_repository
            .delete_password_credential(prt.user_id)
            .await;

        // 5. Create new hashed credential
        let hash_result = self
            .hasher_repository
            .hash_password(&input.new_password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(
                prt.user_id,
                "password".into(),
                hash_result,
                "".into(),
                false,
            )
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        let user_id = prt.user_id;
        let realm_id = prt.realm_id;
        let auth_session_code = prt.auth_session_code;

        // 5. Delete all reset tokens for this user
        self.password_reset_token_repository
            .delete_all_by_user_id(user_id)
            .await?;

        // 6. Remove UpdatePassword from required_actions if present
        let _ = self
            .user_required_action_repository
            .remove_required_action(user_id, RequiredAction::UpdatePassword)
            .await
            .inspect_err(|e| warn!("Failed to remove UpdatePassword required action: {}", e));

        self.token_revocation
            .revoke_all_user_access(user_id, realm_id)
            .await?;

        let realm_id_typed: RealmId = realm_id.into();

        // 7. Log SeaWatch PasswordResetCompleted
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id_typed,
                    SecurityEventType::PasswordResetCompleted,
                    EventStatus::Success,
                    user_id,
                )
                .with_target("user".to_string(), user_id, None),
            )
            .await
            .inspect_err(|e| warn!("Failed to log password reset completed event: {}", e));

        // 8. Emit webhook auth.reset_password
        let _ = self
            .webhook_repository
            .notify(
                realm_id_typed,
                WebhookPayload::new(WebhookTrigger::AuthResetPassword, user_id, None::<()>),
            )
            .await
            .inspect_err(|e| warn!("Failed to emit password reset webhook: {}", e));

        let login_url = if let Some(session_code) = auth_session_code {
            match self
                .auth_session_repository
                .get_by_session_code(session_code)
                .await
            {
                Ok(auth_session) if Uuid::from(auth_session.realm_id) == realm_id => {
                    match self
                        .store_auth_code_and_generate_login_url(&auth_session, user_id, &[])
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
    }

    async fn verify_reset_token(&self, input: VerifyResetTokenInput) -> Result<(), CoreError> {
        let prt = self
            .password_reset_token_repository
            .get_by_token_id(input.token_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if prt.is_expired() {
            self.password_reset_token_repository
                .delete_by_token_id(input.token_id)
                .await?;
            return Err(CoreError::ExpiredToken);
        }

        Ok(())
    }

    async fn passkey_register_options_self_service(
        &self,
        identity: Identity,
        input: PasskeyRegisterOptionsSelfServiceInput,
    ) -> Result<WebAuthnPublicKeyCreateOptionsOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let webauthn = build_webauthn_client(input.rp_info)?;

        let credentials = self
            .credential_repository
            .get_webauthn_public_key_credentials(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let credentials = {
            let filtered = credentials
                .into_iter()
                .filter_map(|v| v.webauthn_credential_id)
                .collect::<Vec<CredentialID>>();
            if filtered.is_empty() {
                None
            } else {
                let _ = self
                    .user_required_action_repository
                    .remove_required_action(user.id, RequiredAction::ConfigurePasskey)
                    .await;
                Some(filtered)
            }
        };

        let (ccr, pr) = webauthn
            .start_passkey_registration(
                user.id,
                user.email.as_deref().unwrap_or(""),
                &user.username,
                credentials,
            )
            .map_err(|e| {
                error!("Failed to generate webauthn challenge: {e:?}");
                CoreError::InternalServerError
            })?;

        // Drop any expired challenges so the table does not grow unbounded.
        let _ = self.webauthn_challenge_repository.cleanup_expired().await;
        store_pending_registration(
            self.webauthn_challenge_repository.as_ref(),
            user.id,
            WebAuthnChallenge::Registration(pr),
        )
        .await?;

        Ok(WebAuthnPublicKeyCreateOptionsOutput(ccr))
    }

    async fn passkey_register_self_service(
        &self,
        identity: Identity,
        input: PasskeyRegisterSelfServiceInput,
    ) -> Result<WebAuthnValidatePublicKeyOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        // The caller must present a valid step-up token minted by
        // `/me/reauthenticate`, so a stolen access token cannot silently add a
        // new factor.
        self.consume_step_up_token(user.id, &input.step_up_token)
            .await?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let pending =
            take_pending_registration(self.webauthn_challenge_repository.as_ref(), user.id).await?;

        let passkey = match pending {
            WebAuthnChallenge::Registration(ref pr) => webauthn
                .finish_passkey_registration(&input.credential, pr)
                .map_err(|e| {
                    debug!("Failed to complete passkey registration: {e:?}");
                    CoreError::Invalid
                }),
            _ => Err(CoreError::Invalid),
        }?;

        self.credential_repository
            .create_webauthn_credential(user.id, passkey)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let _ = self
            .user_required_action_repository
            .remove_required_action(user.id, RequiredAction::ConfigurePasskey)
            .await;

        // Audit the enrollment so a compromised token (even one that passed the
        // step-up check) cannot silently add a passkey without a trail.
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    user.realm_id,
                    SecurityEventType::MfaEnrolled,
                    EventStatus::Success,
                    user.id,
                )
                .with_target(
                    "user".to_string(),
                    user.id,
                    Some("passkey".to_string()),
                ),
            )
            .await
            .inspect_err(|e| warn!("Failed to log MFA enrollment event: {e}"));

        // Notify the account owner that a new passkey was added.
        self.notify_factor_change(user.id, user.realm_id, "passkey", "enrolled")
            .await;

        Ok(WebAuthnValidatePublicKeyOutput {})
    }

    async fn complete_password_reset_with_recovery_code(
        &self,
        input: CompletePasswordResetWithRecoveryCodeInput,
    ) -> Result<CompletePasswordResetOutput, CoreError> {
        // 1. Resolve the realm and honour its forgot-password setting before
        //    anything else, exactly like `request_password_reset`: a realm that
        //    disabled password reset must not expose this path either.
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_settings = self
            .realm_repository
            .get_realm_settings(realm.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if !realm_settings.forgot_password_enabled {
            return Err(CoreError::Forbidden(
                "Password reset is not enabled for this realm".to_string(),
            ));
        }

        // 2. Resolve the user. An unknown email is answered with the same error
        //    as a bad code so this anonymous endpoint cannot be used to
        //    enumerate accounts (`request_password_reset` masks it the same way).
        let user = self
            .user_repository
            .get_by_email(&input.email, realm.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or_else(|| {
                CoreError::RecoveryCodeBurnError(
                    "The provided code is invalid or has already been used".to_string(),
                )
            })?;

        // 3. Enforce account lockout so the unauthenticated endpoint cannot be
        //    used as an unlimited recovery-code oracle (per-account rate limit).
        let lockout_threshold = realm_settings.lockout_threshold;
        let lockout_duration_seconds = realm_settings.lockout_duration_seconds;
        let now = Utc::now();
        if user.is_locked(now) {
            return Err(CoreError::AccountLocked);
        }

        // 3. Locate and burn the matching recovery code. We derive the fast
        //    lookup key from the submitted code and query a single candidate
        //    row, so only one Argon2 verification runs (instead of one per
        //    stored code) — this prevents a memory-hard DoS on this
        //    unauthenticated endpoint.
        let format =
            RecoveryCodeFormat::try_from(input.format).map_err(CoreError::RecoveryCodeBurnError)?;
        let user_code = decode_string(input.code, format)?;
        let lookup = self.recovery_code_repository.lookup_of(&user_code);

        let burnt_code = self
            .find_matching_recovery_code(user.id, &user_code, &lookup)
            .await?;

        let burnt_code = match burnt_code {
            Some(code) => code,
            None => {
                // Failed attempt: bump the lockout counter and emit a failure
                // event so SeaWatch can detect brute-force guessing.
                let locked_until = lockout_compute_locked_until(
                    user.failed_login_attempts + 1,
                    lockout_threshold,
                    lockout_duration_seconds,
                    now,
                );
                let _ = self
                    .user_repository
                    .increment_failed_login_attempts(user.id, locked_until)
                    .await;
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::RecoveryCodeBurned,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_target("user".to_string(), user.id, None),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log recovery code burn failure event: {e}"));
                return Err(CoreError::RecoveryCodeBurnError(
                    "The provided code is invalid or has already been used".to_string(),
                ));
            }
        };

        self.credential_repository
            .delete_by_id(burnt_code.id)
            .await
            .map_err(|e| {
                error!("Failed to delete a credential even though it was just fetched with the same repository: {e}");
                CoreError::InternalServerError
            })?;

        // 4b. A valid code proves possession, so clear any accumulated lockout
        //     counter from prior failed guesses.
        let _ = self
            .user_repository
            .reset_failed_login_attempts(user.id)
            .await;

        // 5. A recovery code is a *second* factor, not a standalone login path.
        //    Verifying it proves possession of the code, but we must still prove
        //    email control and not bypass MFA. So instead of minting a session
        //    here, we issue a password-reset token and email the user a reset
        //    link — the new password is only applied when they complete that
        //    link, exactly like the standard email-reset flow.
        let reset_token = self
            .issue_password_reset_token_and_notify(user.id, realm.id, &input.base_url)
            .await?;

        let realm_id = realm.id;

        // 6. Log the recovery-code burn (success) and the reset-email event.
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::RecoveryCodeBurned,
                    EventStatus::Success,
                    user.id,
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await
            .inspect_err(|e| warn!("Failed to log recovery code burned event: {e}"));

        // 7. Emit webhook auth.reset_password.
        let _ = self
            .webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(WebhookTrigger::AuthResetPassword, user.id, None::<()>),
            )
            .await
            .inspect_err(|e| warn!("Failed to emit password reset webhook: {e}"));

        // The reset link is returned so the caller can surface it / the email
        // carries the canonical link. No tokens or cookies are minted here.
        Ok(CompletePasswordResetOutput {
            user_id: user.id,
            realm_id: Uuid::from(realm_id),
            login_url: reset_token,
        })
    }

    async fn reauthenticate(
        &self,
        identity: Identity,
        mut input: ReauthenticateInput,
    ) -> Result<ReauthenticateOutput, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        // 0. Enforce account lockout so a stolen low-value token cannot be
        //    used as an unlimited password/OTP oracle.
        let realm_settings = self
            .realm_repository
            .get_realm_settings(user.realm_id)
            .await?;
        let lockout_threshold = realm_settings
            .as_ref()
            .map(|s| s.lockout_threshold)
            .unwrap_or(10);
        let lockout_duration_seconds = realm_settings
            .as_ref()
            .map(|s| s.lockout_duration_seconds)
            .unwrap_or(900);
        let now = Utc::now();
        if user.is_locked(now) {
            return Err(CoreError::AccountLocked);
        }

        // 1. The account password must always be correct.
        let password_cred = self
            .credential_repository
            .get_password_credential(user.id)
            .await
            .map_err(|_| CoreError::InvalidPassword)?;

        let salt = password_cred.salt.ok_or(CoreError::InvalidPassword)?;
        let CredentialData::Hash {
            hash_iterations,
            algorithm,
        } = password_cred.credential_data
        else {
            return Err(CoreError::InvalidPassword);
        };

        let valid = self
            .hasher_repository
            .verify_password(
                &input.password,
                &password_cred.secret_data,
                hash_iterations,
                &algorithm,
                &salt,
            )
            .await
            .map_err(|_| CoreError::InvalidPassword)?;

        if !valid {
            let locked_until = lockout_compute_locked_until(
                user.failed_login_attempts + 1,
                lockout_threshold,
                lockout_duration_seconds,
                now,
            );
            let _ = self
                .user_repository
                .increment_failed_login_attempts(user.id, locked_until)
                .await;
            self.emit_reauthentication_failed(user.realm_id, user.id)
                .await;
            input.password.zeroize();
            return Err(CoreError::InvalidPassword);
        }

        // The password has been verified; scrub the plaintext copy before we
        // move on to the (non-sensitive) OTP step.
        input.password.zeroize();

        // 2. When an authenticator is configured, the current OTP code is required too.
        let credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if let Some(otp_cred) = credentials
            .iter()
            .find(|c| c.credential_type == CredentialType::Otp)
        {
            let code = input.otp_code.take().ok_or_else(|| {
                CoreError::TotpVerificationFailed("OTP code is required".to_string())
            })?;

            let secret = TotpSecret::from_base32(&otp_cred.secret_data);
            if !verify(&secret, &code)? {
                error!(
                    "invalid OTP code during reauthentication for user: {}",
                    user.id
                );
                let locked_until = lockout_compute_locked_until(
                    user.failed_login_attempts + 1,
                    lockout_threshold,
                    lockout_duration_seconds,
                    now,
                );
                let _ = self
                    .user_repository
                    .increment_failed_login_attempts(user.id, locked_until)
                    .await;
                self.emit_reauthentication_failed(user.realm_id, user.id)
                    .await;
                return Err(CoreError::TotpVerificationFailed(
                    "failed to verify OTP".to_string(),
                ));
            }
        }

        // Scrub any remaining sensitive material (OTP code, already-cleared password).
        input.zeroize();

        // 3. Success: reset the failure counter and mint a step-up token.
        let _ = self
            .user_repository
            .reset_failed_login_attempts(user.id)
            .await;
        // Drop any expired step-up tokens so the table does not grow unbounded.
        let _ = self.step_up_token_repository.cleanup_expired().await;
        let step_up_token = self.mint_step_up_token(user.id).await?;

        Ok(ReauthenticateOutput { step_up_token })
    }

    async fn list_credentials_self_service(
        &self,
        identity: Identity,
    ) -> Result<Vec<CredentialOverview>, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        let credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(credentials
            .into_iter()
            .map(CredentialOverview::from)
            .collect())
    }

    async fn delete_credential_self_service(
        &self,
        identity: Identity,
        credential_id: Uuid,
        step_up_token: String,
    ) -> Result<(), CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::Forbidden("is not user".to_string())),
        };

        // 0. The caller must present a valid step-up token minted by
        //    `/me/reauthenticate`, so a stolen access token cannot silently
        //    remove the victim's factors.
        self.consume_step_up_token(user.id, &step_up_token).await?;

        let credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if !credentials.iter().any(|c| c.id == credential_id) {
            return Err(CoreError::Forbidden(
                "credential does not belong to the user".to_string(),
            ));
        }

        let target = credentials
            .iter()
            .find(|c| c.id == credential_id)
            .ok_or_else(|| {
                CoreError::Forbidden("credential does not belong to the user".to_string())
            })?;

        // The password credential is the account's primary login path and must
        // not be removed through self-service (use the dedicated password-reset
        // flows instead).
        if target.credential_type == CredentialType::Password {
            return Err(CoreError::Forbidden(
                "the password credential cannot be removed via self-service".to_string(),
            ));
        }

        // Reject removal only when the target is a login factor (Password,
        // Otp or WebAuthn) and deleting it would leave the user with no login
        // factor at all. Recovery codes are not login factors — they only help
        // regain access — so deleting one can never lock the user out.
        let login_factors = credentials
            .iter()
            .filter(|c| c.credential_type != CredentialType::RecoveryCode)
            .count();
        if target.credential_type != CredentialType::RecoveryCode && login_factors <= 1 {
            return Err(CoreError::Forbidden(
                "cannot remove the last remaining credential".to_string(),
            ));
        }

        let is_second_factor = matches!(
            target.credential_type,
            CredentialType::Otp | CredentialType::WebAuthnPublicKeyCredential
        );
        let remaining_second_factors = credentials
            .iter()
            .filter(|c| c.id != credential_id)
            .filter(|c| {
                matches!(
                    c.credential_type,
                    CredentialType::Otp | CredentialType::WebAuthnPublicKeyCredential
                )
            })
            .count();
        let is_primary = target.credential_type != CredentialType::RecoveryCode;

        // Resolve the realm policy *before* mutating anything: if it mandates
        // MFA and the user is removing their last second factor, the relevant
        // required action must be queued so the assurance level is not silently
        // downgraded — and a transient DB error must not surface with the
        // credential already gone and no event/email emitted.
        let realm_settings = self
            .realm_repository
            .get_realm_settings(user.realm_id)
            .await?;
        let require_mfa = realm_settings.is_some_and(|s| s.require_mfa);

        self.credential_repository
            .delete_by_id(credential_id)
            .await
            .map_err(|_| CoreError::DeleteCredentialError)?;

        // 1. If the realm mandates MFA and the user just removed their last
        //    second factor, queue the matching re-enrolment action.
        if require_mfa && is_second_factor && remaining_second_factors == 0 {
            let required_action = match target.credential_type {
                CredentialType::Otp => Some(RequiredAction::ConfigureOtp),
                CredentialType::WebAuthnPublicKeyCredential => {
                    Some(RequiredAction::ConfigurePasskey)
                }
                _ => None,
            };
            if let Some(action) = required_action {
                let _ = self
                    .user_required_action_repository
                    .add_required_action(user.id, action)
                    .await;
            }
        }

        // 2. Audit the removal.
        let event_type = if is_primary {
            SecurityEventType::MfaRemoved
        } else {
            SecurityEventType::CredentialDeleted
        };
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(user.realm_id, event_type, EventStatus::Success, user.id)
                    .with_target("credential".to_string(), credential_id, None),
            )
            .await
            .inspect_err(|e| warn!("Failed to log credential deletion event: {e}"));

        // 3. Emit the user.credentials.deleted webhook so external systems can
        //    react to factor removal.
        let _ = self
            .webhook_repository
            .notify(
                user.realm_id,
                WebhookPayload::new(WebhookTrigger::UserDeleteCredentials, user.id, None::<()>),
            )
            .await
            .inspect_err(|e| warn!("Failed to emit credential-deleted webhook: {e}"));

        // 4. Notify the account owner that a sign-in method was removed.
        let factor = match target.credential_type {
            CredentialType::Otp => "TOTP",
            CredentialType::WebAuthnPublicKeyCredential => "passkey",
            _ => "credential",
        };
        self.notify_factor_change(user.id, user.realm_id, factor, "removed")
            .await;

        Ok(())
    }

    async fn realm_id_for_name(&self, realm_name: &str) -> Result<RealmId, CoreError> {
        self.realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)
            .map(|realm| realm.id)
    }
}

impl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH, ETR, TR, PPR, OER, URR, TRV, WCR, SUT>
    TridentServiceImpl<
        CR,
        RC,
        AS,
        H,
        URA,
        ML,
        UR,
        RR,
        ES,
        SC,
        PRT,
        SE,
        WH,
        ETR,
        TR,
        PPR,
        OER,
        URR,
        TRV,
        WCR,
        SUT,
    >
where
    CR: CredentialRepository,
    RC: RecoveryCodeRepository,
    AS: AuthSessionRepository,
    H: HasherRepository,
    URA: UserRequiredActionRepository,
    ML: MagicLinkRepository,
    UR: UserRepository,
    RR: RealmRepository,
    ES: EmailPort,
    SC: SmtpConfigRepository,
    PRT: PasswordResetTokenRepository,
    SE: SecurityEventRepository,
    WH: WebhookRepository,
    ETR: EmailTemplateRepository,
    TR: TemplateRenderer,
    PPR: PasswordPolicyRepository,
    OER: OtpEnrollmentRepository,
    URR: UserRoleRepository,
    TRV: TokenRevocationPort,
    WCR: WebAuthnChallengeRepository,
    SUT: StepUpTokenRepository,
{
    /// mirroring the standard email-reset flow. Used after a recovery code is
    /// burned so that a recovery code (a *second* factor) never mints a session
    /// on its own — the new password is only applied when the user completes
    /// the emailed link, which also re-proves email control and avoids MFA
    /// bypass. Returns the reset link (also emailed) so the caller can surface
    /// it; returns `None` when no SMTP/template is configured (the token is
    /// still created and logged).
    async fn issue_password_reset_token_and_notify(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        base_url: &str,
    ) -> Result<Option<String>, CoreError> {
        let user = self
            .user_repository
            .get_by_id(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let realm = self
            .realm_repository
            .get_by_id(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InvalidRealm)?;

        // Rate limit: keep the active-token count bounded.
        let active_count = self
            .password_reset_token_repository
            .count_active_by_user_id(user.id)
            .await?;
        if active_count >= 3 {
            warn!("Too many active password reset tokens for user {}", user.id);
        } else {
            self.password_reset_token_repository
                .cleanup_expired()
                .await?;
        }

        let token_id = generate_uuid_v7();
        let raw_token = generate_random_token();
        let token_hash = self
            .hasher_repository
            .hash_magic_token(&raw_token)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let ttl_minutes = 30i64;
        let expires_at = Utc::now() + Duration::minutes(ttl_minutes);

        let prt = PasswordResetToken {
            id: generate_uuid_v7(),
            user_id: user.id,
            realm_id: realm.id.into(),
            token_id,
            token_hash: token_hash.hash,
            created_at: Utc::now(),
            expires_at,
            auth_session_code: None,
        };

        self.password_reset_token_repository.create(&prt).await?;

        let reset_link = format!(
            "{}/realms/{}/authentication/reset-password?token_id={}&token={}",
            base_url, realm.name, token_id, raw_token
        );

        let template_id = realm
            .settings
            .as_ref()
            .and_then(|s| s.reset_password_template_id);

        match (
            template_id,
            self.smtp_config_repository.get_by_realm_id(realm.id).await,
        ) {
            (Some(tid), Ok(Some(smtp_config))) => {
                let body = format!(
                    "A password reset was requested for your account.\n\nClick the link below to reset your password:\n{reset_link}\n\nThis link expires in {ttl_minutes} minutes.\n\nIf you did not request this, please ignore this email."
                );

                let html_body = self
                    .render_email_template(
                        realm.id.into(),
                        tid,
                        &user,
                        &[
                            ("reset_link", reset_link.as_str()),
                            ("expiration", &format!("{ttl_minutes} minutes")),
                        ],
                    )
                    .await
                    .ok();

                match self
                    .email_port
                    .send_email(
                        &smtp_config,
                        user.email.as_deref().unwrap_or(""),
                        "Reset your password",
                        &body,
                        html_body,
                    )
                    .await
                {
                    Ok(()) => {
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailSent,
                                    EventStatus::Success,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "template_id": tid.to_string(),
                                    "email_type": "reset_password",
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| warn!("Failed to log email sent event: {}", e));
                    }
                    Err(e) => {
                        warn!("Failed to send password reset email: {}", e);
                        let _ = self
                            .security_event_repository
                            .store_event(
                                SecurityEvent::new(
                                    realm.id,
                                    SecurityEventType::EmailNotSent,
                                    EventStatus::Failure,
                                    user.id,
                                )
                                .with_details(serde_json::json!({
                                    "reason": format!("Failed to send password reset email: {}", e),
                                    "email_type": "reset_password",
                                    "error_code": "SMTP_SEND_FAILED",
                                    "template_id": tid.to_string(),
                                    "user_id": user.id.to_string(),
                                })),
                            )
                            .await
                            .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
                    }
                }
            }
            (None, _) => {
                warn!(
                    "No reset password email template configured for realm {}",
                    realm.name
                );
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::EmailNotSent,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_details(serde_json::json!({
                            "reason": format!("No reset_password email template configured for realm {}", realm.name),
                            "email_type": "reset_password",
                            "error_code": "TEMPLATE_NOT_CONFIGURED",
                            "user_id": user.id.to_string(),
                        })),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
            }
            (_, _) => {
                warn!("SMTP not configured for realm, logging password reset token instead");
                debug!(
                    "Password reset token_id: {}, token: {}",
                    token_id, raw_token
                );
                let _ = self
                    .security_event_repository
                    .store_event(
                        SecurityEvent::new(
                            realm.id,
                            SecurityEventType::EmailNotSent,
                            EventStatus::Failure,
                            user.id,
                        )
                        .with_details(serde_json::json!({
                            "reason": format!("SMTP not configured for realm {}", realm.name),
                            "email_type": "reset_password",
                            "error_code": "SMTP_NOT_CONFIGURED",
                            "user_id": user.id.to_string(),
                        })),
                    )
                    .await
                    .inspect_err(|e| warn!("Failed to log email not sent event: {}", e));
            }
        }

        // Log SeaWatch PasswordResetRequested event.
        let _ = self
            .security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::PasswordResetRequested,
                    EventStatus::Success,
                    user.id,
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await
            .inspect_err(|e| warn!("Failed to log password reset requested event: {}", e));

        Ok(Some(reset_link))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        authentication::{
            entities::{AuthSession, AuthSessionParams, AuthenticationError},
            ports::MockAuthSessionRepository,
        },
        common::{email::MockEmailPort, services::tests::create_test_realm_with_name},
        credential::{entities::CredentialError, ports::MockCredentialRepository},
        email_template::ports::MockEmailTemplateRepository,
        password_policy::repository::MockPasswordPolicyRepository,
        realm::ports::{MockRealmRepository, MockSmtpConfigRepository},
        seawatch::ports::MockSecurityEventRepository,
        session::ports::MockTokenRevocationPort,
        trident::{
            entities::MagicLink,
            ports::{
                MockMagicLinkRepository, MockOtpEnrollmentRepository,
                MockPasswordResetTokenRepository, MockRecoveryCodeRepository,
                MockStepUpTokenRepository, MockWebAuthnChallengeRepository, OtpEnrollment,
            },
        },
        user::ports::{
            MockUserRepository, MockUserRequiredActionRepository, MockUserRoleRepository,
        },
        webhook::ports::MockWebhookRepository,
    };
    use chrono::DateTime;
    use ferriskey_domain::realm::{RealmSetting, SmtpConfig, SmtpEncryption};
    use ferriskey_security::crypto::{entities::HashResult, ports::MockHasherRepository};
    use std::sync::Mutex;

    #[derive(Debug, Clone)]
    struct NoopTemplateRenderer;

    impl TemplateRenderer for NoopTemplateRenderer {
        fn render_to_intermediate(
            &self,
            _structure: &serde_json::Value,
        ) -> Result<String, CoreError> {
            Ok(String::new())
        }

        fn render_to_html(&self, _intermediate: &str) -> Result<String, CoreError> {
            Ok(String::new())
        }
    }

    type TestTridentService = TridentServiceImpl<
        MockCredentialRepository,
        MockRecoveryCodeRepository,
        MockAuthSessionRepository,
        MockHasherRepository,
        MockUserRequiredActionRepository,
        MockMagicLinkRepository,
        MockUserRepository,
        MockRealmRepository,
        MockEmailPort,
        MockSmtpConfigRepository,
        MockPasswordResetTokenRepository,
        MockSecurityEventRepository,
        MockWebhookRepository,
        MockEmailTemplateRepository,
        NoopTemplateRenderer,
        MockPasswordPolicyRepository,
        MockOtpEnrollmentRepository,
        MockUserRoleRepository,
        MockTokenRevocationPort,
        MockWebAuthnChallengeRepository,
        MockStepUpTokenRepository,
    >;

    /// `(user_id, secret, expires_at)` as handed to `start_enrollment`.
    type CapturedEnrollment = Arc<Mutex<Option<(Uuid, String, DateTime<Utc>)>>>;

    struct TridentTestBuilder {
        credential_repo: Arc<MockCredentialRepository>,
        recovery_code_repo: Arc<MockRecoveryCodeRepository>,
        auth_session_repo: Arc<MockAuthSessionRepository>,
        hasher_repo: Arc<MockHasherRepository>,
        user_required_action_repo: Arc<MockUserRequiredActionRepository>,
        magic_link_repo: Arc<MockMagicLinkRepository>,
        user_repo: Arc<MockUserRepository>,
        realm_repo: Arc<MockRealmRepository>,
        email_port: Arc<MockEmailPort>,
        smtp_config_repo: Arc<MockSmtpConfigRepository>,
        prt_repo: Arc<MockPasswordResetTokenRepository>,
        security_event_repo: Arc<MockSecurityEventRepository>,
        webhook_repo: Arc<MockWebhookRepository>,
        email_template_repo: Arc<MockEmailTemplateRepository>,
        template_renderer: Arc<NoopTemplateRenderer>,
        password_policy_repo: Arc<MockPasswordPolicyRepository>,
        otp_enrollment_repo: Arc<MockOtpEnrollmentRepository>,
        user_role_repo: Arc<MockUserRoleRepository>,
        token_revocation: Arc<MockTokenRevocationPort>,
        webauthn_challenge_repo: Arc<MockWebAuthnChallengeRepository>,
        step_up_token_repo: Arc<MockStepUpTokenRepository>,
    }

    impl TridentTestBuilder {
        fn new() -> Self {
            Self {
                credential_repo: Arc::new(MockCredentialRepository::new()),
                recovery_code_repo: Arc::new(MockRecoveryCodeRepository::new()),
                auth_session_repo: Arc::new(MockAuthSessionRepository::new()),
                hasher_repo: Arc::new(MockHasherRepository::new()),
                user_required_action_repo: Arc::new(MockUserRequiredActionRepository::new()),
                magic_link_repo: Arc::new(MockMagicLinkRepository::new()),
                user_repo: Arc::new(MockUserRepository::new()),
                realm_repo: Arc::new(MockRealmRepository::new()),
                email_port: Arc::new(MockEmailPort::new()),
                smtp_config_repo: Arc::new(MockSmtpConfigRepository::new()),
                prt_repo: Arc::new(MockPasswordResetTokenRepository::new()),
                security_event_repo: Arc::new(MockSecurityEventRepository::new()),
                webhook_repo: Arc::new(MockWebhookRepository::new()),
                email_template_repo: Arc::new(MockEmailTemplateRepository::new()),
                template_renderer: Arc::new(NoopTemplateRenderer),
                password_policy_repo: Arc::new(MockPasswordPolicyRepository::new()),
                otp_enrollment_repo: Arc::new(MockOtpEnrollmentRepository::new()),
                user_role_repo: Arc::new(MockUserRoleRepository::new()),
                token_revocation: Arc::new(MockTokenRevocationPort::new()),
                webauthn_challenge_repo: Arc::new(MockWebAuthnChallengeRepository::new()),
                step_up_token_repo: Arc::new(MockStepUpTokenRepository::new()),
            }
        }

        fn with_user_access_revoked(mut self, times: usize) -> Self {
            Arc::get_mut(&mut self.token_revocation)
                .unwrap()
                .expect_revoke_all_user_access()
                .times(times)
                .returning(|_, _| Box::pin(async { Ok(()) }));
            self
        }

        fn build(self) -> TestTridentService {
            TridentServiceImpl::new(
                self.credential_repo,
                self.recovery_code_repo,
                self.auth_session_repo,
                self.hasher_repo,
                self.user_required_action_repo,
                self.magic_link_repo,
                self.user_repo,
                self.realm_repo,
                self.email_port,
                self.smtp_config_repo,
                self.prt_repo,
                self.security_event_repo,
                self.webhook_repo,
                self.email_template_repo,
                self.template_renderer,
                self.password_policy_repo,
                self.otp_enrollment_repo,
                self.user_role_repo,
                self.token_revocation,
                self.webauthn_challenge_repo,
                self.step_up_token_repo,
            )
        }
    }

    fn create_test_realm_setting(realm_id: RealmId, forgot_password_enabled: bool) -> RealmSetting {
        let mut settings = RealmSetting::new(realm_id, Some("RS256".to_string()));
        settings.forgot_password_enabled = forgot_password_enabled;
        settings
    }

    fn create_test_user_with_email(
        realm: &crate::domain::realm::entities::Realm,
        email: &str,
    ) -> crate::domain::user::entities::User {
        crate::domain::common::services::tests::create_test_user_with_params_and_realm(
            realm,
            "testuser",
            email.to_string(),
            true,
        )
    }

    // ── request_password_reset ──────────────────────────────────────────

    #[tokio::test]
    async fn request_password_reset_valid_email_returns_ok() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let settings = create_test_realm_setting(realm.id, true);
        let user = create_test_user_with_email(&realm, "user@example.com");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let settings_clone = settings.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        let user_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(move |_, _| {
                let u = user_clone.clone();
                Box::pin(async move { Ok(Some(u)) })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_count_active_by_user_id()
            .returning(|_| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_cleanup_expired()
            .returning(|| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_magic_token()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "hashed".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_create()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let user_by_id = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .request_password_reset(RequestPasswordResetInput {
                realm_name: "test-realm".to_string(),
                email: "user@example.com".to_string(),
                base_url: "http://localhost:5555".to_string(),
                session_code: None,
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn request_password_reset_unknown_email_returns_ok() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let settings = create_test_realm_setting(realm.id, true);

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let settings_clone = settings.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .request_password_reset(RequestPasswordResetInput {
                realm_name: "test-realm".to_string(),
                email: "unknown@example.com".to_string(),
                base_url: "http://localhost:5555".to_string(),
                session_code: None,
            })
            .await;

        // Must return Ok to avoid leaking email existence
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn request_password_reset_rate_limit_skips_token_creation() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let settings = create_test_realm_setting(realm.id, true);
        let user = create_test_user_with_email(&realm, "user@example.com");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let settings_clone = settings.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        let user_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(move |_, _| {
                let u = user_clone.clone();
                Box::pin(async move { Ok(Some(u)) })
            });

        // Already 3 active tokens → rate limited
        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_count_active_by_user_id()
            .returning(|_| Box::pin(async { Ok(3) }));

        // create() should NOT be called
        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_create()
            .never()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .request_password_reset(RequestPasswordResetInput {
                realm_name: "test-realm".to_string(),
                email: "user@example.com".to_string(),
                base_url: "http://localhost:5555".to_string(),
                session_code: None,
            })
            .await;

        // Returns Ok even when rate limited (no information leak)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn request_password_reset_disabled_returns_forbidden() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let settings = create_test_realm_setting(realm.id, false);

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let settings_clone = settings.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        let service = builder.build();
        let result = service
            .request_password_reset(RequestPasswordResetInput {
                realm_name: "test-realm".to_string(),
                email: "user@example.com".to_string(),
                base_url: "http://localhost:5555".to_string(),
                session_code: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn update_password_revokes_all_user_tokens() {
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");
        let user_id = user.id;

        let mut builder = TridentTestBuilder::new();

        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_password_credential()
            .returning(|_| Box::pin(async { Err(CredentialError::GetPasswordCredentialError) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_password()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "new_hash".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_credential()
            .returning(move |_, _, _, _, _| {
                let cred = Credential {
                    id: Uuid::new_v4(),
                    salt: Some("salt".to_string()),
                    credential_type: CredentialType::Password,
                    user_id,
                    user_label: None,
                    secret_data: "new_hash".to_string(),
                    credential_data: CredentialData::new_hash(1, "argon2".to_string()),
                    temporary: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    webauthn_credential_id: None,
                    recovery_code_lookup: None,
                };
                Box::pin(async move { Ok(cred) })
            });

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = builder.with_user_access_revoked(1).build();

        let result = service
            .update_password(
                Identity::User(user),
                UpdatePasswordInput {
                    realm_name: "test-realm".to_string(),
                    value: "Str0ng!P@ssword#2024".to_string(),
                },
            )
            .await;

        assert!(result.is_ok(), "update_password should succeed: {result:?}");
    }

    #[tokio::test]
    async fn complete_password_reset_valid_token_succeeds() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let token_id = Uuid::new_v4();

        let prt = PasswordResetToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: realm.id.into(),
            token_id,
            token_hash: "hashed_token".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(30),
            auth_session_code: None,
        };
        let prt_user_id = prt.user_id;

        let prt_clone = prt.clone();
        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_get_by_token_id()
            .returning(move |_| {
                let t = prt_clone.clone();
                Box::pin(async move { Ok(Some(t)) })
            });

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_magic_token()
            .returning(|_, _| Box::pin(async { Ok(true) }));

        // Policy lookup: no stored policy → use CNIL defaults
        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        // User lookup for username/email context
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| Box::pin(async { Err(CoreError::NotFound) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_password_credential()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_password()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "new_hash".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_credential()
            .returning(move |_, _, _, _, _| {
                let cred = crate::domain::credential::entities::Credential {
                    id: Uuid::new_v4(),
                    salt: Some("salt".to_string()),
                    credential_type: CredentialType::Password,
                    user_id: prt_user_id,
                    user_label: None,
                    secret_data: "new_hash".to_string(),
                    credential_data: CredentialData::new_hash(1, "argon2".to_string()),
                    temporary: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    webauthn_credential_id: None,
                    recovery_code_lookup: None,
                };
                Box::pin(async move { Ok(cred) })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_delete_all_by_user_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        let service = builder.with_user_access_revoked(1).build();
        // Strong password satisfying CNIL defaults (≥12 chars, all classes, ≥80 bits entropy)
        let result = service
            .complete_password_reset(CompletePasswordResetInput {
                token_id,
                token: "raw_token".to_string(),
                new_password: "Str0ng!P@ssword#2024".to_string(),
            })
            .await;

        assert!(result.is_ok(), "expected Ok, got Err");
    }

    #[tokio::test]
    async fn complete_password_reset_expired_token_returns_error() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let token_id = Uuid::new_v4();

        let prt = PasswordResetToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: realm.id.into(),
            token_id,
            token_hash: "hashed_token".to_string(),
            created_at: Utc::now() - Duration::hours(1),
            expires_at: Utc::now() - Duration::minutes(30), // expired
            auth_session_code: None,
        };

        let prt_clone = prt.clone();
        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_get_by_token_id()
            .returning(move |_| {
                let t = prt_clone.clone();
                Box::pin(async move { Ok(Some(t)) })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_delete_by_token_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .complete_password_reset(CompletePasswordResetInput {
                token_id,
                token: "raw_token".to_string(),
                new_password: "newpassword123".to_string(),
            })
            .await;

        assert!(matches!(result, Err(CoreError::ExpiredToken)));
    }

    #[tokio::test]
    async fn complete_password_reset_invalid_token_returns_error() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let token_id = Uuid::new_v4();

        let prt = PasswordResetToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: realm.id.into(),
            token_id,
            token_hash: "hashed_token".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(30),
            auth_session_code: None,
        };

        let prt_clone = prt.clone();
        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_get_by_token_id()
            .returning(move |_| {
                let t = prt_clone.clone();
                Box::pin(async move { Ok(Some(t)) })
            });

        // Token verification fails
        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_magic_token()
            .returning(|_, _| Box::pin(async { Ok(false) }));

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_delete_by_token_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .complete_password_reset(CompletePasswordResetInput {
                token_id,
                token: "wrong_token".to_string(),
                new_password: "newpassword123".to_string(),
            })
            .await;

        assert!(matches!(result, Err(CoreError::InvalidToken)));
    }

    #[tokio::test]
    async fn complete_password_reset_not_found_token_returns_error() {
        let mut builder = TridentTestBuilder::new();
        let token_id = Uuid::new_v4();

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_get_by_token_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .complete_password_reset(CompletePasswordResetInput {
                token_id,
                token: "raw_token".to_string(),
                new_password: "newpassword123".to_string(),
            })
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    // ── OTP enrolment (FK-003) ──────────────────────────────────────────

    /// 20 raw bytes encoded as base32/RFC4648 without padding — the only shape
    /// `TotpSecret::to_bytes` accepts.
    const SERVER_SECRET: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    /// A second, different well-formed secret, standing in for one an attacker
    /// picked themselves instead of using the one the server issued.
    const ATTACKER_SECRET: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";

    fn current_code_for(secret_b32: &str) -> String {
        let bytes = TotpSecret::from_base32(secret_b32)
            .to_bytes()
            .expect("test secret must decode to 20 bytes");

        let counter = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before UNIX_EPOCH")
            .as_secs()
            / 30;

        let code = generate_totp_code(&bytes, counter, 6).expect("code generation must succeed");
        format!("{code:06}")
    }

    fn otp_credential(user_id: Uuid) -> Credential {
        Credential {
            id: Uuid::new_v4(),
            salt: None,
            credential_type: CredentialType::Otp,
            user_id,
            user_label: Some("victim phone".to_string()),
            secret_data: SERVER_SECRET.to_string(),
            credential_data: CredentialData::new_hash(1, "argon2".to_string()),
            temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
            recovery_code_lookup: None,
        }
    }

    fn created_otp_credential(user_id: Uuid, secret_data: String) -> Credential {
        Credential {
            id: Uuid::new_v4(),
            salt: None,
            credential_type: CredentialType::Otp,
            user_id,
            user_label: None,
            secret_data,
            credential_data: CredentialData::new_hash(1, "argon2".to_string()),
            temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
            recovery_code_lookup: None,
        }
    }

    fn active_otp_enrollment(user_id: Uuid, secret: String) -> OtpEnrollment {
        OtpEnrollment {
            id: Uuid::new_v4(),
            user_id,
            secret,
            expires_at: Utc::now() + Duration::minutes(10),
            created_at: Utc::now(),
        }
    }

    fn expect_active_otp_enrollment(
        builder: &mut TridentTestBuilder,
        user_id: Uuid,
        secret: String,
    ) {
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_get_active_enrollment()
            .returning(move |requested_user_id, _| {
                let enrollment = active_otp_enrollment(user_id, secret.clone());
                Box::pin(async move {
                    if requested_user_id == enrollment.user_id {
                        Ok(Some(enrollment))
                    } else {
                        Ok(None)
                    }
                })
            });
    }

    #[tokio::test]
    async fn setup_otp_persists_candidate_secret_server_side() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        let recorded: CapturedEnrollment = Arc::new(Mutex::new(None));
        let sink = Arc::clone(&recorded);

        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_cleanup_expired()
            .times(1)
            .returning(|| Box::pin(async { Ok(0u64) }));

        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_start_enrollment()
            .times(1)
            .returning(move |user_id, secret, expires_at| {
                *sink.lock().expect("mutex poisoned") = Some((user_id, secret.clone(), expires_at));
                Box::pin(async move {
                    Ok(OtpEnrollment {
                        id: Uuid::new_v4(),
                        user_id,
                        secret,
                        expires_at,
                        created_at: Utc::now(),
                    })
                })
            });

        let service = builder.build();
        let before = Utc::now();
        let result = service
            .setup_otp(
                Identity::User(user.clone()),
                SetupOtpInput {
                    issuer: "https://idp.example".to_string(),
                },
            )
            .await
            .expect("setup_otp must succeed");

        let (persisted_user, persisted_secret, expires_at) = recorded
            .lock()
            .expect("mutex poisoned")
            .clone()
            .expect("setup_otp must persist the candidate secret server-side");

        assert_eq!(persisted_user, user.id);
        assert_eq!(
            persisted_secret, result.secret,
            "the persisted candidate must be the very secret handed to the user"
        );
        assert!(
            expires_at > before + Duration::minutes(OTP_ENROLLMENT_TTL_MINUTES - 1)
                && expires_at < before + Duration::minutes(OTP_ENROLLMENT_TTL_MINUTES + 1),
            "enrolment must expire after the configured TTL, got {expires_at}"
        );
    }

    #[tokio::test]
    async fn verify_otp_rejects_code_computed_from_caller_chosen_secret() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        // The server did issue an enrolment — for a secret the caller does not use.
        expect_active_otp_enrollment(&mut builder, user.id, SERVER_SECRET.to_string());

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .never()
            .returning(move |uid, _, secret, _, _| {
                let cred = created_otp_credential(uid, secret);
                Box::pin(async move { Ok(cred) })
            });

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(ATTACKER_SECRET),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            result.is_err(),
            "a code valid only against a caller-chosen secret must not enrol anything"
        );
    }

    #[tokio::test]
    async fn verify_otp_without_pending_enrollment_is_rejected() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_get_active_enrollment()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .never()
            .returning(move |uid, _, secret, _, _| {
                let cred = created_otp_credential(uid, secret);
                Box::pin(async move { Ok(cred) })
            });

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            result.is_err(),
            "verify_otp must refuse when setup_otp never ran"
        );
    }

    #[tokio::test]
    async fn verify_otp_rejects_expired_enrollment() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        // Stands in for the adapter: an expired enrollment is never handed
        // back, so `get_active_enrollment` returns `None` and the service must refuse.
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_get_active_enrollment()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .never()
            .returning(move |uid, _, secret, _, _| {
                let cred = created_otp_credential(uid, secret);
                Box::pin(async move { Ok(cred) })
            });

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            result.is_err(),
            "an enrolment past its TTL must not be usable"
        );
    }

    #[tokio::test]
    async fn verify_otp_rejects_second_use_of_same_enrollment() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        expect_active_otp_enrollment(&mut builder, user.id, SERVER_SECRET.to_string());
        let claimed = Arc::new(Mutex::new(false));
        let claimed_clone = Arc::clone(&claimed);
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_claim_enrollment()
            .times(2)
            .returning(move |_, _| {
                let already_claimed = {
                    let mut guard = claimed_clone.lock().expect("mutex poisoned");
                    let seen = *guard;
                    *guard = true;
                    seen
                };
                Box::pin(async move { Ok(!already_claimed) })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .times(1)
            .returning(move |uid, _, secret, _, _| {
                let cred = created_otp_credential(uid, secret);
                Box::pin(async move { Ok(cred) })
            });

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        // The first (successful) enrolment audits an MfaEnrolled event and
        // sends a best-effort factor-change email.
        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let first = service
            .verify_otp(
                Identity::User(user.clone()),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: None,
                },
            )
            .await;
        assert!(first.is_ok(), "the first enrolment must succeed");

        let second = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            second.is_err(),
            "an enrolment already claimed must not be replayable"
        );
    }

    #[tokio::test]
    async fn verify_otp_rejects_reenrollment_without_configure_otp_and_keeps_existing_credential() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        let existing_otp = created_otp_credential(user.id, "EXISTINGSECRET".to_string());
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = vec![existing_otp.clone()];
                Box::pin(async move { Ok(creds) })
            });

        expect_active_otp_enrollment(&mut builder, user.id, SERVER_SECRET.to_string());

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .never()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .never()
            .returning(move |uid, _, secret, _, _| {
                let cred = created_otp_credential(uid, secret);
                Box::pin(async move { Ok(cred) })
            });

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "verify_otp must reject re-enrollment unless ConfigureOtp is required"
        );
    }

    #[tokio::test]
    async fn verify_otp_allows_reenrollment_when_configure_otp_is_required() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let mut user = create_test_user_with_email(&realm, "user@example.com");
        user.required_actions = vec![RequiredAction::ConfigureOtp];

        let existing_otp = created_otp_credential(user.id, "EXISTINGSECRET".to_string());
        let existing_id = existing_otp.id;
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = vec![existing_otp.clone()];
                Box::pin(async move { Ok(creds) })
            });

        expect_active_otp_enrollment(&mut builder, user.id, SERVER_SECRET.to_string());
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_claim_enrollment()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .times(1)
            .returning(move |id| {
                assert_eq!(id, existing_id);
                Box::pin(async { Ok(()) })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .times(1)
            .returning(move |uid, _, secret, _, _| {
                let cred = created_otp_credential(uid, secret);
                Box::pin(async move { Ok(cred) })
            });

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            result.is_ok(),
            "verify_otp must still allow re-enrollment when ConfigureOtp is required"
        );
    }

    #[tokio::test]
    async fn verify_otp_allows_reenrollment_with_valid_step_up_token() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        // No ConfigureOtp action: only the step-up token authorises the
        // replacement of the existing authenticator (e.g. the user changed
        // phone and walked /me/reauthenticate → /me/totp/setup → verify).
        let user = create_test_user_with_email(&realm, "user@example.com");

        let existing_otp = created_otp_credential(user.id, "EXISTINGSECRET".to_string());
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = vec![existing_otp.clone()];
                Box::pin(async move { Ok(creds) })
            });

        expect_active_otp_enrollment(&mut builder, user.id, SERVER_SECRET.to_string());
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_claim_enrollment()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .times(1)
            .returning(move |uid, _, secret, _, _| {
                let cred = created_otp_credential(uid, secret);
                Box::pin(async move { Ok(cred) })
            });

        expect_valid_step_up_token(&mut builder, "valid-step-up-token");

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: Some("valid-step-up-token".to_string()),
                },
            )
            .await;

        assert!(
            result.is_ok(),
            "a consumed step-up token must authorise replacing an existing authenticator"
        );
    }

    #[tokio::test]
    async fn verify_otp_guard_rejection_does_not_touch_step_up_token_store() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        let existing_otp = created_otp_credential(user.id, "EXISTINGSECRET".to_string());
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = vec![existing_otp.clone()];
                Box::pin(async move { Ok(creds) })
            });

        // Deliberately NO expectations on step_up_token_repo: the guard must
        // reject before any token logic runs, so any call would panic the mock
        // and fail this test.
        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: current_code_for(SERVER_SECRET),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "the login-flow guard must still reject without ConfigureOtp"
        );
    }

    // ── Passkey enrolment (FK-003, WebAuthn side) ───────────────────────

    fn dummy_register_credential() -> RegisterPublicKeyCredential {
        serde_json::from_value(serde_json::json!({
            "id": "AAAA",
            "rawId": "AAAA",
            "response": {
                "attestationObject": "AAAA",
                "clientDataJSON": "AAAA",
            },
            "type": "public-key",
        }))
        .expect("static fixture must deserialize")
    }

    fn auth_session_with_challenge_issued_at(
        realm: &crate::domain::realm::entities::Realm,
        session_code: Uuid,
        issued_at: Option<DateTime<Utc>>,
    ) -> AuthSession {
        AuthSession {
            id: session_code,
            realm_id: realm.id,
            client_id: Uuid::new_v4(),
            redirect_uri: "https://app.example/callback".to_string(),
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: Some("state".to_string()),
            nonce: None,
            user_id: None,
            code: None,
            authenticated: false,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(10),
            webauthn_challenge: None,
            webauthn_challenge_issued_at: issued_at,
            compass_flow_id: None,
            code_challenge: None,
            code_challenge_method: None,
        }
    }

    fn webauthn_create_input(session_code: Uuid) -> WebAuthnValidatePublicKeyInput {
        WebAuthnValidatePublicKeyInput {
            rp_info: WebAuthnRpInfo {
                rp_id: "localhost".to_string(),
                allowed_origin: "http://localhost:5555".to_string(),
            },
            session_code: session_code.to_string(),
            credential: dummy_register_credential(),
        }
    }

    #[tokio::test]
    async fn webauthn_public_key_create_rejects_when_configure_passkey_not_required() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "victim@example.com");

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_get_required_actions()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_get_by_session_code()
            .never()
            .returning(|_| Box::pin(async { Err(AuthenticationError::NotFound) }));

        let service = builder.build();
        let result = service
            .webauthn_public_key_create(Identity::User(user), webauthn_create_input(Uuid::new_v4()))
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "enrolling a passkey requires a pending ConfigurePasskey required action"
        );
    }

    #[tokio::test]
    async fn webauthn_public_key_create_rejects_stale_challenge() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");
        let session_code = Uuid::new_v4();

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_get_required_actions()
            .returning(|_| Box::pin(async { Ok(vec![RequiredAction::ConfigurePasskey]) }));

        let stale = auth_session_with_challenge_issued_at(
            &realm,
            session_code,
            Some(Utc::now() - Duration::minutes(WEBAUTHN_CHALLENGE_TTL_MINUTES + 5)),
        );
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_get_by_session_code()
            .returning(move |_| {
                let session = stale.clone();
                Box::pin(async move { Ok(session) })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_webauthn_credential()
            .never()
            .returning(|_, _| Box::pin(async { Err(CredentialError::CreateCredentialError) }));

        let service = builder.build();
        let result = service
            .webauthn_public_key_create(Identity::User(user), webauthn_create_input(session_code))
            .await;

        assert!(
            matches!(result, Err(CoreError::WebAuthnChallengeFailed)),
            "a challenge older than its TTL must be refused"
        );
    }

    fn password_credential(user_id: Uuid) -> Credential {
        Credential {
            id: Uuid::new_v4(),
            salt: Some("salt".to_string()),
            credential_type: CredentialType::Password,
            user_id,
            user_label: None,
            secret_data: "new_hash".to_string(),
            credential_data: CredentialData::new_hash(1, "argon2".to_string()),
            temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
            recovery_code_lookup: None,
        }
    }

    fn magic_link_for(user_id: Uuid, realm_id: Uuid, session_code: Uuid) -> MagicLink {
        MagicLink {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            magic_token_id: Uuid::new_v4(),
            magic_token_hash: "hashed".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(10),
            auth_session_code: Some(session_code),
        }
    }

    fn expect_pending_step_lookups(
        builder: &mut TridentTestBuilder,
        user: crate::domain::user::entities::User,
        credentials: Vec<Credential>,
        settings: RealmSetting,
    ) {
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user.clone();
                Box::pin(async move { Ok(u) })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let c = credentials.clone();
                Box::pin(async move { Ok(c) })
            });

        Arc::get_mut(&mut builder.user_role_repo)
            .unwrap()
            .expect_get_user_roles()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings.clone();
                Box::pin(async move { Ok(Some(s)) })
            });
    }

    fn expect_no_authorization_code(builder: &mut TridentTestBuilder) {
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_update_code_and_user_id()
            .never()
            .returning(|_, _, _| Box::pin(async { Err(AuthenticationError::NotFound) }));
    }

    fn expect_authorization_code(builder: &mut TridentTestBuilder, session: AuthSession) {
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_update_code_and_user_id()
            .times(1)
            .returning(move |_, _, _| {
                let s = session.clone();
                Box::pin(async move { Ok(s) })
            });
    }

    struct MagicLinkFixture {
        builder: TridentTestBuilder,
        token_id: Uuid,
        session: AuthSession,
    }

    fn magic_link_fixture(
        realm: &crate::domain::realm::entities::Realm,
        user: &crate::domain::user::entities::User,
    ) -> MagicLinkFixture {
        let mut builder = TridentTestBuilder::new();
        let session_code = Uuid::new_v4();
        let session = auth_session_with_challenge_issued_at(realm, session_code, None);

        let magic_link = magic_link_for(user.id, Uuid::from(realm.id), session_code);
        let token_id = magic_link.magic_token_id;

        Arc::get_mut(&mut builder.magic_link_repo)
            .unwrap()
            .expect_get_by_token_id()
            .returning(move |_| {
                let ml = magic_link.clone();
                Box::pin(async move { Ok(Some(ml)) })
            });

        let session_clone = session.clone();
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_get_by_session_code()
            .returning(move |_| {
                let s = session_clone.clone();
                Box::pin(async move { Ok(s) })
            });

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_magic_token()
            .returning(|_, _| Box::pin(async { Ok(true) }));

        MagicLinkFixture {
            builder,
            token_id,
            session,
        }
    }

    #[tokio::test]
    async fn verify_magic_link_withholds_the_code_while_otp_enrolment_is_pending() {
        let realm = create_test_realm_with_name("test-realm");
        let mut user = create_test_user_with_email(&realm, "user@example.com");
        user.required_actions = vec![RequiredAction::ConfigureOtp];

        let MagicLinkFixture {
            mut builder,
            token_id,
            ..
        } = magic_link_fixture(&realm, &user);

        expect_pending_step_lookups(
            &mut builder,
            user,
            Vec::new(),
            create_test_realm_setting(realm.id, false),
        );
        expect_no_authorization_code(&mut builder);

        let service = builder.build();
        let result = service
            .verify_magic_link(VerifyMagicLinkInput {
                magic_token_id: token_id,
                magic_token: "raw_token".to_string(),
            })
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "a magic link must not log in a user who still owes ConfigureOtp: {result:?}"
        );
    }

    #[tokio::test]
    async fn verify_magic_link_withholds_the_code_from_an_otp_credential_holder() {
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        let MagicLinkFixture {
            mut builder,
            token_id,
            ..
        } = magic_link_fixture(&realm, &user);

        let credentials = vec![otp_credential(user.id)];
        expect_pending_step_lookups(
            &mut builder,
            user,
            credentials,
            create_test_realm_setting(realm.id, false),
        );
        expect_no_authorization_code(&mut builder);

        let service = builder.build();
        let result = service
            .verify_magic_link(VerifyMagicLinkInput {
                magic_token_id: token_id,
                magic_token: "raw_token".to_string(),
            })
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "an OTP credential still demands a challenge before any code is minted: {result:?}"
        );
    }

    #[tokio::test]
    async fn verify_magic_link_does_not_stall_on_a_pending_email_verification() {
        let realm = create_test_realm_with_name("test-realm");
        let mut user = create_test_user_with_email(&realm, "user@example.com");
        user.required_actions = vec![RequiredAction::VerifyEmail];

        let MagicLinkFixture {
            mut builder,
            token_id,
            session,
        } = magic_link_fixture(&realm, &user);

        expect_pending_step_lookups(
            &mut builder,
            user,
            Vec::new(),
            create_test_realm_setting(realm.id, false),
        );
        expect_authorization_code(&mut builder, session);

        Arc::get_mut(&mut builder.magic_link_repo)
            .unwrap()
            .expect_delete_by_token_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .verify_magic_link(VerifyMagicLinkInput {
                magic_token_id: token_id,
                magic_token: "raw_token".to_string(),
            })
            .await;

        let url = result.expect("clicking the link already proves mailbox control");
        assert!(
            url.contains("code="),
            "expected an authorization code in {url}"
        );
    }

    #[tokio::test]
    async fn verify_magic_link_still_logs_in_a_user_owing_nothing() {
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        let MagicLinkFixture {
            mut builder,
            token_id,
            session,
        } = magic_link_fixture(&realm, &user);

        expect_pending_step_lookups(
            &mut builder,
            user,
            Vec::new(),
            create_test_realm_setting(realm.id, false),
        );
        expect_authorization_code(&mut builder, session);

        Arc::get_mut(&mut builder.magic_link_repo)
            .unwrap()
            .expect_delete_by_token_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .verify_magic_link(VerifyMagicLinkInput {
                magic_token_id: token_id,
                magic_token: "raw_token".to_string(),
            })
            .await;

        let url = result.expect("a user owing no step must still be logged in");
        assert!(
            url.contains("code="),
            "expected an authorization code in {url}"
        );
    }

    struct PasswordResetFixture {
        builder: TridentTestBuilder,
        token_id: Uuid,
        session: AuthSession,
    }

    fn password_reset_fixture(
        realm: &crate::domain::realm::entities::Realm,
        user: &crate::domain::user::entities::User,
    ) -> PasswordResetFixture {
        let mut builder = TridentTestBuilder::new();
        let token_id = Uuid::new_v4();
        let session_code = Uuid::new_v4();
        let session = auth_session_with_challenge_issued_at(realm, session_code, None);
        let user_id = user.id;

        let prt = PasswordResetToken {
            id: Uuid::new_v4(),
            user_id,
            realm_id: Uuid::from(realm.id),
            token_id,
            token_hash: "hashed_token".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(30),
            auth_session_code: Some(session_code),
        };

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_get_by_token_id()
            .returning(move |_| {
                let t = prt.clone();
                Box::pin(async move { Ok(Some(t)) })
            });

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_magic_token()
            .returning(|_, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_password_credential()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_password()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "new_hash".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_credential()
            .times(1)
            .returning(move |_, _, _, _, _| {
                let cred = password_credential(user_id);
                Box::pin(async move { Ok(cred) })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_delete_all_by_user_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        let session_clone = session.clone();
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_get_by_session_code()
            .returning(move |_| {
                let s = session_clone.clone();
                Box::pin(async move { Ok(s) })
            });

        PasswordResetFixture {
            builder,
            token_id,
            session,
        }
    }

    #[tokio::test]
    async fn complete_password_reset_withholds_auto_login_while_an_action_is_pending() {
        let realm = create_test_realm_with_name("test-realm");
        let mut user = create_test_user_with_email(&realm, "user@example.com");
        user.required_actions = vec![RequiredAction::ConfigureOtp];

        let PasswordResetFixture {
            mut builder,
            token_id,
            ..
        } = password_reset_fixture(&realm, &user);

        expect_pending_step_lookups(
            &mut builder,
            user,
            Vec::new(),
            create_test_realm_setting(realm.id, false),
        );
        expect_no_authorization_code(&mut builder);

        let service = builder.with_user_access_revoked(1).build();
        let output = service
            .complete_password_reset(CompletePasswordResetInput {
                token_id,
                token: "raw_token".to_string(),
                new_password: "Str0ng!P@ssword#2024".to_string(),
            })
            .await
            .expect("the password change itself must still go through");

        assert!(
            output.login_url.is_none(),
            "a residual required action must not be skipped by the reset auto-login"
        );
    }

    #[tokio::test]
    async fn complete_password_reset_still_logs_in_a_user_owing_nothing() {
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");

        let PasswordResetFixture {
            mut builder,
            token_id,
            session,
        } = password_reset_fixture(&realm, &user);

        expect_pending_step_lookups(
            &mut builder,
            user,
            Vec::new(),
            create_test_realm_setting(realm.id, false),
        );
        expect_authorization_code(&mut builder, session);

        let service = builder.with_user_access_revoked(1).build();
        let output = service
            .complete_password_reset(CompletePasswordResetInput {
                token_id,
                token: "raw_token".to_string(),
                new_password: "Str0ng!P@ssword#2024".to_string(),
            })
            .await
            .expect("password reset must succeed");

        let url = output
            .login_url
            .expect("a user owing no step keeps the auto-login");
        assert!(
            url.contains("code="),
            "expected an authorization code in {url}"
        );
    }

    #[tokio::test]
    async fn the_shared_gate_refuses_an_otp_holder_for_passkey_and_webauthn_callers() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "user@example.com");
        let session = auth_session_with_challenge_issued_at(&realm, Uuid::new_v4(), None);

        let credentials = vec![otp_credential(user.id)];
        let user_id = user.id;
        expect_pending_step_lookups(
            &mut builder,
            user,
            credentials,
            create_test_realm_setting(realm.id, false),
        );
        expect_no_authorization_code(&mut builder);

        let service = builder.build();
        let result = service
            .store_auth_code_and_generate_login_url(&session, user_id, &[])
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "passkey and webauthn waive nothing, so an OTP credential must block: {result:?}"
        );
    }

    #[tokio::test]
    async fn only_the_magic_link_path_waives_the_email_verification_step() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let mut user = create_test_user_with_email(&realm, "user@example.com");
        user.required_actions = vec![RequiredAction::VerifyEmail];
        let session = auth_session_with_challenge_issued_at(&realm, Uuid::new_v4(), None);

        let user_id = user.id;
        expect_pending_step_lookups(
            &mut builder,
            user,
            Vec::new(),
            create_test_realm_setting(realm.id, false),
        );
        expect_no_authorization_code(&mut builder);

        let service = builder.build();
        let result = service
            .store_auth_code_and_generate_login_url(&session, user_id, &[])
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "the VerifyEmail waiver belongs to the magic link path only: {result:?}"
        );
    }

    // ── list_credentials_self_service ───────────────────────────────────

    fn test_credential(user_id: Uuid, credential_type: CredentialType) -> Credential {
        Credential {
            id: Uuid::new_v4(),
            salt: Some("salt".to_string()),
            credential_type,
            user_id,
            user_label: None,
            secret_data: "secret".to_string(),
            credential_data: CredentialData::new_hash(1, "argon2".to_string()),
            temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
            recovery_code_lookup: None,
        }
    }

    /// Wire the step-up token repository so `consume_step_up_token` succeeds
    /// for the given presented token. Used by tests that exercise sensitive
    /// self-service operations gated behind a step-up token.
    fn expect_valid_step_up_token(builder: &mut TridentTestBuilder, token: &str) {
        let stored = token.to_string();
        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_find_active()
            .returning(move |_| {
                let s = stored.clone();
                Box::pin(async move {
                    Ok(vec![StepUpTokenRecord {
                        id: Uuid::new_v4(),
                        user_id: Uuid::new_v4(),
                        token_hash: s,
                        expires_at: Utc::now() + Duration::minutes(5),
                    }])
                })
            });
        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(true) }));
        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_magic_token()
            .returning(|presented: &str, stored: &str| {
                let p = presented.to_string();
                let h = stored.to_string();
                Box::pin(async move { Ok(p == h) })
            });
    }

    /// Stub the realm settings lookup so lockout/realm-config checks resolve.
    fn expect_realm_settings(
        builder: &mut TridentTestBuilder,
        realm: &crate::domain::realm::entities::Realm,
    ) {
        let settings = create_test_realm_setting(realm.id, true);
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings.clone();
                Box::pin(async move { Ok(Some(s)) })
            });
    }

    /// Stub the fast recovery-code lookup key so the unauthenticated reset and
    /// the authenticated burn both map a submitted code to a single candidate.
    fn expect_recovery_code_lookup(builder: &mut TridentTestBuilder) {
        Arc::get_mut(&mut builder.recovery_code_repo)
            .unwrap()
            .expect_lookup_of()
            .returning(|_| "test-lookup-key".to_string());
    }

    #[tokio::test]
    async fn list_credentials_self_service_returns_overview() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let otp_cred = test_credential(user.id, CredentialType::Otp);
        let passkey_cred = test_credential(user.id, CredentialType::WebAuthnPublicKeyCredential);
        let otp_id = otp_cred.id;
        let passkey_id = passkey_cred.id;

        let creds = vec![otp_cred, passkey_cred];
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = creds.clone();
                Box::pin(async move { Ok(creds) })
            });

        let service = builder.build();
        let result = service
            .list_credentials_self_service(Identity::User(user))
            .await;

        assert!(result.is_ok(), "expected Ok");
        let overviews = result.unwrap();
        assert_eq!(overviews.len(), 2);
        assert_eq!(overviews[0].id, otp_id);
        assert_eq!(overviews[0].credential_type, "otp");
        assert_eq!(overviews[1].id, passkey_id);
        assert_eq!(
            overviews[1].credential_type,
            "webauthn-public-key-credential"
        );
    }

    #[tokio::test]
    async fn list_credentials_self_service_forbids_non_user_identity() {
        let builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let service = builder.build();

        let result = service
            .list_credentials_self_service(
                crate::domain::common::services::tests::create_test_client_identity(realm.id),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    // ── delete_credential_self_service ──────────────────────────────────

    #[tokio::test]
    async fn delete_credential_self_service_removes_owned_credential() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        // The user keeps a password credential, so removing the OTP factor is safe.
        let password_cred = test_credential(user.id, CredentialType::Password);
        let owned = test_credential(user.id, CredentialType::Otp);
        let owned_id = owned.id;

        let creds = vec![password_cred, owned.clone()];
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = creds.clone();
                Box::pin(async move { Ok(creds) })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        expect_valid_step_up_token(&mut builder, "valid-step-up-token");

        expect_realm_settings(&mut builder, &realm);

        // The removal emits the user.credentials.deleted webhook.
        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .times(1)
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        // The factor-change notification email looks up the user and realm SMTP
        // config (best-effort; no SMTP configured → EmailNotSent event logged).
        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .delete_credential_self_service(
                Identity::User(user),
                owned_id,
                "valid-step-up-token".to_string(),
            )
            .await;

        assert!(result.is_ok(), "expected Ok");
    }

    #[tokio::test]
    async fn delete_credential_self_service_rejects_password_credential() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let password_cred = test_credential(user.id, CredentialType::Password);
        let password_id = password_cred.id;
        let otp_cred = test_credential(user.id, CredentialType::Otp);

        let creds = vec![password_cred, otp_cred];
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = creds.clone();
                Box::pin(async move { Ok(creds) })
            });

        expect_valid_step_up_token(&mut builder, "valid-step-up-token");

        let service = builder.build();
        let result = service
            .delete_credential_self_service(
                Identity::User(user),
                password_id,
                "valid-step-up-token".to_string(),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn delete_credential_self_service_allows_recovery_code_deletion() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        // A recovery code is not a primary factor, so deleting it while the
        // password remains cannot lock the user out and must succeed.
        let password_cred = test_credential(user.id, CredentialType::Password);
        let recovery_cred = test_credential(user.id, CredentialType::RecoveryCode);
        let recovery_id = recovery_cred.id;

        let creds = vec![password_cred, recovery_cred];
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = creds.clone();
                Box::pin(async move { Ok(creds) })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        expect_valid_step_up_token(&mut builder, "valid-step-up-token");

        // The realm policy is resolved before the deletion (require_mfa=false
        // by default, so no re-enrolment action is queued).
        expect_realm_settings(&mut builder, &realm);

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        // The removal emits the user.credentials.deleted webhook.
        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .times(1)
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        // The factor-change notification email looks up the user and realm SMTP
        // config (best-effort; no SMTP configured → EmailNotSent event logged).
        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .delete_credential_self_service(
                Identity::User(user),
                recovery_id,
                "valid-step-up-token".to_string(),
            )
            .await;

        assert!(result.is_ok(), "expected Ok");
    }

    #[tokio::test]
    async fn delete_credential_self_service_rejects_last_primary_factor_removal() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        // The OTP is the user's only primary factor (recovery codes do not
        // count), so removing it would lock the user out and is rejected.
        let otp_cred = test_credential(user.id, CredentialType::Otp);
        let otp_id = otp_cred.id;
        let recovery_cred = test_credential(user.id, CredentialType::RecoveryCode);

        let creds = vec![otp_cred, recovery_cred];
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = creds.clone();
                Box::pin(async move { Ok(creds) })
            });

        expect_valid_step_up_token(&mut builder, "valid-step-up-token");

        let service = builder.build();
        let result = service
            .delete_credential_self_service(
                Identity::User(user),
                otp_id,
                "valid-step-up-token".to_string(),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn delete_credential_self_service_rejects_unowned_credential() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let other = test_credential(user.id, CredentialType::Otp);
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let other = other.clone();
                Box::pin(async move { Ok(vec![other]) })
            });

        expect_valid_step_up_token(&mut builder, "valid-step-up-token");

        let service = builder.build();
        let result = service
            .delete_credential_self_service(
                Identity::User(user),
                Uuid::new_v4(),
                "valid-step-up-token".to_string(),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    // ── reauthenticate ──────────────────────────────────────────────────

    #[tokio::test]
    async fn reauthenticate_valid_password_without_otp_succeeds() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let password_cred = test_credential(user.id, CredentialType::Password);
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_password_credential()
            .returning(move |_| {
                let c = password_cred.clone();
                Box::pin(async move { Ok(c) })
            });

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_password()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(true) }));

        // No OTP credential configured.
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        expect_realm_settings(&mut builder, &realm);

        // Success path resets the failure counter, scrubs expired step-up
        // tokens, then mints a new one (hash + persist).
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_reset_failed_login_attempts()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_cleanup_expired()
            .returning(|| Box::pin(async { Ok(0u64) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_magic_token()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "hashed".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_save()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .reauthenticate(
                Identity::User(user),
                ReauthenticateInput {
                    password: "correct-password".to_string(),
                    otp_code: None,
                },
            )
            .await;

        assert!(result.is_ok(), "expected Ok");
    }

    #[tokio::test]
    async fn reauthenticate_wrong_password_fails() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let password_cred = test_credential(user.id, CredentialType::Password);
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_password_credential()
            .returning(move |_| {
                let c = password_cred.clone();
                Box::pin(async move { Ok(c) })
            });

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_password()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(false) }));

        expect_realm_settings(&mut builder, &realm);

        // A wrong password bumps the lockout counter and audits the failure.
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_increment_failed_login_attempts()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .reauthenticate(
                Identity::User(user),
                ReauthenticateInput {
                    password: "wrong-password".to_string(),
                    otp_code: None,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::InvalidPassword)));
    }

    #[tokio::test]
    async fn reauthenticate_requires_otp_when_authenticator_configured() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let password_cred = test_credential(user.id, CredentialType::Password);
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_password_credential()
            .returning(move |_| {
                let c = password_cred.clone();
                Box::pin(async move { Ok(c) })
            });

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_password()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(true) }));

        let otp_cred = test_credential(user.id, CredentialType::Otp);
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let otp_cred = otp_cred.clone();
                Box::pin(async move { Ok(vec![otp_cred]) })
            });

        expect_realm_settings(&mut builder, &realm);

        let service = builder.build();
        let result = service
            .reauthenticate(
                Identity::User(user),
                ReauthenticateInput {
                    password: "correct-password".to_string(),
                    otp_code: None,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::TotpVerificationFailed(_))));
    }

    // ── passkey_register_options_self_service ───────────────────────────

    fn test_rp_info() -> WebAuthnRpInfo {
        WebAuthnRpInfo {
            rp_id: "localhost".to_string(),
            allowed_origin: "http://localhost:5555".to_string(),
        }
    }

    #[tokio::test]
    async fn passkey_register_options_self_service_forbids_non_user_identity() {
        let builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let service = builder.build();

        let result = service
            .passkey_register_options_self_service(
                crate::domain::common::services::tests::create_test_client_identity(realm.id),
                PasskeyRegisterOptionsSelfServiceInput {
                    rp_info: test_rp_info(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn passkey_register_options_self_service_generates_options_for_user() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        // No existing passkey credentials.
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_webauthn_public_key_credentials()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        Arc::get_mut(&mut builder.webauthn_challenge_repo)
            .unwrap()
            .expect_cleanup_expired()
            .returning(|| Box::pin(async { Ok(0u64) }));

        Arc::get_mut(&mut builder.webauthn_challenge_repo)
            .unwrap()
            .expect_save()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .passkey_register_options_self_service(
                Identity::User(user),
                PasskeyRegisterOptionsSelfServiceInput {
                    rp_info: test_rp_info(),
                },
            )
            .await;

        assert!(result.is_ok(), "expected Ok, got an error");
    }

    // ── passkey_register_self_service ───────────────────────────────────

    fn test_register_public_key_credential() -> RegisterPublicKeyCredential {
        serde_json::from_value(serde_json::json!({
            "id": "dummy-id",
            "rawId": "",
            "response": {
                "attestationObject": "",
                "clientDataJSON": ""
            },
            "type": "public-key",
            "extensions": {}
        }))
        .expect("valid RegisterPublicKeyCredential payload")
    }

    #[tokio::test]
    async fn passkey_register_self_service_forbids_non_user_identity() {
        let builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let service = builder.build();

        let result = service
            .passkey_register_self_service(
                crate::domain::common::services::tests::create_test_client_identity(realm.id),
                PasskeyRegisterSelfServiceInput {
                    rp_info: test_rp_info(),
                    credential: test_register_public_key_credential(),
                    step_up_token: "x".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn passkey_register_self_service_missing_pending_challenge_fails() {
        let builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let mut builder = builder;
        Arc::get_mut(&mut builder.webauthn_challenge_repo)
            .unwrap()
            .expect_take()
            .returning(|_| Box::pin(async { Ok(None) }));

        expect_valid_step_up_token(&mut builder, "valid-step-up-token");

        let service = builder.build();
        let result = service
            .passkey_register_self_service(
                Identity::User(user),
                PasskeyRegisterSelfServiceInput {
                    rp_info: test_rp_info(),
                    credential: test_register_public_key_credential(),
                    step_up_token: "valid-step-up-token".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::WebAuthnMissingChallenge)));
    }

    // ── setup_otp / verify_otp (self-service TOTP) ──────────────────────

    #[tokio::test]
    async fn setup_otp_self_service_generates_secret_and_uri() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_cleanup_expired()
            .returning(|| Box::pin(async { Ok(0u64) }));

        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_start_enrollment()
            .returning(|_, _, _| {
                Box::pin(async {
                    Ok(OtpEnrollment {
                        id: Uuid::new_v4(),
                        user_id: Uuid::new_v4(),
                        secret: String::new(),
                        expires_at: Utc::now(),
                        created_at: Utc::now(),
                    })
                })
            });

        let service = builder.build();
        let result = service
            .setup_otp(
                Identity::User(user),
                SetupOtpInput {
                    issuer: "example.com".to_string(),
                },
            )
            .await;

        assert!(result.is_ok(), "expected Ok from setup_otp");
        let output = result.unwrap();
        assert!(!output.secret.is_empty(), "TOTP secret must be emitted");
        assert!(output.otpauth_uri.starts_with("otpauth://totp/"));
    }

    #[tokio::test]
    async fn verify_otp_self_service_succeeds_with_valid_code() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let secret = generate_secret().expect("generate totp secret");
        let secret_bytes = secret.to_bytes().expect("secret bytes");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("epoch")
            .as_secs();
        let counter = now / 30;
        let code = generate_totp_code(&secret_bytes, counter, 6).expect("totp code");

        // No pre-existing OTP credentials to delete.
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        expect_active_otp_enrollment(&mut builder, user.id, secret.base32_encoded().to_string());
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_claim_enrollment()
            .returning(|_, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .returning(|user_id, credential_type, _, _, _| {
                Box::pin(async move {
                    Ok(Credential {
                        id: Uuid::new_v4(),
                        salt: None,
                        credential_type: if credential_type == "otp" {
                            CredentialType::Otp
                        } else {
                            CredentialType::Password
                        },
                        user_id,
                        user_label: None,
                        secret_data: "secret".to_string(),
                        credential_data: CredentialData::new_hash(1, "argon2".to_string()),
                        temporary: false,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        webauthn_credential_id: None,
                        recovery_code_lookup: None,
                    })
                })
            });

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        // The factor-change notification email looks up the user and realm SMTP
        // config (best-effort; no SMTP configured → EmailNotSent event logged).
        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: code.to_string(),
                    step_up_token: None,
                },
            )
            .await;

        assert!(result.is_ok(), "expected Ok from verify_otp");
    }

    #[tokio::test]
    async fn verify_otp_self_service_sends_factor_change_email_when_smtp_configured() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let secret = generate_secret().expect("generate totp secret");
        let secret_bytes = secret.to_bytes().expect("secret bytes");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("epoch")
            .as_secs();
        let counter = now / 30;
        let code = generate_totp_code(&secret_bytes, counter, 6).expect("totp code");

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        expect_active_otp_enrollment(&mut builder, user.id, secret.base32_encoded().to_string());
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_claim_enrollment()
            .returning(|_, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_create_custom_credential()
            .returning(|user_id, credential_type, _, _, _| {
                Box::pin(async move {
                    Ok(Credential {
                        id: Uuid::new_v4(),
                        salt: None,
                        credential_type: if credential_type == "otp" {
                            CredentialType::Otp
                        } else {
                            CredentialType::Password
                        },
                        user_id,
                        user_label: None,
                        secret_data: "secret".to_string(),
                        credential_data: CredentialData::new_hash(1, "argon2".to_string()),
                        temporary: false,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        webauthn_credential_id: None,
                        recovery_code_lookup: None,
                    })
                })
            });

        Arc::get_mut(&mut builder.user_required_action_repo)
            .unwrap()
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        // SMTP is configured, so the factor-change email is actually sent.
        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let smtp_config = SmtpConfig {
            id: Uuid::new_v4(),
            realm_id: realm.id.into(),
            host: "smtp.example.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
            from_email: "no-reply@example.com".to_string(),
            from_name: "FerrisKey".to_string(),
            encryption: SmtpEncryption::StartTls,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let smtp_clone = smtp_config.clone();
        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(move |_| {
                let s = smtp_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        Arc::get_mut(&mut builder.email_port)
            .unwrap()
            .expect_send_email()
            .times(1)
            .returning(|_, _, _, _, _| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: code.to_string(),
                    step_up_token: None,
                },
            )
            .await;

        assert!(
            result.is_ok(),
            "expected Ok from verify_otp with a factor-change email sent"
        );
    }

    // ── complete_password_reset_with_recovery_code ──────────────────────

    fn test_recovery_credential(user_id: Uuid) -> Credential {
        Credential {
            id: Uuid::new_v4(),
            salt: Some("salt".to_string()),
            credential_type: CredentialType::RecoveryCode,
            user_id,
            user_label: None,
            secret_data: "hashed-secret".to_string(),
            credential_data: CredentialData::new_hash(1, "argon2".to_string()),
            temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
            recovery_code_lookup: None,
        }
    }

    #[tokio::test]
    async fn complete_password_reset_with_recovery_code_succeeds() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let user_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(move |_, _| {
                let u = user_clone.clone();
                Box::pin(async move { Ok(Some(u)) })
            });

        expect_realm_settings(&mut builder, &realm);

        // Password policy lookup: no stored policy → use defaults.
        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let recovery_cred = test_recovery_credential(user.id);
        let recovery_cred_clone = recovery_cred.clone();
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_find_recovery_code_by_lookup()
            .returning(move |_, _| {
                let c = recovery_cred_clone.clone();
                Box::pin(async move { Ok(Some(c)) })
            });

        expect_recovery_code_lookup(&mut builder);

        Arc::get_mut(&mut builder.recovery_code_repo)
            .unwrap()
            .expect_verify()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        // A valid code clears any accumulated lockout counter.
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_reset_failed_login_attempts()
            .returning(|_| Box::pin(async { Ok(()) }));

        // After the recovery code is burned, the service issues a password-reset
        // token (emailed to the user) rather than minting a session or applying
        // the new password inline. Wire the mocks that flow requires.
        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_count_active_by_user_id()
            .returning(|_| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_cleanup_expired()
            .returning(|| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_magic_token()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "hashed".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_create()
            .returning(|_| Box::pin(async { Ok(()) }));

        // No SMTP configured → EmailNotSent + PasswordResetRequested events logged.
        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let user_by_id = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .complete_password_reset_with_recovery_code(
                CompletePasswordResetWithRecoveryCodeInput {
                    realm_name: "test-realm".to_string(),
                    email: "alice@example.com".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                    format: "b32-split-4".to_string(),
                    base_url: "http://localhost:5555".to_string(),
                },
            )
            .await;

        assert!(result.is_ok(), "expected Ok, got an error");
    }

    #[tokio::test]
    async fn complete_password_reset_with_recovery_code_invalid_code_fails() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let realm_settings_clone = create_test_realm_setting(realm.id, true);
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = realm_settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        let user_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(move |_, _| {
                let u = user_clone.clone();
                Box::pin(async move { Ok(Some(u)) })
            });

        // Password policy lookup: no stored policy → use defaults.
        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        // The submitted code maps to no stored recovery-code credential, so the
        // lookup returns None and the burn fails before any Argon2 verification.
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_find_recovery_code_by_lookup()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        expect_recovery_code_lookup(&mut builder);

        // A failed attempt must bump the account's failed-login counter.
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_increment_failed_login_attempts()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        // A failed recovery-code burn must be audited as a failure event.
        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .complete_password_reset_with_recovery_code(
                CompletePasswordResetWithRecoveryCodeInput {
                    realm_name: "test-realm".to_string(),
                    email: "alice@example.com".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                    format: "b32-split-4".to_string(),
                    base_url: "http://localhost:5555".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::RecoveryCodeBurnError(_))));
    }

    #[tokio::test]
    async fn complete_password_reset_with_recovery_code_unknown_email_fails() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let realm_settings_clone = create_test_realm_setting(realm.id, true);
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = realm_settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .complete_password_reset_with_recovery_code(
                CompletePasswordResetWithRecoveryCodeInput {
                    realm_name: "test-realm".to_string(),
                    email: "unknown@example.com".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                    format: "b32-split-4".to_string(),
                    base_url: "http://localhost:5555".to_string(),
                },
            )
            .await;

        // The unknown-email answer must be indistinguishable from a bad code so
        // the anonymous endpoint cannot enumerate accounts.
        assert!(
            matches!(result, Err(CoreError::RecoveryCodeBurnError(_))),
            "unknown email must be masked as a bad-code failure"
        );
    }

    #[tokio::test]
    async fn complete_password_reset_with_recovery_code_requires_forgot_password_enabled() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        // forgot_password disabled → the recovery-code path must refuse before
        // touching users or codes.
        let realm_settings_clone = create_test_realm_setting(realm.id, false);
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = realm_settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .never();

        let service = builder.build();
        let result = service
            .complete_password_reset_with_recovery_code(
                CompletePasswordResetWithRecoveryCodeInput {
                    realm_name: "test-realm".to_string(),
                    email: "alice@example.com".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                    format: "b32-split-4".to_string(),
                    base_url: "http://localhost:5555".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn complete_password_reset_with_recovery_code_falls_back_to_legacy_rows_without_lookup() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let realm_settings_clone = create_test_realm_setting(realm.id, true);
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = realm_settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        let user_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(move |_, _| {
                let u = user_clone.clone();
                Box::pin(async move { Ok(Some(u)) })
            });

        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_find_recovery_code_by_lookup()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let legacy_cred = test_recovery_credential(user.id);
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(move |_| {
                let creds = vec![legacy_cred.clone()];
                Box::pin(async move { Ok(creds) })
            });

        expect_recovery_code_lookup(&mut builder);

        Arc::get_mut(&mut builder.recovery_code_repo)
            .unwrap()
            .expect_verify()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_reset_failed_login_attempts()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_count_active_by_user_id()
            .returning(|_| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_cleanup_expired()
            .returning(|| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_magic_token()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "hashed-reset-token".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_create()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let user_by_id = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .complete_password_reset_with_recovery_code(
                CompletePasswordResetWithRecoveryCodeInput {
                    realm_name: "test-realm".to_string(),
                    email: "alice@example.com".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                    format: "b32-split-4".to_string(),
                    base_url: "http://localhost:5555".to_string(),
                },
            )
            .await;

        assert!(
            result.is_ok(),
            "legacy recovery-code rows without lookup must remain usable after deploy"
        );
    }

    // ── verify_otp: server-side pending secret enforcement (finding #2) ───

    #[tokio::test]
    async fn verify_otp_self_service_fails_without_pending_enrollment() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        // No enrollment was stored by /me/totp/setup -> cannot enroll.
        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_get_active_enrollment()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: "123456".to_string(),
                    step_up_token: None,
                },
            )
            .await;

        // The client-supplied secret is no longer trusted; missing pending
        // secret must surface a 400 (not a panic / 500 / silent enrollment).
        assert!(matches!(result, Err(CoreError::PendingTotpSecretMissing)));
    }

    #[tokio::test]
    async fn verify_otp_self_service_rejects_invalid_step_up_token() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        // No pre-existing OTP credential, so the guard passes and the presented
        // token is the next gate.
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        // A valid step-up token must be presented; an invalid one is rejected
        // before any pending secret is consumed.
        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_find_active()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: "123456".to_string(),
                    step_up_token: Some("not-a-valid-token".to_string()),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::StepUpTokenInvalid)));
    }

    #[tokio::test]
    async fn verify_otp_self_service_invalid_step_up_token_does_not_consume_other_valid_tokens() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let take_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let take_calls_clone = Arc::clone(&take_calls);
        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_find_active()
            .times(2)
            .returning(move |_| {
                let call = take_calls_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Box::pin(async move {
                    Ok(if call == 0 || call == 1 {
                        vec![StepUpTokenRecord {
                            id: Uuid::new_v4(),
                            user_id: Uuid::new_v4(),
                            token_hash: "stored-good-token".to_string(),
                            expires_at: Utc::now() + Duration::minutes(5),
                        }]
                    } else {
                        Vec::new()
                    })
                })
            });

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_verify_magic_token()
            .times(2)
            .returning(|presented, stored| {
                let valid = presented == stored;
                Box::pin(async move { Ok(valid) })
            });

        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_delete_by_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .times(2)
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        Arc::get_mut(&mut builder.otp_enrollment_repo)
            .unwrap()
            .expect_get_active_enrollment()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let service = builder.build();
        let first = service
            .verify_otp(
                Identity::User(user.clone()),
                VerifyOtpInput {
                    code: "123456".to_string(),
                    step_up_token: Some("wrong-token".to_string()),
                },
            )
            .await;

        assert!(
            matches!(first, Err(CoreError::StepUpTokenInvalid)),
            "an invalid presented step-up token must be rejected"
        );

        let second = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: "123456".to_string(),
                    step_up_token: Some("stored-good-token".to_string()),
                },
            )
            .await;

        assert!(
            !matches!(second, Err(CoreError::StepUpTokenInvalid)),
            "rejecting one invalid token must not destroy a different valid token for the user"
        );
    }

    #[tokio::test]
    async fn verify_otp_self_service_rejects_wrong_code() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        // Provide an active enrollment that does NOT match the submitted code.
        let pending_secret = generate_secret().expect("generate totp secret");
        expect_active_otp_enrollment(
            &mut builder,
            user.id,
            pending_secret.base32_encoded().to_string(),
        );

        let service = builder.build();
        let result = service
            .verify_otp(
                Identity::User(user),
                VerifyOtpInput {
                    code: "000000".to_string(),
                    step_up_token: None,
                },
            )
            .await;

        // Wrong code → 400 (InvalidOtpCode), not a silent enrollment.
        assert!(matches!(result, Err(CoreError::InvalidOtpCode)));
    }

    // ── step-up token enforcement on sensitive self-service ops (#1) ──────

    #[tokio::test]
    async fn register_passkey_self_service_rejects_missing_step_up_token() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        // No stored step-up token → consume fails before any WebAuthn work.
        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_find_active()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        let service = builder.build();
        let result = service
            .passkey_register_self_service(
                Identity::User(user),
                PasskeyRegisterSelfServiceInput {
                    rp_info: test_rp_info(),
                    credential: test_register_public_key_credential(),
                    step_up_token: "missing".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::StepUpTokenInvalid)));
    }

    #[tokio::test]
    async fn delete_credential_self_service_rejects_invalid_step_up_token() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        // A stolen access token without a valid step-up token cannot remove a
        // factor.
        Arc::get_mut(&mut builder.step_up_token_repo)
            .unwrap()
            .expect_find_active()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        let service = builder.build();
        let result = service
            .delete_credential_self_service(
                Identity::User(user),
                Uuid::new_v4(),
                "invalid".to_string(),
            )
            .await;

        assert!(matches!(result, Err(CoreError::StepUpTokenInvalid)));
    }

    // ── recovery-code endpoint lockout (finding #4 / #5) ──────────────────

    #[tokio::test]
    async fn complete_password_reset_with_recovery_code_locked_account_rejected() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        // A user already locked out by too many failed attempts.
        let mut user = create_test_user_with_email(&realm, "alice@example.com");
        user.failed_login_attempts = 99;
        user.locked_until = Some(Utc::now() + chrono::Duration::minutes(15));

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let user_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(move |_, _| {
                let u = user_clone.clone();
                Box::pin(async move { Ok(Some(u)) })
            });

        // Password policy lookup: no stored policy → use defaults.
        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let settings_clone = create_test_realm_setting(realm.id, true);
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        let service = builder.build();
        let result = service
            .complete_password_reset_with_recovery_code(
                CompletePasswordResetWithRecoveryCodeInput {
                    realm_name: "test-realm".to_string(),
                    email: "alice@example.com".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                    format: "b32-split-4".to_string(),
                    base_url: "http://localhost:5555".to_string(),
                },
            )
            .await;

        // A locked account cannot be used to brute-force recovery codes.
        assert!(matches!(result, Err(CoreError::AccountLocked)));
    }

    // ── burn_recovery_code uses single-candidate lookup (finding #5) ──────

    #[tokio::test]
    async fn burn_recovery_code_uses_single_lookup_candidate() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let session_code = Uuid::new_v4().to_string();
        let auth_session = AuthSession::new(AuthSessionParams {
            realm_id: realm.id,
            client_id: Uuid::new_v4(),
            redirect_uri: "http://localhost:5555/callback".to_string(),
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: Some("state".to_string()),
            nonce: None,
            user_id: Some(user.id),
            code: None,
            authenticated: false,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
            code_challenge: None,
            code_challenge_method: None,
        });
        let auth_session_for_update = auth_session.clone();
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_get_by_session_code()
            .returning(move |_| {
                let s = auth_session.clone();
                Box::pin(async move { Ok(s) })
            });

        // Exactly one candidate must be returned from the lookup; Argon2 runs
        // only once on that single row (no N× Argon2 scan).
        let recovery_cred = test_recovery_credential(user.id);
        let recovery_cred_clone = recovery_cred.clone();
        let verify_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let verify_calls_clone = verify_calls.clone();
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_find_recovery_code_by_lookup()
            .returning(move |_, _| {
                let c = recovery_cred_clone.clone();
                Box::pin(async move { Ok(Some(c)) })
            });
        expect_recovery_code_lookup(&mut builder);
        Arc::get_mut(&mut builder.recovery_code_repo)
            .unwrap()
            .expect_verify()
            .returning(move |_, _, _, _, _| {
                verify_calls_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Box::pin(async { Ok(true) })
            });

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        expect_realm_settings(&mut builder, &realm);

        // Success resets the accumulated failure counter.
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_reset_failed_login_attempts()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_update_code_and_user_id()
            .returning(move |_, _, _| {
                let s = auth_session_for_update.clone();
                Box::pin(async move { Ok(s) })
            });

        let service = builder.build();
        let result = service
            .burn_recovery_code(
                Identity::User(user),
                BurnRecoveryCodeInput {
                    session_code,
                    format: "b32-split-4".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                },
            )
            .await;

        assert!(result.is_ok(), "expected Ok from burn_recovery_code");
        // Argon2 verification ran exactly once, on the single lookup candidate.
        assert_eq!(
            verify_calls.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "Argon2 should run once per burn, not per stored code"
        );
    }

    #[tokio::test]
    async fn burn_recovery_code_failure_increments_lockout_and_emits_failure_event() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let session_code = Uuid::new_v4().to_string();
        let auth_session = AuthSession::new(AuthSessionParams {
            realm_id: realm.id,
            client_id: Uuid::new_v4(),
            redirect_uri: "http://localhost:5555/callback".to_string(),
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: Some("state".to_string()),
            nonce: None,
            user_id: Some(user.id),
            code: None,
            authenticated: false,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
            code_challenge: None,
            code_challenge_method: None,
        });
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_get_by_session_code()
            .returning(move |_| {
                let s = auth_session.clone();
                Box::pin(async move { Ok(s) })
            });

        expect_realm_settings(&mut builder, &realm);
        expect_recovery_code_lookup(&mut builder);

        // No candidate matches the submitted code.
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_find_recovery_code_by_lookup()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_get_credentials_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        // The failed guess must bump the lockout counter…
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_increment_failed_login_attempts()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        // …and be audited as a RecoveryCodeBurned *failure* event.
        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .burn_recovery_code(
                Identity::User(user),
                BurnRecoveryCodeInput {
                    session_code,
                    format: "b32-split-4".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::RecoveryCodeBurnError(_))));
    }

    #[tokio::test]
    async fn burn_recovery_code_locked_account_rejected() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let mut user = create_test_user_with_email(&realm, "alice@example.com");
        user.failed_login_attempts = 99;
        user.locked_until = Some(Utc::now() + chrono::Duration::minutes(15));

        let session_code = Uuid::new_v4().to_string();
        let auth_session = AuthSession::new(AuthSessionParams {
            realm_id: realm.id,
            client_id: Uuid::new_v4(),
            redirect_uri: "http://localhost:5555/callback".to_string(),
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: Some("state".to_string()),
            nonce: None,
            user_id: Some(user.id),
            code: None,
            authenticated: false,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
            code_challenge: None,
            code_challenge_method: None,
        });
        Arc::get_mut(&mut builder.auth_session_repo)
            .unwrap()
            .expect_get_by_session_code()
            .returning(move |_| {
                let s = auth_session.clone();
                Box::pin(async move { Ok(s) })
            });

        expect_realm_settings(&mut builder, &realm);

        // A locked account cannot be used to brute-force recovery codes during
        // the MFA login fallback either.
        Arc::get_mut(&mut builder.recovery_code_repo)
            .unwrap()
            .expect_lookup_of()
            .never();

        let service = builder.build();
        let result = service
            .burn_recovery_code(
                Identity::User(user),
                BurnRecoveryCodeInput {
                    session_code,
                    format: "b32-split-4".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::AccountLocked)));
    }

    // ── issue_password_reset_token_and_notify behaviour (finding #3) ──────

    #[tokio::test]
    async fn complete_password_reset_with_recovery_code_mints_reset_token_not_session() {
        let mut builder = TridentTestBuilder::new();
        let realm = create_test_realm_with_name("test-realm");
        let user = create_test_user_with_email(&realm, "alice@example.com");

        let realm_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_name()
            .returning(move |_| {
                let r = realm_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        let settings_clone = create_test_realm_setting(realm.id, true);
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_realm_settings()
            .returning(move |_| {
                let s = settings_clone.clone();
                Box::pin(async move { Ok(Some(s)) })
            });

        let user_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_email()
            .returning(move |_, _| {
                let u = user_clone.clone();
                Box::pin(async move { Ok(Some(u)) })
            });

        // Password policy lookup: no stored policy → use defaults.
        Arc::get_mut(&mut builder.password_policy_repo)
            .unwrap()
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let recovery_cred = test_recovery_credential(user.id);
        let recovery_cred_clone = recovery_cred.clone();
        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_find_recovery_code_by_lookup()
            .returning(move |_, _| {
                let c = recovery_cred_clone.clone();
                Box::pin(async move { Ok(Some(c)) })
            });

        expect_recovery_code_lookup(&mut builder);

        Arc::get_mut(&mut builder.recovery_code_repo)
            .unwrap()
            .expect_verify()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(true) }));

        Arc::get_mut(&mut builder.credential_repo)
            .unwrap()
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));

        // A valid code clears any accumulated lockout counter.
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_reset_failed_login_attempts()
            .returning(|_| Box::pin(async { Ok(()) }));

        // The reset flow looks up the user and realm, counts/cleans tokens, then
        // creates exactly one password-reset token (no session is minted).
        let user_by_id_clone = user.clone();
        Arc::get_mut(&mut builder.user_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let u = user_by_id_clone.clone();
                Box::pin(async move { Ok(u) })
            });

        let realm_by_id_clone = realm.clone();
        Arc::get_mut(&mut builder.realm_repo)
            .unwrap()
            .expect_get_by_id()
            .returning(move |_| {
                let r = realm_by_id_clone.clone();
                Box::pin(async move { Ok(Some(r)) })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_count_active_by_user_id()
            .returning(|_| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_cleanup_expired()
            .returning(|| Box::pin(async { Ok(0) }));

        Arc::get_mut(&mut builder.hasher_repo)
            .unwrap()
            .expect_hash_magic_token()
            .returning(|_| {
                Box::pin(async {
                    Ok(HashResult::new(
                        "hashed".to_string(),
                        "salt".to_string(),
                        1,
                        "argon2".to_string(),
                    ))
                })
            });

        Arc::get_mut(&mut builder.prt_repo)
            .unwrap()
            .expect_create()
            .returning(|_| Box::pin(async { Ok(()) }));

        // No SMTP configured → EmailNotSent + PasswordResetRequested events logged.
        Arc::get_mut(&mut builder.smtp_config_repo)
            .unwrap()
            .expect_get_by_realm_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        Arc::get_mut(&mut builder.webhook_repo)
            .unwrap()
            .expect_notify()
            .returning(|_, _: WebhookPayload<()>| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .complete_password_reset_with_recovery_code(
                CompletePasswordResetWithRecoveryCodeInput {
                    realm_name: "test-realm".to_string(),
                    email: "alice@example.com".to_string(),
                    code: "abcd-efgh-ij9m-nopq".to_string(),
                    format: "b32-split-4".to_string(),
                    base_url: "http://localhost:5555".to_string(),
                },
            )
            .await;

        assert!(result.is_ok(), "expected Ok from recovery-code reset");
        // The output link carries the password-reset token, *not* a session.
        let output = result.unwrap();
        let login_url = output.login_url.expect("reset link must be present");
        assert!(
            login_url.contains("reset-password?token_id="),
            "expected a reset link, got: {login_url}"
        );
    }
}
