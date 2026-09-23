use std::sync::Arc;

use chrono::{Duration, Utc};
use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::credential::entities::{Credential, CredentialType};
use ferriskey_domain::credential::ports::CredentialRepository;
use ferriskey_domain::elevation::entities::{Elevated, ElevationId};
use ferriskey_domain::elevation::ports::ElevationRepository;
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::realm::scope::{RealmScope, Scoped};
use ferriskey_domain::user::entities::{RequiredAction, User};
use ferriskey_domain::user::ports::{
    UserRepository, UserRequiredActionRepository, UserRoleRepository,
};
use ferriskey_password_policy::entity::PasswordPolicy;
use ferriskey_password_policy::repository::PasswordPolicyRepository;
use ferriskey_password_policy::service::violations_to_core_error;
use ferriskey_password_policy::validator;
use ferriskey_security::crypto::password_check::verify_user_password;
use ferriskey_security::crypto::ports::HasherRepository;
use ferriskey_trident::entities::TotpSecret;
use ferriskey_trident::factor_removal::{
    AccountFactors, FactorRemoval, RemovalRefusal, refusal_for_removal,
};
use tracing::warn;
use uuid::Uuid;
use webauthn_rs::prelude::CredentialID;

use crate::domain::account_security::entities::{
    ChangeOwnPasswordInput, ConfirmOwnOtpEnrollmentInput, ConfirmOwnPasskeyRegistrationInput,
    DeleteOwnPasskeyInput, DisableOwnOtpInput, ElevationProof, ListOwnCredentialsInput,
    OwnCredential, RequestElevationInput, RequestElevationOutput, StartOwnOtpEnrollmentInput,
    StartOwnOtpEnrollmentOutput, StartOwnPasskeyRegistrationInput,
    StartOwnPasskeyRegistrationOutput,
};
use crate::domain::account_security::ports::{
    AccountSecurityService, OtherSessionsRevocationPort, PasskeyRegistrationRepository,
};
use crate::domain::trident::ports::OtpEnrollmentRepository;
use crate::domain::trident::services::{
    build_webauthn_client, generate_otpauth_uri, generate_secret, verify,
};

pub const ELEVATION_TTL_MINUTES: i64 = 5;
pub const OTP_ENROLLMENT_TTL_MINUTES: i64 = 10;
pub const PASSKEY_REGISTRATION_TTL_MINUTES: i64 = 5;

pub struct AccountSecurityServiceImpl<CR, H, UR, RR, PPR, OER, URR, ER, OSR, URA, PRR>
where
    CR: CredentialRepository,
    H: HasherRepository,
    UR: UserRepository,
    RR: RealmRepository,
    PPR: PasswordPolicyRepository,
    OER: OtpEnrollmentRepository,
    URR: UserRoleRepository,
    ER: ElevationRepository,
    OSR: OtherSessionsRevocationPort,
    URA: UserRequiredActionRepository,
    PRR: PasskeyRegistrationRepository,
{
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) user_repository: Arc<UR>,
    pub(crate) realm_repository: Arc<RR>,
    pub(crate) password_policy_repository: Arc<PPR>,
    pub(crate) otp_enrollment_repository: Arc<OER>,
    pub(crate) user_role_repository: Arc<URR>,
    pub(crate) elevation_repository: Arc<ER>,
    pub(crate) other_sessions_revocation: Arc<OSR>,
    pub(crate) user_required_action_repository: Arc<URA>,
    pub(crate) passkey_registration_repository: Arc<PRR>,
}

impl<CR, H, UR, RR, PPR, OER, URR, ER, OSR, URA, PRR>
    AccountSecurityServiceImpl<CR, H, UR, RR, PPR, OER, URR, ER, OSR, URA, PRR>
