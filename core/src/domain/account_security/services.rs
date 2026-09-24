use std::sync::Arc;

use chrono::{Duration, Utc};
use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::credential::entities::{Credential, CredentialData, CredentialType};
use ferriskey_domain::credential::ports::CredentialRepository;
use ferriskey_domain::elevation::entities::{Elevated, ElevationId, ElevationProofKind};
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
use ferriskey_security::crypto::password_check::{has_local_password, verify_user_password};
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

        if !user.get().enabled {
            return Err(CoreError::UserDisabled);
        }

        if user
            .get()
            .locked_until
            .is_some_and(|until| until > Utc::now())
        {
            warn!(
                user_id = %user.get().id,
                "Refused an account security operation: the account is locked"
            );
            return Err(CoreError::AccountLocked);
        }

        Ok((scope, user))
    }

    async fn claim_elevation(
        &self,
        scope: &RealmScope,
        caller: Uuid,
        session_id: Uuid,
        elevation_id: Uuid,
    ) -> Result<Elevated, CoreError> {
        ferriskey_domain::elevation::claim(
            self.elevation_repository.as_ref(),
            scope,
            ferriskey_domain::elevation::Claim {
                caller,
                session_id,
                elevation_id: ElevationId::new(elevation_id),
                now: Utc::now(),
            },
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
            has_password: credentials.iter().any(|c| {
                c.credential_type == CredentialType::Password
                    && matches!(c.credential_data, CredentialData::Hash { .. })
            }),
            passkey_count: credentials
                .iter()
                .filter(|c| c.credential_type == CredentialType::WebAuthnPublicKeyCredential)
                .count(),
            has_otp: credentials
                .iter()
                .any(|c| c.credential_type == CredentialType::Otp),
            federated_identity_count: credentials
                .iter()
                .filter(|c| matches!(c.credential_data, CredentialData::Federated { .. }))
                .count(),
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

        let (accepted, kind) = match &input.proof {
            ElevationProof::Password(password) => {
                if !has_local_password(self.credential_repository.as_ref(), user_id).await? {
                    warn!(
                        user_id = %user_id,
                        "Refused an elevation request: the account has no local password"
                    );
                    return Err(CoreError::NoLocalPassword);
                }

                (
                    verify_user_password(
                        self.credential_repository.as_ref(),
                        self.hasher_repository.as_ref(),
                        user_id,
                        password,
                    )
                    .await?,
                    ElevationProofKind::Password,
                )
            }
            ElevationProof::Otp(code) => (
                match self.otp_secret_of(user_id).await? {
                    Some(secret) => verify(&secret, code)?,
                    None => false,
                },
                ElevationProofKind::Otp,
            ),
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
                input.session_id,
                kind,
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
            .claim_elevation(&scope, user_id, input.session_id, input.elevation_id)
            .await?;

        let target = elevated.user_id();

        if !has_local_password(self.credential_repository.as_ref(), target).await? {
            warn!(
                user_id = %target,
                "Refused a password change: the account has no local password"
            );
            return Err(CoreError::NoLocalPassword);
        }

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

        let hash_result = self
            .hasher_repository
            .hash_password(&input.new_password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .delete_password_credential(target)
            .await
            .map_err(|_| CoreError::DeleteCredentialError)?;

        self.credential_repository
            .create_credential(target, "password".into(), hash_result, "".into(), false)
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        self.other_sessions_revocation
            .revoke_all_sessions_except(&user, Some(elevated.session_id()))
            .await?;

        if let Err(e) = self.elevation_repository.clear_for_user(target).await {
            warn!(user_id = %target, "Failed to drop outstanding elevations after a password change: {e:?}");
        }

        if let Err(e) = self
            .user_required_action_repository
            .remove_required_action(target, RequiredAction::UpdatePassword)
            .await
        {
            warn!(user_id = %target, "Failed to remove the UpdatePassword required action: {e:?}");
        }

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
            .claim_elevation(&scope, user_id, input.session_id, input.elevation_id)
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
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let elevated = self
            .claim_elevation(&scope, user_id, input.session_id, input.elevation_id)
            .await?;

        let target = elevated.primary()?.user_id();

        let enrollment = self
            .otp_enrollment_repository
            .consume_enrollment(target, Utc::now())
            .await?
            .ok_or_else(|| {
                warn!(user_id = %target, "Refused an OTP confirmation: no live enrolment to claim");
                CoreError::TotpVerificationFailed("no pending OTP enrollment for this user".into())
            })?;

        let secret = TotpSecret::from_base32(&enrollment.secret);

        if !verify(&secret, &input.code)? {
            warn!(user_id = %target, "Refused an OTP confirmation: invalid code");
            return Err(CoreError::TotpVerificationFailed(
                "failed to verify OTP".into(),
            ));
        }

        let superseded = self
            .credentials_of(target)
            .await?
            .into_iter()
            .filter(|credential| credential.credential_type == CredentialType::Otp)
            .map(|credential| credential.id)
            .collect::<Vec<Uuid>>();

        let credential_data = serde_json::json!({
            "subType": "totp",
            "digits": 6,
            "counter": 0,
            "period": 30,
            "algorithm": "HmacSha256",
        });

        self.credential_repository
            .create_custom_credential(
                target,
                CredentialType::Otp.to_string(),
                enrollment.secret,
                input.label,
                credential_data,
            )
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        for credential_id in superseded {
            if let Err(e) = self
                .credential_repository
                .delete_by_id(&user, credential_id)
                .await
            {
                warn!(user_id = %target, "Failed to drop a superseded OTP credential: {e:?}");
            }
        }

        if let Err(e) = self
            .user_required_action_repository
            .remove_required_action(target, RequiredAction::ConfigureOtp)
            .await
        {
            warn!(user_id = %target, "Failed to remove the ConfigureOtp required action: {e:?}");
        }

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
            .claim_elevation(&scope, user_id, input.session_id, input.elevation_id)
            .await?;

        let target = elevated.primary()?.user_id();

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
            .claim_elevation(&scope, user_id, input.session_id, input.elevation_id)
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
        let (scope, user) = self.caller(&identity, &input.realm_name).await?;
        let user_id = user.get().id;

        let elevated = self
            .claim_elevation(&scope, user_id, input.session_id, input.elevation_id)
            .await?;

        let target = elevated.user_id();

        let registration = self
            .passkey_registration_repository
            .consume(target, Utc::now())
            .await?
            .ok_or_else(|| {
                warn!(user_id = %target, "Refused a passkey confirmation: no live registration to claim");
                CoreError::WebAuthnMissingChallenge
            })?;

        let webauthn = build_webauthn_client(input.rp_info)?;

        let passkey = webauthn
            .finish_passkey_registration(&input.credential, &registration)
            .map_err(|e| {
                warn!(user_id = %target, "Refused a passkey confirmation: {e:?}");
                CoreError::WebAuthnChallengeFailed
            })?;

        self.credential_repository
            .create_webauthn_credential(target, passkey)
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        if let Err(e) = self
            .user_required_action_repository
            .remove_required_action(target, RequiredAction::ConfigurePasskey)
            .await
        {
            warn!(user_id = %target, "Failed to remove the ConfigurePasskey required action: {e:?}");
        }

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
            .claim_elevation(&scope, user_id, input.session_id, input.elevation_id)
            .await?;

        let target = elevated.primary()?.user_id();

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
    use crate::domain::trident::ports::{
        MockOtpEnrollmentRepository, OtpEnrollment, WebAuthnRpInfo,
    };
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

    const SECRET: &str = "JBSWY3DPEHPK3PXP";

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

            let wanted = user.id;
            let returned = user.clone();
            self.users
                .expect_get_by_id()
                .withf(move |id| *id == wanted)
                .returning(move |_| {
                    let user = returned.clone();
                    Box::pin(async move { Ok(Unscoped::new(user)) })
                });
        }

        fn granting(
            &mut self,
            user: &User,
            realm_id: RealmId,
            session: Uuid,
            proof: ElevationProofKind,
        ) {
            let user_id = user.id;
            self.elevations
                .expect_find_live()
                .withf(move |_, caller, _| *caller == user_id)
                .returning(move |_, _, _| {
                    let elevation = Elevation {
                        id: ElevationId::new(Uuid::now_v7()),
                        user_id,
                        realm_id,
                        session_id: session,
                        proof,
                        expires_at: Utc::now() + Duration::minutes(5),
                    };
                    Box::pin(async move { Ok(Some(Unscoped::new(elevation))) })
                });
        }

        fn refusing_elevation(&mut self) {
            self.elevations
                .expect_find_live()
                .returning(|_, _, _| Box::pin(async move { Ok(None) }));
        }

        fn with_password(&mut self, user_id: Uuid, accepted: bool) {
            self.credentials
                .expect_get_password_credential()
                .withf(move |id| *id == user_id)
                .returning(move |_| Box::pin(async move { Ok(password_credential(user_id)) }));

            self.hasher
                .expect_verify_password()
                .returning(move |_, _, _, _, _| Box::pin(async move { Ok(accepted) }));

            self.hasher.expect_needs_rehash().returning(|_| false);
        }

        fn holding(&mut self, user_id: Uuid, credentials: Vec<Credential>) {
            self.credentials
                .expect_get_credentials_by_user_id()
                .withf(move |id| *id == user_id)
                .returning(move |_| {
                    let credentials = credentials.clone();
                    Box::pin(async move { Ok(credentials) })
                });
        }

        fn with_settings(&mut self, realm_id: RealmId, require_mfa: bool) {
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

    fn otp_credential(user_id: Uuid) -> Credential {
        Credential {
            secret_data: SECRET.to_string(),
            ..credential_of(user_id, CredentialType::Otp)
        }
    }

    fn actors() -> (Realm, User, Identity) {
        let realm = create_test_realm_with_name("acme");
        let user =
            create_test_user_with_params_and_realm(&realm, "alice", "alice@acme.test".into(), true);
        let identity = Identity::User(user.clone());

        (realm, user, identity)
    }

    fn rp_info() -> WebAuthnRpInfo {
        WebAuthnRpInfo {
            rp_id: "localhost".into(),
            allowed_origin: "http://localhost".into(),
        }
    }

    fn change_password(session: Uuid, current: &str) -> ChangeOwnPasswordInput {
        ChangeOwnPasswordInput {
            realm_name: "acme".into(),
            session_id: session,
            elevation_id: Uuid::now_v7(),
            current_password: current.into(),
            new_password: "NewPassw0rd!x".into(),
        }
    }

    #[tokio::test]
    async fn changing_the_password_without_a_live_elevation_is_refused() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.refusing_elevation();
        harness.credentials.expect_create_credential().never();

        let refused = harness
            .build()
            .change_own_password(identity, change_password(session, "old"))
            .await;

        assert!(matches!(refused, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn an_elevation_minted_in_another_realm_cannot_change_the_password() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(
            &user,
            RealmId::new(Uuid::now_v7()),
            session,
            ElevationProofKind::Password,
        );

        let refused = harness
            .build()
            .change_own_password(identity, change_password(session, "old"))
            .await;

        assert!(matches!(refused, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn an_elevation_minted_for_another_session_cannot_change_the_password() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(
            &user,
            realm.id,
            Uuid::now_v7(),
            ElevationProofKind::Password,
        );
        harness.credentials.expect_create_credential().never();

        let refused = harness
            .build()
            .change_own_password(identity, change_password(Uuid::now_v7(), "old"))
            .await;

        assert!(matches!(refused, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn an_otp_proof_cannot_disable_the_otp_it_proved() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Otp);
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .disable_own_otp(
                identity,
                DisableOwnOtpInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::PrimaryProofRequired)));
    }

    #[tokio::test]
    async fn an_otp_proof_cannot_delete_a_passkey() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Otp);
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .delete_own_passkey(
                identity,
                DeleteOwnPasskeyInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    credential_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::PrimaryProofRequired)));
    }

    #[tokio::test]
    async fn an_otp_proof_cannot_replace_the_authenticator() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Otp);
        harness.enrollments.expect_consume_enrollment().never();
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
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    code: "123456".into(),
                    label: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::PrimaryProofRequired)));
    }

    #[tokio::test]
    async fn confirming_a_passkey_without_a_live_elevation_is_refused() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.refusing_elevation();
        harness.passkey_registrations.expect_consume().never();
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
                    session_id: Uuid::now_v7(),
                    elevation_id: Uuid::now_v7(),
                    rp_info: rp_info(),
                    credential: registration_payload(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::ElevationRequired)));
    }

    fn registration_payload() -> webauthn_rs::prelude::RegisterPublicKeyCredential {
        serde_json::from_value(serde_json::json!({
            "id": "AAAA",
            "rawId": "AAAA",
            "response": { "attestationObject": "AAAA", "clientDataJSON": "AAAA" },
            "type": "public-key",
            "extensions": {}
        }))
        .expect("a syntactically valid registration payload")
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

    #[tokio::test]
    async fn a_locked_account_cannot_reach_the_account_security_surface() {
        let (realm, mut user, _) = actors();
        user.locked_until = Some(Utc::now() + Duration::minutes(10));
        let identity = Identity::User(user.clone());

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.credentials.expect_get_password_credential().never();

        let refused = harness
            .build()
            .request_elevation(
                identity,
                RequestElevationInput {
                    realm_name: "acme".into(),
                    session_id: Uuid::now_v7(),
                    proof: ElevationProof::Password("hunter2".into()),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::AccountLocked)));
    }

    #[tokio::test]
    async fn a_disabled_account_cannot_reach_the_account_security_surface() {
        let realm = create_test_realm_with_name("acme");
        let user = create_test_user_with_params_and_realm(
            &realm,
            "alice",
            "alice@acme.test".into(),
            false,
        );
        let identity = Identity::User(user.clone());

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);

        let refused = harness
            .build()
            .list_own_credentials(
                identity,
                ListOwnCredentialsInput {
                    realm_name: "acme".into(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::UserDisabled)));
    }

    #[tokio::test]
    async fn an_elevation_request_with_a_wrong_password_mints_nothing() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.with_password(user.id, false);
        harness.elevations.expect_start().never();

        let refused = harness
            .build()
            .request_elevation(
                identity,
                RequestElevationInput {
                    realm_name: "acme".into(),
                    session_id: Uuid::now_v7(),
                    proof: ElevationProof::Password("wrong".into()),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::InvalidPassword)));
    }

    #[tokio::test]
    async fn an_accepted_password_mints_a_primary_proof_for_the_caller_and_session() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let expected_user = user.id;
        let expected_realm = realm.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.with_password(user.id, true);
        harness
            .elevations
            .expect_start()
            .withf(move |user_id, realm_id, session_id, proof, expires_at| {
                *user_id == expected_user
                    && *realm_id == expected_realm
                    && *session_id == session
                    && *proof == ElevationProofKind::Password
                    && *expires_at > Utc::now()
            })
            .times(1)
            .returning(move |user_id, realm_id, session_id, proof, expires_at| {
                let elevation = Elevation {
                    id: ElevationId::new(Uuid::now_v7()),
                    user_id,
                    realm_id,
                    session_id,
                    proof,
                    expires_at,
                };
                Box::pin(async move { Ok(elevation) })
            });

        let minted = harness
            .build()
            .request_elevation(
                identity,
                RequestElevationInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    proof: ElevationProof::Password("hunter2".into()),
                },
            )
            .await
            .expect("an accepted password must mint an elevation");

        assert!(minted.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn an_otp_proof_is_refused_when_the_account_has_no_authenticator() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.holding(
            user.id,
            vec![credential_of(user.id, CredentialType::Password)],
        );
        harness.elevations.expect_start().never();

        let refused = harness
            .build()
            .request_elevation(
                identity,
                RequestElevationInput {
                    realm_name: "acme".into(),
                    session_id: Uuid::now_v7(),
                    proof: ElevationProof::Otp("123456".into()),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::InvalidPassword)));
    }

    #[tokio::test]
    async fn a_bcrypt_credential_carries_no_separate_salt_and_is_still_usable() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let expected = user.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness
            .credentials
            .expect_get_password_credential()
            .returning(move |id| {
                let legacy = Credential {
                    salt: None,
                    secret_data: "$2a$10$abcdefghijklmnopqrstuv".into(),
                    credential_data: CredentialData::Hash {
                        hash_iterations: 10,
                        algorithm: "bcrypt".into(),
                    },
                    ..password_credential(id)
                };
                Box::pin(async move { Ok(legacy) })
            });
        harness
            .hasher
            .expect_verify_password()
            .returning(|_, _, _, _, _| Box::pin(async move { Ok(true) }));
        harness
            .hasher
            .expect_needs_rehash()
            .returning(|algorithm| algorithm == "bcrypt");
        harness.hasher.expect_hash_password().returning(|_| {
            Box::pin(async move {
                Ok(HashResult::new(
                    "argon".into(),
                    "salt".into(),
                    1,
                    "argon2id".into(),
                ))
            })
        });
        harness
            .credentials
            .expect_update_password_credential()
            .times(1)
            .returning(|_, _, _| Box::pin(async move { Ok(()) }));
        harness
            .elevations
            .expect_start()
            .withf(move |user_id, _, _, _, _| *user_id == expected)
            .times(1)
            .returning(move |user_id, realm_id, session_id, proof, expires_at| {
                let elevation = Elevation {
                    id: ElevationId::new(Uuid::now_v7()),
                    user_id,
                    realm_id,
                    session_id,
                    proof,
                    expires_at,
                };
                Box::pin(async move { Ok(elevation) })
            });

        let minted = harness
            .build()
            .request_elevation(
                identity,
                RequestElevationInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    proof: ElevationProof::Password("hunter2".into()),
                },
            )
            .await;

        assert!(
            minted.is_ok(),
            "a bcrypt credential must still elevate: {minted:?}"
        );
    }

    #[tokio::test]
    async fn a_federated_account_is_told_it_has_no_local_password_rather_than_given_a_500() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness
            .credentials
            .expect_get_password_credential()
            .returning(move |id| {
                let marker = Credential {
                    salt: None,
                    credential_data: CredentialData::Federated {
                        provider_id: "ldap".into(),
                        provider_type: "ldap".into(),
                    },
                    ..password_credential(id)
                };
                Box::pin(async move { Ok(marker) })
            });
        harness.elevations.expect_start().never();

        let refused = harness
            .build()
            .request_elevation(
                identity,
                RequestElevationInput {
                    realm_name: "acme".into(),
                    session_id: Uuid::now_v7(),
                    proof: ElevationProof::Password("directory-secret".into()),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::NoLocalPassword)));
    }

    #[tokio::test]
    async fn a_wrong_current_password_is_refused_before_anything_is_written() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.with_password(user.id, false);
        harness
            .credentials
            .expect_delete_password_credential()
            .never();
        harness.credentials.expect_create_credential().never();
        harness.sessions.expect_revoke_all_sessions_except().never();

        let refused = harness
            .build()
            .change_own_password(identity, change_password(session, "wrong"))
            .await;

        assert!(matches!(refused, Err(CoreError::InvalidPassword)));
    }

    #[tokio::test]
    async fn a_successful_password_change_writes_for_the_caller_and_spares_their_session() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let expected = user.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.with_password(user.id, true);

        harness
            .policies
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async move { Ok(None) }));
        harness
            .credentials
            .expect_delete_password_credential()
            .withf(move |id| *id == expected)
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
        harness
            .credentials
            .expect_create_credential()
            .withf(move |id, kind, _, _, temporary| {
                *id == expected && kind == "password" && !*temporary
            })
            .times(1)
            .returning(move |_, _, _, _, _| {
                Box::pin(async move { Ok(password_credential(expected)) })
            });
        harness
            .sessions
            .expect_revoke_all_sessions_except()
            .withf(move |user, keep| user.get().id == expected && *keep == Some(session))
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(()) }));
        harness
            .elevations
            .expect_clear_for_user()
            .withf(move |id| *id == expected)
            .times(1)
            .returning(|_| Box::pin(async move { Ok(1) }));
        harness
            .required_actions
            .expect_remove_required_action()
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        let changed = harness
            .build()
            .change_own_password(identity, change_password(session, "old"))
            .await;

        assert!(
            changed.is_ok(),
            "expected the change to go through: {changed:?}"
        );
    }

    #[tokio::test]
    async fn a_hashing_failure_leaves_the_current_password_in_place() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.with_password(user.id, true);
        harness
            .policies
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async move { Ok(None) }));
        harness.hasher.expect_hash_password().returning(|_| {
            Box::pin(async move {
                Err(ferriskey_security::SecurityError::HashingError(
                    "boom".into(),
                ))
            })
        });
        harness
            .credentials
            .expect_delete_password_credential()
            .never();
        harness.credentials.expect_create_credential().never();

        let refused = harness
            .build()
            .change_own_password(identity, change_password(session, "old"))
            .await;

        assert!(matches!(refused, Err(CoreError::HashPasswordError(_))));
    }

    #[tokio::test]
    async fn a_new_password_that_violates_the_realm_policy_is_refused() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.with_password(user.id, true);
        harness
            .policies
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async move { Ok(None) }));
        harness
            .credentials
            .expect_delete_password_credential()
            .never();
        harness.credentials.expect_create_credential().never();

        let mut input = change_password(session, "old");
        input.new_password = "a".into();

        let refused = harness.build().change_own_password(identity, input).await;

        assert!(matches!(
            refused,
            Err(CoreError::PasswordPolicyViolation(_))
        ));
    }

    #[tokio::test]
    async fn confirming_an_otp_without_a_live_enrolment_is_refused() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
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
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    code: "123456".into(),
                    label: None,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::TotpVerificationFailed(_))));
    }

    #[tokio::test]
    async fn a_code_that_does_not_match_the_enrolment_is_refused_and_writes_nothing() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness
            .enrollments
            .expect_consume_enrollment()
            .returning(move |user_id, _| {
                let enrollment = OtpEnrollment {
                    id: Uuid::now_v7(),
                    user_id,
                    secret: SECRET.to_string(),
                    expires_at: Utc::now() + Duration::minutes(5),
                    created_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(enrollment)) })
            });
        harness
            .credentials
            .expect_create_custom_credential()
            .never();
        harness.credentials.expect_delete_by_id().never();

        let live = verify(&TotpSecret::from_base32(SECRET), "000000").unwrap_or(false);

        let refused = harness
            .build()
            .confirm_own_otp_enrollment(
                identity,
                ConfirmOwnOtpEnrollmentInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    code: "000000".into(),
                    label: None,
                },
            )
            .await;

        if live {
            assert!(refused.is_ok(), "000000 happened to be the live code");
        } else {
            assert!(matches!(refused, Err(CoreError::TotpVerificationFailed(_))));
        }
    }

    #[tokio::test]
    async fn disabling_the_last_second_factor_under_mfa_enforcement_is_refused() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(
            user.id,
            vec![
                credential_of(user.id, CredentialType::Password),
                otp_credential(user.id),
            ],
        );
        harness.with_settings(realm.id, true);
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .disable_own_otp(
                identity,
                DisableOwnOtpInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::MfaFactorRequired)));
    }

    #[tokio::test]
    async fn a_role_lookup_failure_refuses_the_removal_rather_than_assuming_no_roles() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(
            user.id,
            vec![
                credential_of(user.id, CredentialType::Password),
                otp_credential(user.id),
            ],
        );
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
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::Database(_))));
    }

    #[tokio::test]
    async fn disabling_otp_drops_the_caller_own_authenticator() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let otp = otp_credential(user.id);
        let doomed = otp.id;
        let owner = user.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(
            user.id,
            vec![credential_of(user.id, CredentialType::Password), otp],
        );
        harness.with_settings(realm.id, false);
        harness
            .credentials
            .expect_delete_by_id()
            .withf(move |user, id| user.get().id == owner && *id == doomed)
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(()) }));
        harness
            .enrollments
            .expect_clear_enrollments()
            .returning(|_| Box::pin(async move { Ok(0) }));

        let disabled = harness
            .build()
            .disable_own_otp(
                identity,
                DisableOwnOtpInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(
            disabled.is_ok(),
            "expected the removal to go through: {disabled:?}"
        );
    }

    #[tokio::test]
    async fn deleting_the_only_passkey_of_a_passwordless_account_is_refused() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let passkey = credential_of(user.id, CredentialType::WebAuthnPublicKeyCredential);
        let target = passkey.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(user.id, vec![passkey]);
        harness.with_settings(realm.id, false);
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .delete_own_passkey(
                identity,
                DeleteOwnPasskeyInput {
                    realm_name: "acme".into(),
                    session_id: session,
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
        let session = Uuid::now_v7();
        let otp = otp_credential(user.id);
        let target = otp.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(
            user.id,
            vec![credential_of(user.id, CredentialType::Password), otp],
        );
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .delete_own_passkey(
                identity,
                DeleteOwnPasskeyInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    credential_id: target,
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_passkey_of_another_account_is_not_deletable_through_this_route() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(
            user.id,
            vec![credential_of(user.id, CredentialType::Password)],
        );
        harness.credentials.expect_delete_by_id().never();

        let refused = harness
            .build()
            .delete_own_passkey(
                identity,
                DeleteOwnPasskeyInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    credential_id: Uuid::now_v7(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn deleting_a_passkey_is_allowed_while_a_password_remains() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let passkey = credential_of(user.id, CredentialType::WebAuthnPublicKeyCredential);
        let doomed = passkey.id;
        let owner = user.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(
            user.id,
            vec![credential_of(user.id, CredentialType::Password), passkey],
        );
        harness.with_settings(realm.id, false);
        harness
            .credentials
            .expect_delete_by_id()
            .withf(move |user, id| user.get().id == owner && *id == doomed)
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        let deleted = harness
            .build()
            .delete_own_passkey(
                identity,
                DeleteOwnPasskeyInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    credential_id: doomed,
                },
            )
            .await;

        assert!(
            deleted.is_ok(),
            "expected the removal to go through: {deleted:?}"
        );
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
                    session_id: Uuid::now_v7(),
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
        let session = Uuid::now_v7();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
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
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    rp_info: rp_info(),
                    credential: registration_payload(),
                },
            )
            .await;

        assert!(matches!(refused, Err(CoreError::WebAuthnMissingChallenge)));
    }

    #[tokio::test]
    async fn starting_a_passkey_registration_records_a_challenge_for_the_caller() {
        let (realm, user, identity) = actors();
        let session = Uuid::now_v7();
        let owner = user.id;

        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.granting(&user, realm.id, session, ElevationProofKind::Password);
        harness.holding(
            user.id,
            vec![credential_of(user.id, CredentialType::Password)],
        );
        harness
            .passkey_registrations
            .expect_start()
            .withf(move |user_id, _, expires_at| *user_id == owner && *expires_at > Utc::now())
            .times(1)
            .returning(|_, _, _| Box::pin(async move { Ok(()) }));

        let started = harness
            .build()
            .start_own_passkey_registration(
                identity,
                StartOwnPasskeyRegistrationInput {
                    realm_name: "acme".into(),
                    session_id: session,
                    elevation_id: Uuid::now_v7(),
                    rp_info: rp_info(),
                },
            )
            .await;

        assert!(started.is_ok(), "expected a challenge: {:?}", started.err());
    }

    #[tokio::test]
    async fn listing_credentials_never_hands_back_secret_material() {
        let (realm, user, identity) = actors();
        let mut harness = Harness::new();
        harness.resolving(&realm, &user);
        harness.holding(
            user.id,
            vec![
                credential_of(user.id, CredentialType::Password),
                otp_credential(user.id),
            ],
        );

        let listed = harness
            .build()
            .list_own_credentials(
                identity,
                ListOwnCredentialsInput {
                    realm_name: "acme".into(),
                },
            )
            .await
            .expect("the caller must be able to list their own credentials");

        assert_eq!(listed.len(), 2);
        let rendered = format!("{listed:?}");
        assert!(
            !rendered.contains(SECRET) && !rendered.contains("hashed"),
            "secret material leaked into the listing: {rendered}"
        );
    }
}
