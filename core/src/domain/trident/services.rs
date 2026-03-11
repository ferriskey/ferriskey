use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{Duration, Utc};
use ferriskey_domain::generate_uuid_v7;
use futures::future::try_join_all;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha1::Sha1;
use tracing::{debug, error, warn};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::{
    domain::{
        authentication::{
            entities::{AuthSession, WebAuthnChallenge},
            ports::AuthSessionRepository,
            value_objects::Identity,
        },
        common::{
            email::EmailPort, entities::app_errors::CoreError, generate_random_string,
            generate_random_token,
        },
        credential::{
            entities::{Credential, CredentialData, CredentialType},
            ports::CredentialRepository,
        },
        crypto::HasherRepository,
        realm::{
            entities::RealmId,
            ports::{RealmRepository, SmtpConfigRepository},
        },
        seawatch::{
            entities::{EventStatus, SecurityEvent, SecurityEventType},
            ports::SecurityEventRepository,
        },
        trident::{
            entities::{MfaRecoveryCode, PasswordResetToken, TotpSecret},
            ports::{
                BurnRecoveryCodeInput, BurnRecoveryCodeOutput, ChallengeOtpInput,
                ChallengeOtpOutput, CompletePasswordResetInput, CompletePasswordResetOutput,
                GenerateRecoveryCodeInput, GenerateRecoveryCodeOutput, MagicLinkInput,
                MagicLinkRepository, PasswordResetTokenRepository, RecoveryCodeFormatter,
                RecoveryCodeRepository, RequestPasswordResetInput, SetupOtpInput, SetupOtpOutput,
                TridentService, UpdatePasswordInput, VerifyMagicLinkInput, VerifyOtpInput,
                VerifyOtpOutput, VerifyResetTokenInput, WebAuthnPublicKeyAuthenticateInput,
                WebAuthnPublicKeyAuthenticateOutput, WebAuthnPublicKeyCreateOptionsInput,
                WebAuthnPublicKeyCreateOptionsOutput, WebAuthnPublicKeyRequestOptionsInput,
                WebAuthnPublicKeyRequestOptionsOutput, WebAuthnRpInfo,
                WebAuthnValidatePublicKeyInput, WebAuthnValidatePublicKeyOutput,
            },
        },
        user::{
            entities::RequiredAction,
            ports::{UserRepository, UserRequiredActionRepository},
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

/// Generates a random authorization code, stores it in the user auth session
/// and returns it in a formated URL ready to be sent to the user
async fn store_auth_code_and_generate_login_url<AS: AuthSessionRepository>(
    auth_session_repository: &AS,
    auth_session: &AuthSession,
    user_id: Uuid,
) -> Result<String, CoreError> {
    let authorization_code = generate_random_string();

    auth_session_repository
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

#[derive(Clone, Debug)]
pub struct TridentServiceImpl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH>
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
}

impl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH>
    TridentServiceImpl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH>
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
        }
    }
}