where
    CR: CredentialRepository,
    H: HasherRepository,
    UR: UserRepository,
    RR: RealmRepository,
    PPR: PasswordPolicyRepository,
    OER: OtpEnrollmentRepository,
    URR: UserRoleRepository,
    ER: ElevationRepository,
    OSR: OtherSessionsRevocationPort,
    URA: UserRequiredActionRepository,
    PRR: PasskeyRegistrationRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        credential_repository: Arc<CR>,
        hasher_repository: Arc<H>,
        user_repository: Arc<UR>,
        realm_repository: Arc<RR>,
        password_policy_repository: Arc<PPR>,
        otp_enrollment_repository: Arc<OER>,
        user_role_repository: Arc<URR>,
        elevation_repository: Arc<ER>,
        other_sessions_revocation: Arc<OSR>,
        user_required_action_repository: Arc<URA>,
        passkey_registration_repository: Arc<PRR>,
    ) -> Self {
        Self {
            credential_repository,
            hasher_repository,
            user_repository,
            realm_repository,
            password_policy_repository,
            otp_enrollment_repository,
            user_role_repository,
            elevation_repository,
            other_sessions_revocation,
            user_required_action_repository,
            passkey_registration_repository,
        }
    }

    async fn caller(
        &self,
        identity: &Identity,
        realm_name: &str,
    ) -> Result<(RealmScope, Scoped<User>), CoreError> {
        if !identity.is_regular_user() {
            return Err(CoreError::Forbidden("is not user".to_string()));
        }

        let scope = RealmScope::resolve(self.realm_repository.as_ref(), realm_name).await?;

        let user = self
            .user_repository
            .get_by_id(identity.id())
            .await?
            .in_realm(&scope)?;

        Ok((scope, user))
    }

    async fn claim_elevation(
        &self,
        scope: &RealmScope,
        caller: Uuid,
        elevation_id: Uuid,
    ) -> Result<Elevated, CoreError> {
        ferriskey_domain::elevation::claim(
            self.elevation_repository.as_ref(),
            scope,
            caller,
            ElevationId::new(elevation_id),
            Utc::now(),
        )
        .await
    }

    async fn credentials_of(&self, user_id: Uuid) -> Result<Vec<Credential>, CoreError> {
        self.credential_repository
            .get_credentials_by_user_id(user_id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)
    }

    async fn factors_of(&self, user_id: Uuid) -> Result<AccountFactors, CoreError> {
        let credentials = self.credentials_of(user_id).await?;

        Ok(AccountFactors {
            has_password: credentials
                .iter()
                .any(|c| c.credential_type == CredentialType::Password),
            passkey_count: credentials
                .iter()
                .filter(|c| c.credential_type == CredentialType::WebAuthnPublicKeyCredential)
                .count(),
            has_otp: credentials
                .iter()
                .any(|c| c.credential_type == CredentialType::Otp),
            federated_identity_count: 0,
        })
    }

    async fn refuse_lockout(
        &self,
        scope: &RealmScope,
        user: &Scoped<User>,
        removal: FactorRemoval,
    ) -> Result<(), CoreError> {
        let factors = self.factors_of(user.get().id).await?;

        let settings = self.realm_repository.get_realm_settings(scope.id()).await?;

        let roles = self
            .user_role_repository
            .get_user_roles(user.get().id)
            .await?;

        match refusal_for_removal(factors, removal, settings.as_ref(), &roles) {
            None => Ok(()),
            Some(RemovalRefusal::LastSignInMeans) => {
                warn!(
                    user_id = %user.get().id,
                    "Refused a credential removal that would leave the account with no way to sign in"
                );
                Err(CoreError::LastSignInMeans)
            }
            Some(RemovalRefusal::MfaRequired) => {
                warn!(
                    user_id = %user.get().id,
                    "Refused a credential removal that would drop the second factor this realm requires"
                );
                Err(CoreError::MfaFactorRequired)
            }
        }
    }

    async fn otp_secret_of(&self, user_id: Uuid) -> Result<Option<TotpSecret>, CoreError> {
        Ok(self
            .credentials_of(user_id)
            .await?
            .into_iter()
            .find(|c| c.credential_type == CredentialType::Otp)
            .map(|c| TotpSecret::from_base32(&c.secret_data)))
    }
}

impl<CR, H, UR, RR, PPR, OER, URR, ER, OSR, URA, PRR> AccountSecurityService
    for AccountSecurityServiceImpl<CR, H, UR, RR, PPR, OER, URR, ER, OSR, URA, PRR>
