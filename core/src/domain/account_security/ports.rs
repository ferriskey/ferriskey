use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::scope::{RealmScope, Scoped};
use ferriskey_domain::user::entities::User;
use uuid::Uuid;

use chrono::{DateTime, Utc};
use webauthn_rs::prelude::PasskeyRegistration;

use crate::domain::account_security::entities::{
    ChangeOwnPasswordInput, ConfirmOwnOtpEnrollmentInput, ConfirmOwnPasskeyRegistrationInput,
    DeleteOwnPasskeyInput, DisableOwnOtpInput, ListOwnCredentialsInput, OwnCredential,
    RequestElevationInput, RequestElevationOutput, StartOwnOtpEnrollmentInput,
    StartOwnOtpEnrollmentOutput, StartOwnPasskeyRegistrationInput,
    StartOwnPasskeyRegistrationOutput,
};

pub trait AccountSecurityService: Send + Sync {
    fn request_elevation(
        &self,
        identity: Identity,
        input: RequestElevationInput,
    ) -> impl Future<Output = Result<RequestElevationOutput, CoreError>> + Send;

    fn change_own_password(
        &self,
        identity: Identity,
        input: ChangeOwnPasswordInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn start_own_otp_enrollment(
        &self,
        identity: Identity,
        input: StartOwnOtpEnrollmentInput,
    ) -> impl Future<Output = Result<StartOwnOtpEnrollmentOutput, CoreError>> + Send;

    fn confirm_own_otp_enrollment(
        &self,
        identity: Identity,
        input: ConfirmOwnOtpEnrollmentInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn disable_own_otp(
        &self,
        identity: Identity,
        input: DisableOwnOtpInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_own_credentials(
        &self,
        identity: Identity,
        input: ListOwnCredentialsInput,
    ) -> impl Future<Output = Result<Vec<OwnCredential>, CoreError>> + Send;

    fn start_own_passkey_registration(
        &self,
        identity: Identity,
        input: StartOwnPasskeyRegistrationInput,
    ) -> impl Future<Output = Result<StartOwnPasskeyRegistrationOutput, CoreError>> + Send;

    fn confirm_own_passkey_registration(
        &self,
        identity: Identity,
        input: ConfirmOwnPasskeyRegistrationInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete_own_passkey(
        &self,
        identity: Identity,
        input: DeleteOwnPasskeyInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait PasskeyRegistrationRepository: Send + Sync {
    fn start(
        &self,
        user_id: Uuid,
        registration: PasskeyRegistration,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn consume(
        &self,
        user_id: Uuid,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<PasskeyRegistration>, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait OtherSessionsRevocationPort: Send + Sync {
    fn revoke_all_sessions_except(
        &self,
        scope: &RealmScope,
        user: &Scoped<User>,
        keep_session_id: Option<Uuid>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
