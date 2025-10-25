use std::time::{SystemTime, UNIX_EPOCH};

use futures::future::try_join_all;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha1::Sha1;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::{
    domain::{
        authentication::{
            entities::{AuthSession, WebAuthnChallenge},
            ports::AuthSessionRepository,
            value_objects::Identity,
        },
        client::ports::{ClientRepository, RedirectUriRepository},
        common::{entities::app_errors::CoreError, generate_random_string, services::Service},
        credential::{
            entities::{Credential, CredentialData},
            ports::CredentialRepository,
        },
        crypto::ports::HasherRepository,
        health::ports::HealthCheckRepository,
        jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
        realm::ports::RealmRepository,
        role::ports::RoleRepository,
        trident::{
            entities::{MfaRecoveryCode, TotpSecret},
            ports::{
                BurnRecoveryCodeInput, BurnRecoveryCodeOutput, ChallengeOtpInput,
                ChallengeOtpOutput, GenerateRecoveryCodeInput, GenerateRecoveryCodeOutput,
                RecoveryCodeFormatter, RecoveryCodeRepository, SetupOtpInput, SetupOtpOutput,
                TridentService, UpdatePasswordInput, VerifyOtpInput, VerifyOtpOutput,
                WebAuthnPublicKeyAuthenticateInput, WebAuthnPublicKeyAuthenticateOutput,
                WebAuthnPublicKeyCreateOptionsInput, WebAuthnPublicKeyCreateOptionsOutput,
                WebAuthnPublicKeyRequestOptionsInput, WebAuthnPublicKeyRequestOptionsOutput,
                WebAuthnRpInfo, WebAuthnValidatePublicKeyInput, WebAuthnValidatePublicKeyOutput,
            },
        },
        user::{
            entities::RequiredAction,
            ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
        },
        webhook::ports::{WebhookNotifierRepository, WebhookRepository},
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
        tracing::error!("failed to parse code");
        return Ok(false);
    };

    let Ok(secret_bytes) = secret.to_bytes() else {
        tracing::error!("failed to convert secret to bytes");
        return Ok(false);
    };

    let time_step = 30;
    let digits = 6;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_secs();

    let counter = now / time_step;

    for i in -1..=1 {
        let adjusted_counter = counter.wrapping_add(i as u64);
        let generated = generate_totp_code(&secret_bytes, adjusted_counter, digits)?;
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
        tracing::error!("Failed to parse server_host as URL: {e}");
        CoreError::InternalServerError
    })?;

    Ok(WebauthnBuilder::new(&rp_info.rp_id, &rp_url)
        .map_err(|e| {
            tracing::error!("Failed to build Webauthn client: {e:?}");
            CoreError::InternalServerError
        })?
        .build()
        .map_err(|e| {
            tracing::error!("Failed to build Webauthn client: {e:?}");
            CoreError::InternalServerError
        })?)
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
impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC> TridentService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    WN: WebhookNotifierRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
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
                tracing::error!("{e}");
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
            tracing::error!("Failed to delete previously fetched credentials: {e}");
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
            .filter(|cred| cred.credential_type == "recovery-code")
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
                tracing::error!(
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
                tracing::error!("Failed to delete a credential even though it was just fetched with the same repository: {e}");
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
                tracing::error!("Failed to generate webauthn challenge: {e:?}");
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
                    tracing::debug!("Failed to complete passkey registration: {e:?}");
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
                        Ok(Passkey::from(credential))
                    },
                    _ => {
                        tracing::error!("A Webauthn credential doesn't hold WebAuthn credential data ! Something went wrong during creation...");
                        Err(CoreError::InternalServerError)
                    }
                }
            )
            .collect::<Result<Vec<Passkey>, CoreError>>()?;

        let (rcr, pa) = webauthn.start_passkey_authentication(&creds).map_err(|e| {
            tracing::error!("Failed to generate webauthn challenge: {e:?}");
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
                .finish_passkey_authentication(&input.credential, &pa)
                .map_err(|e| {
                    tracing::error!("Error during webauthn verification: {e:?}");
                    CoreError::InternalServerError
                }),
            _ => Err(CoreError::Invalid),
        }?;

        if auth_result.needs_update() {
            let _ = self
                .credential_repository
                .update_webauthn_credential(&auth_result)
                .await
                .map_err(|e| {
                    tracing::debug!("{e:?}");
                    CoreError::InternalServerError
                })?;
        }

        if !auth_result.user_verified() {
            return Err(CoreError::WebAuthnChallengeFailed);
        }

        let login_url = store_auth_code_and_generate_login_url::<AS>(
            &self.auth_session_repository,
            &auth_session,
            user.id.clone(),
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
            .find(|cred| cred.credential_type == "otp")
            .ok_or_else(|| {
                CoreError::TotpVerificationFailed("user has not OTP configured".to_string())
            })?;

        let secret = TotpSecret::from_base32(&otp_credential.secret_data);

        let is_valid = verify(&secret, &input.code)?;

        if !is_valid {
            tracing::error!("invalid OTP code for user: {}", user.email);
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
            tracing::error!("invalid OTP code");
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
}