impl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH> TridentService
    for TridentServiceImpl<CR, RC, AS, H, URA, ML, UR, RR, ES, SC, PRT, SE, WH>
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

        let user_credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        let recovery_code_creds = user_credentials
            .into_iter()
            .filter(|cred| cred.credential_type == CredentialType::RecoveryCode)
            .collect::<Vec<Credential>>();

        // This is a suboptimal way to do it but I was having ownership errors
        let mut burnt_code: Option<Credential> = None;
        for code_cred in recovery_code_creds.into_iter() {
            if let CredentialData::Hash {
                hash_iterations,
                algorithm,
            } = &code_cred.credential_data
            {
                let salt = code_cred
                    .salt
                    .as_ref()
                    .ok_or(CoreError::InternalServerError)?;

                let result = self
                    .recovery_code_repository
                    .verify(
                        &user_code,
                        &code_cred.secret_data,
                        *hash_iterations,
                        algorithm,
                        salt,
                    )
                    .await?;

                if result {
                    burnt_code = Some(code_cred);
                    break;
                }
            } else {
                error!(
                    "A recovery code credential has no Hash credential data. This is a server bug. Do not forward this message back to the user"
                );
                return Err(CoreError::InternalServerError);
            }
        }

        // This doesn't check if there are multiple matches because it is not necessarly a bug
        // It is highly unlikely but a user may have multiple identical recovery codes
        // or it could also be a duplicate storage bug.
        // Anyway, this is not the place to check such a bug
        let burnt_code = burnt_code.ok_or_else(|| {
            CoreError::RecoveryCodeBurnError(
                "The provided code is invalid or has already been used".to_string(),
            )
        })?;

        self
            .credential_repository
            .delete_by_id(burnt_code.id)
            .await
            .map_err(|e| {
                error!("Failed to delete a credential even though it was just fetched with the same repository: {e}");
                CoreError::InternalServerError
            })?;

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
                Some(filtered)
            }
        };

        let (ccr, pr) = webauthn
            .start_passkey_registration(user.id, &user.email, &user.username, credentials)
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

        let session_code =
            Uuid::parse_str(&input.session_code).map_err(|_| CoreError::SessionCreateError)?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

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

        let login_url = store_auth_code_and_generate_login_url::<AS>(
            &self.auth_session_repository,
            &auth_session,
            user.id,
        )
        .await?;

        Ok(WebAuthnPublicKeyAuthenticateOutput { login_url })
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
            error!("invalid OTP code for user: {}", user.email);
            return Err(CoreError::TotpVerificationFailed(
                "failed to verify OTP".to_string(),
            ));
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

        Ok(ChallengeOtpOutput { login_url })
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
        let otpauth_uri = generate_otpauth_uri(&input.issuer, &user.email, &secret);

        Ok(SetupOtpOutput {
            otpauth_uri,
            secret: secret.base32_encoded().to_string(),
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

        Ok(())
    }

    async fn verify_otp(
        &self,
        identity: Identity,
        input: VerifyOtpInput,
    ) -> Result<VerifyOtpOutput, CoreError> {
        let decoded = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &input.secret)
            .ok_or(CoreError::InternalServerError)?;

        if decoded.len() != 20 {
            return Err(CoreError::InternalServerError);
        }

        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(CoreError::InternalServerError),
        };

        let secret = TotpSecret::from_base32(&input.secret);

        let is_valid = verify(&secret, &input.code)?;

        if !is_valid {
            error!("invalid OTP code");
            return Err(CoreError::InternalServerError);
        }

        let credential_data = serde_json::json!({
          "subType": "totp",
          "digits": 6,
          "counter": 0,
          "period": 30,
          "algorithm": "HmacSha256",
        });

        self.credential_repository
            .create_custom_credential(
                user.id,
                "otp".to_string(),
                secret.base32_encoded().to_string(),
                input.label,
                credential_data,
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.user_required_action_repository
            .remove_required_action(user.id, RequiredAction::ConfigureOtp)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(VerifyOtpOutput {
            message: "OTP verified successfully".to_string(),
            user_id: user.id,
        })
    }

    async fn generate_magic_link(&self, input: MagicLinkInput) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
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
        self.magic_link_repository
            .create_magic_link(
                user.id,
                realm.id.into(),
                magic_token_id,
                &magic_token_hash,
                expires_at,
            )
            .await?;

        match self.smtp_config_repository.get_by_realm_id(realm.id).await {
            Ok(Some(smtp_config)) => {
                let subject = "Your magic link";
                let body = format!(
                    "Your magic link token: {}\nThis link expires in {} minutes.",
                    magic_token, ttl_minutes
                );
                if let Err(e) = self
                    .email_port
                    .send_email(&smtp_config, &user.email, subject, &body)
                    .await
                {
                    warn!("Failed to send magic link email: {}", e);
                }
            }
            _ => {
                warn!("SMTP not configured for realm, logging magic link instead");
                debug!(
                    "Magic link token_id: {}, token: {}",
                    magic_token_id, magic_token
                );
            }
        }

        Ok(())
    }

    async fn verify_magic_link(&self, input: VerifyMagicLinkInput) -> Result<String, CoreError> {
        let session_code = Uuid::parse_str(&input.session_code).map_err(|_| {
            error!("Failed to parse session code");
            CoreError::SessionCreateError
        })?;

        // Fetch the auth session first
        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .inspect_err(|_| error!("Session not found for code: {}", session_code))
            .map_err(|_| CoreError::SessionNotFound)?;

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
        let login_url = store_auth_code_and_generate_login_url::<AS>(
            &self.auth_session_repository,
            &auth_session,
            magic_link.user_id,
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
            .get_by_name(input.realm_name)
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

        let prt = PasswordResetToken {
            id: generate_uuid_v7(),
            user_id: user.id,
            realm_id: realm.id.into(),
            token_id,
            token_hash: token_hash.hash,
            created_at: Utc::now(),
            expires_at,
        };

        self.password_reset_token_repository.create(&prt).await?;

        match self.smtp_config_repository.get_by_realm_id(realm.id).await {
            Ok(Some(smtp_config)) => {
                let subject = "Reset your password";
                let body = format!(
                    "A password reset was requested for your account.\n\nClick the link below to reset your password:\n{}/realms/{}/authentication/reset-password?token_id={}&token={}\n\nThis link expires in {} minutes.\n\nIf you did not request this, please ignore this email.",
                    input.base_url, realm.name, token_id, raw_token, ttl_minutes
                );
                if let Err(e) = self
                    .email_port
                    .send_email(&smtp_config, &user.email, subject, &body)
                    .await
                {
                    warn!("Failed to send password reset email: {}", e);
                }
            }
            _ => {
                warn!("SMTP not configured for realm, logging password reset token instead");
                debug!(
                    "Password reset token_id: {}, token: {}",
                    token_id, raw_token
                );
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

        // 3. Delete old password credential
        let _ = self
            .credential_repository
            .delete_password_credential(prt.user_id)
            .await;

        // 4. Create new hashed credential
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

        Ok(CompletePasswordResetOutput { user_id, realm_id })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        authentication::ports::MockAuthSessionRepository,
        common::{email::MockEmailPort, services::tests::create_test_realm_with_name},
        credential::ports::MockCredentialRepository,
        realm::ports::{MockRealmRepository, MockSmtpConfigRepository},
        seawatch::ports::MockSecurityEventRepository,
        trident::ports::{
            MockMagicLinkRepository, MockPasswordResetTokenRepository, MockRecoveryCodeRepository,
        },
        user::ports::{MockUserRepository, MockUserRequiredActionRepository},
        webhook::ports::MockWebhookRepository,
    };
    use ferriskey_domain::realm::RealmSetting;
    use ferriskey_security::crypto::{entities::HashResult, ports::MockHasherRepository};

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
            }
        }

        fn build(
            self,
        ) -> TridentServiceImpl<
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
        > {
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

        Arc::get_mut(&mut builder.security_event_repo)
            .unwrap()
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        let service = builder.build();
        let result = service
            .request_password_reset(RequestPasswordResetInput {
                realm_name: "test-realm".to_string(),
                email: "user@example.com".to_string(),
                base_url: "http://localhost:5555".to_string(),
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
            })
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    // ── complete_password_reset ─────────────────────────────────────────

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

        let service = builder.build();
        let result = service
            .complete_password_reset(CompletePasswordResetInput {
                token_id,
                token: "raw_token".to_string(),
                new_password: "newpassword123".to_string(),
            })
            .await;

        assert!(result.is_ok());
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
}