where
    CR: CredentialRepository,
    H: HasherRepository,
    UR: UserRepository,
    RR: RealmRepository,
    PPR: PasswordPolicyRepository,
    OER: OtpEnrollmentRepository,
    URR: UserRoleRepository,
    ER: ElevationRepository,
    OSR: OtherSessionsRevocationPort,
    URA: UserRequiredActionRepository,
    PRR: PasskeyRegistrationRepository,
{
    async fn request_elevation(
        &self,
        identity: Identity,
        input: RequestElevationInput,
    ) -> Result<RequestElevationOutput, CoreError> {
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let accepted = match &input.proof {
            ElevationProof::Password(password) => {
                verify_user_password(
                    self.credential_repository.as_ref(),
                    self.hasher_repository.as_ref(),
                    user_id,
                    password,
                )
                .await?
            }
            ElevationProof::Otp(code) => match self.otp_secret_of(user_id).await? {
                Some(secret) => verify(&secret, code)?,
                None => false,
            },
        };

        if !accepted {
            warn!(user_id = %user_id, "Refused an elevation request: the proof did not check out");
            return Err(CoreError::InvalidPassword);
        }

        let elevation = self
            .elevation_repository
            .start(
                user_id,
                scope.id(),
                Utc::now() + Duration::minutes(ELEVATION_TTL_MINUTES),
            )
            .await?;

        Ok(RequestElevationOutput {
            elevation_id: elevation.id.into(),
            expires_at: elevation.expires_at,
        })
    }

    async fn change_own_password(
        &self,
        identity: Identity,
        input: ChangeOwnPasswordInput,
    ) -> Result<(), CoreError> {
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let elevated = self
            .claim_elevation(&scope, user_id, input.elevation_id)
            .await?;

        let target = elevated.user_id();

        let current_is_valid = verify_user_password(
            self.credential_repository.as_ref(),
            self.hasher_repository.as_ref(),
            target,
            &input.current_password,
        )
        .await?;

        if !current_is_valid {
            warn!(user_id = %user_id, "Refused a password change: the current password did not check out");
            return Err(CoreError::InvalidPassword);
        }

        let policy = self
            .password_policy_repository
            .find_by_realm_id(scope.id().into())
            .await?
            .unwrap_or_else(|| PasswordPolicy::default(scope.id().into()));

        let email_local = user
            .get()
            .email
            .as_deref()
            .and_then(|email| email.split('@').next())
            .map(str::to_string);

        validator::validate(
            &input.new_password,
            &policy,
            Some(user.get().username.as_str()),
            email_local.as_deref(),
        )
        .map_err(violations_to_core_error)?;

        self.credential_repository
            .delete_password_credential(target)
            .await
            .map_err(|_| CoreError::DeleteCredentialError)?;

        let hash_result = self
            .hasher_repository
            .hash_password(&input.new_password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(target, "password".into(), hash_result, "".into(), false)
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        self.user_required_action_repository
            .remove_required_action(target, RequiredAction::UpdatePassword)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.other_sessions_revocation
            .revoke_all_sessions_except(&user, input.keep_session_id)
            .await?;

        Ok(())
    }

    async fn start_own_otp_enrollment(
        &self,
        identity: Identity,
        input: StartOwnOtpEnrollmentInput,
    ) -> Result<StartOwnOtpEnrollmentOutput, CoreError> {
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let elevated = self
            .claim_elevation(&scope, user_id, input.elevation_id)
            .await?;

        let target = elevated.user_id();

        let secret = generate_secret()?;
        let otpauth_uri = generate_otpauth_uri(
            &input.issuer,
            user.get().email.as_deref().unwrap_or(""),
            &secret,
        );
        let secret = secret.base32_encoded().to_string();

        self.otp_enrollment_repository
            .start_enrollment(
                target,
                secret.clone(),
                Utc::now() + Duration::minutes(OTP_ENROLLMENT_TTL_MINUTES),
            )
            .await?;

        Ok(StartOwnOtpEnrollmentOutput {
            secret,
            otpauth_uri,
        })
    }

    async fn confirm_own_otp_enrollment(
        &self,
        identity: Identity,
        input: ConfirmOwnOtpEnrollmentInput,
    ) -> Result<(), CoreError> {
        let (_, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let enrollment = self
            .otp_enrollment_repository
            .consume_enrollment(user_id, Utc::now())
            .await?
            .ok_or_else(|| {
                warn!(user_id = %user_id, "Refused an OTP confirmation: no live enrolment to claim");
                CoreError::TotpVerificationFailed("no pending OTP enrollment for this user".into())
            })?;

        let secret = TotpSecret::from_base32(&enrollment.secret);

        if !verify(&secret, &input.code)? {
            warn!(user_id = %user_id, "Refused an OTP confirmation: invalid code");
            return Err(CoreError::TotpVerificationFailed(
                "failed to verify OTP".into(),
            ));
        }

        for credential in self.credentials_of(user_id).await? {
            if credential.credential_type == CredentialType::Otp {
                self.credential_repository
                    .delete_by_id(&user, credential.id)
                    .await
                    .map_err(|_| CoreError::DeleteCredentialError)?;
            }
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
                user_id,
                CredentialType::Otp.to_string(),
                enrollment.secret,
                input.label,
                credential_data,
            )
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        self.user_required_action_repository
            .remove_required_action(user_id, RequiredAction::ConfigureOtp)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn disable_own_otp(
        &self,
        identity: Identity,
        input: DisableOwnOtpInput,
    ) -> Result<(), CoreError> {
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let elevated = self
            .claim_elevation(&scope, user_id, input.elevation_id)
            .await?;

        let target = elevated.user_id();

        self.refuse_lockout(&scope, &user, FactorRemoval::Otp)
            .await?;

        for credential in self.credentials_of(target).await? {
            if credential.credential_type == CredentialType::Otp {
                self.credential_repository
                    .delete_by_id(&user, credential.id)
                    .await
                    .map_err(|_| CoreError::DeleteCredentialError)?;
            }
        }

        let _ = self
            .otp_enrollment_repository
            .clear_enrollments(target)
            .await;

        Ok(())
    }

    async fn list_own_credentials(
        &self,
        identity: Identity,
        input: ListOwnCredentialsInput,
    ) -> Result<Vec<OwnCredential>, CoreError> {
        let (_, user) = self.caller(&identity, &input.realm_name).await?;

        Ok(self
            .credentials_of(user.get().id)
            .await?
            .into_iter()
            .map(|credential| OwnCredential {
                id: credential.id,
                credential_type: credential.credential_type,
                label: credential.user_label,
                created_at: credential.created_at,
            })
            .collect())
    }

    async fn start_own_passkey_registration(
        &self,
        identity: Identity,
        input: StartOwnPasskeyRegistrationInput,
    ) -> Result<StartOwnPasskeyRegistrationOutput, CoreError> {
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let elevated = self
            .claim_elevation(&scope, user_id, input.elevation_id)
            .await?;

        let target = elevated.user_id();

        let webauthn = build_webauthn_client(input.rp_info)?;

        let known = self
            .credentials_of(target)
            .await?
            .into_iter()
            .filter_map(|credential| credential.webauthn_credential_id)
            .collect::<Vec<CredentialID>>();

        let (challenge, registration) = webauthn
            .start_passkey_registration(
                target,
                user.get().email.as_deref().unwrap_or(""),
                &user.get().username,
                (!known.is_empty()).then_some(known),
            )
            .map_err(|e| {
                warn!(user_id = %target, "Failed to start a passkey registration: {e:?}");
                CoreError::InternalServerError
            })?;

        self.passkey_registration_repository
            .start(
                target,
                registration,
                Utc::now() + Duration::minutes(PASSKEY_REGISTRATION_TTL_MINUTES),
            )
            .await?;

        Ok(StartOwnPasskeyRegistrationOutput(challenge))
    }

    async fn confirm_own_passkey_registration(
        &self,
        identity: Identity,
        input: ConfirmOwnPasskeyRegistrationInput,
    ) -> Result<(), CoreError> {
        let (_, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let registration = self
            .passkey_registration_repository
            .consume(user_id, Utc::now())
            .await?
            .ok_or_else(|| {
                warn!(user_id = %user_id, "Refused a passkey confirmation: no live registration to claim");
                CoreError::WebAuthnMissingChallenge
            })?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let passkey = webauthn
            .finish_passkey_registration(&input.credential, &registration)
            .map_err(|e| {
                warn!(user_id = %user_id, "Refused a passkey confirmation: {e:?}");
                CoreError::WebAuthnChallengeFailed
            })?;

        self.credential_repository
            .create_webauthn_credential(user_id, passkey)
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        let _ = self
            .user_required_action_repository
            .remove_required_action(user_id, RequiredAction::ConfigurePasskey)
            .await;

        Ok(())
    }

    async fn delete_own_passkey(
        &self,
        identity: Identity,
        input: DeleteOwnPasskeyInput,
    ) -> Result<(), CoreError> {
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let elevated = self
            .claim_elevation(&scope, user_id, input.elevation_id)
            .await?;

        let target = elevated.user_id();

        let passkey = self
            .credentials_of(target)
            .await?
            .into_iter()
            .find(|credential| credential.id == input.credential_id)
            .ok_or(CoreError::NotFound)?;

        if passkey.credential_type != CredentialType::WebAuthnPublicKeyCredential {
            warn!(
                user_id = %user_id,
                credential_id = %input.credential_id,
                "Refused a passkey removal: the credential is not a passkey"
            );
            return Err(CoreError::NotFound);
        }

        self.refuse_lockout(&scope, &user, FactorRemoval::Passkey)
            .await?;

        self.credential_repository
            .delete_by_id(&user, input.credential_id)
            .await
            .map_err(|_| CoreError::DeleteCredentialError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account_security::ports::{
        MockOtherSessionsRevocationPort, MockPasskeyRegistrationRepository,
    };
    use crate::domain::common::services::tests::{
        create_test_realm_with_name, create_test_user_with_params_and_realm,
    };
    use crate::domain::credential::ports::MockCredentialRepository;
    use crate::domain::password_policy::repository::MockPasswordPolicyRepository;
    use crate::domain::realm::ports::MockRealmRepository;
    use crate::domain::trident::ports::{MockOtpEnrollmentRepository, OtpEnrollment};
    use crate::domain::user::ports::{
        MockUserRepository, MockUserRequiredActionRepository, MockUserRoleRepository,
    };
    use ferriskey_domain::credential::entities::CredentialData;
    use ferriskey_domain::crypto::HashResult;
    use ferriskey_domain::elevation::entities::Elevation;
    use ferriskey_domain::elevation::ports::MockElevationRepository;
    use ferriskey_domain::realm::scope::Unscoped;
    use ferriskey_domain::realm::{Realm, RealmId, RealmSetting};
    use ferriskey_security::crypto::ports::MockHasherRepository;

    type TestService = AccountSecurityServiceImpl<
        MockCredentialRepository,
        MockHasherRepository,
        MockUserRepository,
        MockRealmRepository,
        MockPasswordPolicyRepository,
        MockOtpEnrollmentRepository,
        MockUserRoleRepository,
        MockElevationRepository,
        MockOtherSessionsRevocationPort,
        MockUserRequiredActionRepository,
        MockPasskeyRegistrationRepository,
    >;

    struct Harness {
        credentials: MockCredentialRepository,
        hasher: MockHasherRepository,
        users: MockUserRepository,
        realms: MockRealmRepository,
        policies: MockPasswordPolicyRepository,
        enrollments: MockOtpEnrollmentRepository,
        roles: MockUserRoleRepository,
        elevations: MockElevationRepository,
        sessions: MockOtherSessionsRevocationPort,
        required_actions: MockUserRequiredActionRepository,
        passkey_registrations: MockPasskeyRegistrationRepository,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                credentials: MockCredentialRepository::new(),
                hasher: MockHasherRepository::new(),
                users: MockUserRepository::new(),
                realms: MockRealmRepository::new(),
                policies: MockPasswordPolicyRepository::new(),
                enrollments: MockOtpEnrollmentRepository::new(),
                roles: MockUserRoleRepository::new(),
                elevations: MockElevationRepository::new(),
                sessions: MockOtherSessionsRevocationPort::new(),
                required_actions: MockUserRequiredActionRepository::new(),
                passkey_registrations: MockPasskeyRegistrationRepository::new(),
            }
        }

        fn build(self) -> TestService {
            AccountSecurityServiceImpl::new(
                Arc::new(self.credentials),
                Arc::new(self.hasher),
                Arc::new(self.users),
                Arc::new(self.realms),
                Arc::new(self.policies),
                Arc::new(self.enrollments),
                Arc::new(self.roles),
                Arc::new(self.elevations),
                Arc::new(self.sessions),
                Arc::new(self.required_actions),
                Arc::new(self.passkey_registrations),
            )
        }

        fn resolving(&mut self, realm: &Realm, user: &User) {
            let returned = realm.clone();
            self.realms.expect_get_by_name().returning(move |_| {
                let realm = returned.clone();
                Box::pin(async move { Ok(Some(realm)) })
            });

            let returned = user.clone();
            self.users.expect_get_by_id().returning(move |_| {
                let user = returned.clone();
                Box::pin(async move { Ok(Unscoped::new(user)) })
            });
        }

        fn granting_elevation(&mut self, user: &User, realm_id: RealmId) {
            let user_id = user.id;
            self.elevations.expect_consume().returning(move |_, _| {
                let elevation = Elevation {
                    id: ElevationId::new(Uuid::now_v7()),
                    user_id,
                    realm_id,
                    expires_at: Utc::now() + Duration::minutes(5),
                };
                Box::pin(async move { Ok(Some(Unscoped::new(elevation))) })
            });
        }

        fn refusing_elevation(&mut self) {
            self.elevations
                .expect_consume()
                .returning(|_, _| Box::pin(async move { Ok(None) }));
        }

        fn with_password_credential(&mut self, user_id: Uuid, accepted: bool) {
            self.credentials
                .expect_get_password_credential()
                .returning(move |_| Box::pin(async move { Ok(password_credential(user_id)) }));

            self.hasher
                .expect_verify_password()
                .returning(move |_, _, _, _, _| Box::pin(async move { Ok(accepted) }));
        }

        fn holding_credentials(&mut self, credentials: Vec<Credential>) {
            self.credentials
                .expect_get_credentials_by_user_id()
                .returning(move |_| {
                    let credentials = credentials.clone();
                    Box::pin(async move { Ok(credentials) })
                });
        }

        fn with_realm_settings(&mut self, realm_id: RealmId, require_mfa: bool) {
            self.realms.expect_get_realm_settings().returning(move |_| {
                let mut settings = RealmSetting::new(realm_id, None);
                settings.require_mfa = require_mfa;
                Box::pin(async move { Ok(Some(settings)) })
            });

            self.roles
                .expect_get_user_roles()
                .returning(|_| Box::pin(async move { Ok(Vec::new()) }));
        }
    }

    fn password_credential(user_id: Uuid) -> Credential {
        Credential {
            id: Uuid::now_v7(),
            salt: Some("salt".to_string()),
            credential_type: CredentialType::Password,
            user_id,
            user_label: None,
            secret_data: "hashed".to_string(),
            credential_data: CredentialData::Hash {
                hash_iterations: 1,
                algorithm: "argon2".to_string(),
            },
            temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
        }
    }

    fn credential_of(user_id: Uuid, credential_type: CredentialType) -> Credential {
        Credential {
            credential_type,
            ..password_credential(user_id)
        }
    }

    fn actors() -> (Realm, User, Identity) {
        let realm = create_test_realm_with_name("acme");
        let user =
            create_test_user_with_params_and_realm(&realm, "alice", "alice@acme.test".into(), true);
        let identity = Identity::User(user.clone());

        (realm, user, identity)
    }

    #[tokio::test]
    async fn changing_the_password_without_a_live_elevation_is_refused() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.refusing_elevation();

        let refused = harness
            .build()
            .change_own_password(
                identity,
                ChangeOwnPasswordInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                    current_password: "old".into(),
                    new_password: "NewPassw0rd!x".into(),
                    keep_session_id: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn an_elevation_minted_in_another_realm_cannot_change_the_password() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting_elevation(&user, RealmId::new(Uuid::now_v7()));

        let refused = harness
            .build()
            .change_own_password(
                identity,
                ChangeOwnPasswordInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                    current_password: "old".into(),
                    new_password: "NewPassw0rd!x".into(),
                    keep_session_id: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_wrong_current_password_is_refused_before_anything_is_written() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting_elevation(&user, realm.id);
        harness.with_password_credential(user.id, false);
        harness
            .credentials
            .expect_delete_password_credential()
            .never();
        harness.credentials.expect_create_credential().never();

        let refused = harness
            .build()
            .change_own_password(
                identity,
                ChangeOwnPasswordInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                    current_password: "wrong".into(),
                    new_password: "NewPassw0rd!x".into(),
                    keep_session_id: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::InvalidPassword)));
    }

    #[tokio::test]
    async fn a_successful_password_change_spares_the_caller_own_session() {
        let (realm, user, identity) = actors();
        let kept = Uuid::now_v7();

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting_elevation(&user, realm.id);
        harness.with_password_credential(user.id, true);

        harness
            .policies
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async move { Ok(None) }));
        harness
            .credentials
            .expect_delete_password_credential()
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));
        harness.hasher.expect_hash_password().returning(|_| {
            Box::pin(async move {
                Ok(HashResult::new(
                    "hash".into(),
                    "salt".into(),
                    1,
                    "argon2".into(),
                ))
            })
        });
        let created_for = user.id;
        harness
            .credentials
            .expect_create_credential()
            .times(1)
            .returning(move |_, _, _, _, _| {
                Box::pin(async move { Ok(password_credential(created_for)) })
            });
        harness
            .required_actions
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async move { Ok(()) }));
        harness
            .sessions
            .expect_revoke_all_sessions_except()
            .withf(move |_, keep| *keep == Some(kept))
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        let changed = harness
            .build()
            .change_own_password(
                identity,
                ChangeOwnPasswordInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                    current_password: "old".into(),
                    new_password: "NewPassw0rd!x".into(),
                    keep_session_id: Some(kept),
                },
            )
            .await;

        assert!(
            changed.is_ok(),
            "expected the change to go through: {changed:?}"
        );
    }

    #[tokio::test]
    async fn an_elevation_request_with_a_wrong_password_mints_nothing() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.with_password_credential(user.id, false);
        harness.elevations.expect_start().never();

        let refused = harness
            .build()
            .request_elevation(
                identity,
                RequestElevationInput {
                    realm_name: "acme".into(),
                    proof: ElevationProof::Password("wrong".into()),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::InvalidPassword)));
    }

    #[tokio::test]
    async fn disabling_the_last_second_factor_under_mfa_enforcement_is_refused() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting_elevation(&user, realm.id);
        harness.holding_credentials(vec![
            credential_of(user.id, CredentialType::Password),
            credential_of(user.id, CredentialType::Otp),
        ]);
        harness.with_realm_settings(realm.id, true);
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .disable_own_otp(
                identity,
                DisableOwnOtpInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::MfaFactorRequired)));
    }

    #[tokio::test]
    async fn a_role_lookup_failure_refuses_the_removal_rather_than_assuming_no_roles() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting_elevation(&user, realm.id);
        harness.holding_credentials(vec![
            credential_of(user.id, CredentialType::Password),
            credential_of(user.id, CredentialType::Otp),
        ]);
        harness
            .realms
            .expect_get_realm_settings()
            .returning(move |_| {
                let settings = RealmSetting::new(realm.id, None);
                Box::pin(async move { Ok(Some(settings)) })
            });
        harness
            .roles
            .expect_get_user_roles()
            .returning(|_| Box::pin(async move { Err(CoreError::Database("down".into())) }));
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .disable_own_otp(
                identity,
                DisableOwnOtpInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::Database(_))));
    }

    #[tokio::test]
    async fn deleting_the_only_passkey_of_a_passwordless_account_is_refused() {
        let (realm, user, identity) = actors();
        let passkey = credential_of(user.id, CredentialType::WebAuthnPublicKeyCredential);
        let target = passkey.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting_elevation(&user, realm.id);
        harness.holding_credentials(vec![passkey]);
        harness.with_realm_settings(realm.id, false);
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .delete_own_passkey(
                identity,
                DeleteOwnPasskeyInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                    credential_id: target,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::LastSignInMeans)));
    }

    #[tokio::test]
    async fn a_credential_that_is_not_a_passkey_cannot_be_deleted_through_the_passkey_route() {
        let (realm, user, identity) = actors();
        let otp = credential_of(user.id, CredentialType::Otp);
        let target = otp.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting_elevation(&user, realm.id);
        harness.holding_credentials(vec![credential_of(user.id, CredentialType::Password), otp]);
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .delete_own_passkey(
                identity,
                DeleteOwnPasskeyInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                    credential_id: target,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn confirming_an_otp_without_a_live_enrolment_is_refused() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness
            .enrollments
            .expect_consume_enrollment()
            .returning(|_, _| Box::pin(async move { Ok(None) }));
        harness
            .credentials
            .expect_create_custom_credential()
            .never();

        let refused = harness
            .build()
            .confirm_own_otp_enrollment(
                identity,
                ConfirmOwnOtpEnrollmentInput {
                    realm_name: "acme".into(),
                    code: "123456".into(),
                    label: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::TotpVerificationFailed(_))));
    }

    #[tokio::test]
    async fn an_otp_confirmation_never_reads_the_secret_from_the_caller() {
        let (realm, user, identity) = actors();
        let secret = ferriskey_trident::entities::TotpSecret::from_base32("JBSWY3DPEHPK3PXP");

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);

        let stored = secret.base32_encoded().to_string();
        harness
            .enrollments
            .expect_consume_enrollment()
            .returning(move |user_id, _| {
                let enrollment = OtpEnrollment {
                    id: Uuid::now_v7(),
                    user_id,
                    secret: stored.clone(),
                    expires_at: Utc::now() + Duration::minutes(5),
                    created_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(enrollment)) })
            });
        harness
            .credentials
            .expect_create_custom_credential()
            .never();

        let refused = harness
            .build()
            .confirm_own_otp_enrollment(
                identity,
                ConfirmOwnOtpEnrollmentInput {
                    realm_name: "acme".into(),
                    code: "000000".into(),
                    label: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::TotpVerificationFailed(_))));
    }

    fn rp_info() -> crate::domain::trident::ports::WebAuthnRpInfo {
        crate::domain::trident::ports::WebAuthnRpInfo {
            rp_id: "localhost".into(),
            allowed_origin: "http://localhost".into(),
        }
    }

    #[tokio::test]
    async fn starting_a_passkey_registration_without_a_live_elevation_is_refused() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.refusing_elevation();
        harness.passkey_registrations.expect_start().never();

        let refused = harness
            .build()
            .start_own_passkey_registration(
                identity,
                StartOwnPasskeyRegistrationInput {
                    realm_name: "acme".into(),
                    elevation_id: Uuid::now_v7(),
                    rp_info: rp_info(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn confirming_a_passkey_without_a_live_registration_is_refused() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness
            .passkey_registrations
            .expect_consume()
            .returning(|_, _| Box::pin(async move { Ok(None) }));
        harness
            .credentials
            .expect_create_webauthn_credential()
            .never();

        let refused = harness
            .build()
            .confirm_own_passkey_registration(
                identity,
                ConfirmOwnPasskeyRegistrationInput {
                    realm_name: "acme".into(),
                    rp_info: rp_info(),
                    credential: serde_json::from_value(serde_json::json!({
                        "id": "AAAA",
                        "rawId": "AAAA",
                        "response": {
                            "attestationObject": "AAAA",
                            "clientDataJSON": "AAAA"
                        },
                        "type": "public-key",
                        "extensions": {}
                    }))
                    .expect("a syntactically valid registration payload"),
                    label: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::WebAuthnMissingChallenge)));
    }

    #[tokio::test]
    async fn a_service_account_is_refused_everywhere() {
        let realm = create_test_realm_with_name("acme");
        let mut user =
            create_test_user_with_params_and_realm(&realm, "svc", "svc@acme.test".into(), true);
        user.client_id = Some(Uuid::now_v7());
        let identity = Identity::User(user);

        let refused = Harness::new()
            .build()
            .list_own_credentials(
                identity,
                ListOwnCredentialsInput {
                    realm_name: "acme".into(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::Forbidden(_))));
    }
}
