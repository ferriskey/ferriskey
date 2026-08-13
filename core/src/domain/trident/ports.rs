use chrono::{DateTime, Utc};
use ferriskey_trident::entities::{MagicLink, PasswordResetToken};
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    authentication::entities::WebAuthnChallenge,
    common::entities::app_errors::CoreError,
    credential::entities::CredentialOverview,
    crypto::HashResult,
    trident::entities::{MfaRecoveryCode, TotpSecret},
    user::entities::RequiredAction,
};

pub use webauthn_rs::prelude::{
    CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

pub trait TotpService: Send + Sync + Clone + 'static {
    fn generate_secret(&self) -> Result<TotpSecret, CoreError>;
    fn generate_otpauth_uri(&self, issuer: &str, user_email: &str, secret: &TotpSecret) -> String;
    fn verify(&self, secret: &TotpSecret, code: &str) -> Result<bool, CoreError>;
}

/// Required relying party information for the good use of Webauthn
pub struct WebAuthnRpInfo {
    /// https://www.w3.org/TR/2021/REC-webauthn-2-20210408/#relying-party-identifier
    /// tldr; a hostname which determines the scope of origin for the public key.
    /// e.g: if 'my-app.com' then only origins under 'my-app.com' ('api.my-app.com', 'client.my-app.com', etc.) will be allowed.
    ///
    /// For localhost apps set this to 'localhost'
    pub rp_id: String,

    /// Required for internal validation when receiving a payload from a client.
    /// The server decides which origin is allowed for this specific context. If the client's
    /// payload doesn't match, then no further verification is done and the payload is rejected.
    /// Must be a valid origin format string ! (scheme://host[:port])
    pub allowed_origin: String,
}

/// A persisted WebAuthn challenge for the self-service (Bearer) passkey
/// registration flow.
///
/// The login flow keeps its challenge on the `auth_sessions` row, but the
/// self-service flow has no auth session (it is keyed by the authenticated
/// user id instead). Storing it in a repository — rather than a process-local
/// map — keeps it consistent across instances and lets it expire via
/// `expires_at`.
#[derive(Debug, Clone)]
pub struct WebAuthnChallengeRecord {
    pub user_id: Uuid,
    pub challenge: WebAuthnChallenge,
    pub expires_at: DateTime<Utc>,
}

/// Persistence for self-service WebAuthn registration challenges.
#[cfg_attr(test, mockall::automock)]
pub trait WebAuthnChallengeRepository: Send + Sync {
    /// Store (or replace) the pending registration challenge for a user.
    fn save(
        &self,
        record: WebAuthnChallengeRecord,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Take the pending registration challenge for a user, removing it so it
    /// cannot be reused. Returns `None` when no (unexpired) challenge exists.
    fn take(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<WebAuthnChallenge>, CoreError>> + Send;

    /// Drop any expired challenges.
    fn cleanup_expired(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

pub struct WebAuthnPublicKeyCreateOptionsInput {
    pub session_code: String,
    pub rp_info: WebAuthnRpInfo,
}
/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialrpentity
pub struct WebAuthnPublicKeyCreateOptionsOutput(pub CreationChallengeResponse);

pub struct WebAuthnValidatePublicKeyInput {
    pub rp_info: WebAuthnRpInfo,
    pub session_code: String,
    pub credential: RegisterPublicKeyCredential,
}
pub struct WebAuthnValidatePublicKeyOutput {}

pub struct WebAuthnPublicKeyRequestOptionsInput {
    pub session_code: String,
    pub rp_info: WebAuthnRpInfo,
}
pub struct WebAuthnPublicKeyRequestOptionsOutput(pub RequestChallengeResponse);

pub struct WebAuthnPublicKeyAuthenticateInput {
    pub session_code: String,
    pub rp_info: WebAuthnRpInfo,
    pub credential: PublicKeyCredential,
}
pub struct WebAuthnPublicKeyAuthenticateOutput {
    pub login_url: String,
}

pub struct PasskeyRequestOptionsInput {
    pub realm_name: String,
    pub session_code: String,
    pub username: Option<String>,
    pub rp_info: WebAuthnRpInfo,
}

pub struct PasskeyAuthenticateInput {
    pub realm_name: String,
    pub session_code: String,
    pub rp_info: WebAuthnRpInfo,
    pub credential: PublicKeyCredential,
}

pub struct PasskeyAuthenticateOutput {
    pub login_url: String,
}

pub struct ChallengeOtpInput {
    pub session_code: String,
    pub code: String,
}

pub struct ChallengeOtpOutput {
    pub login_url: Option<String>,
    pub required_actions: Vec<RequiredAction>,
    pub temporary_token: Option<String>,
}

pub struct SetupOtpInput {
    pub issuer: String,
}

pub struct SetupOtpOutput {
    pub secret: String,
    pub otpauth_uri: String,
}

pub struct UpdatePasswordInput {
    pub realm_name: String,
    pub value: String,
}

/// Deliberately carries no secret. The secret is read back from the pending
/// `OtpEnrollment` the server itself issued; taking it from the request body made the
/// verification a tautology and let a caller enrol a secret of their choosing (FK-003).
pub struct VerifyOtpInput {
    pub code: String,
    pub label: Option<String>,
}

pub struct VerifyOtpOutput {
    pub message: String,
    pub user_id: Uuid,
}

pub struct GenerateRecoveryCodeInput {
    pub amount: u8,
    pub format: String,
}

pub struct GenerateRecoveryCodeOutput {
    pub codes: Vec<String>,
}

pub struct BurnRecoveryCodeInput {
    pub session_code: String,
    pub format: String,
    pub code: String,
}

pub struct BurnRecoveryCodeOutput {
    pub login_url: String,
}

pub struct MagicLinkInput {
    pub realm_name: String,
    pub email: String,
    pub base_url: String,
    /// Session code from the FERRISKEY_SESSION cookie at send time,
    /// stored so verify can use the correct AuthSession without an OAuth redirect.
    pub session_code: Option<String>,
}

pub struct VerifyMagicLinkInput {
    pub magic_token_id: Uuid,
    pub magic_token: String,
}

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

pub struct VerifyResetTokenInput {
    pub token_id: Uuid,
}

/// A TOTP secret proposed by `setup_otp` and awaiting confirmation by `verify_otp`.
///
/// It exists because the server previously kept no record of the secret it had just
/// generated: `verify_otp` took the secret back from the request body and checked the
/// submitted code against it, which is a tautology that always succeeds. The
/// enrolment is the server-side state that makes the check meaningful (FK-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtpEnrollment {
    pub id: Uuid,
    pub user_id: Uuid,
    /// Base32, as stored for active OTP credentials.
    pub secret: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[cfg_attr(test, mockall::automock)]
pub trait OtpEnrollmentRepository: Send + Sync {
    /// Record a candidate secret, replacing any enrolment already pending for this
    /// user. Restarting the setup flow must invalidate the previous candidate.
    fn start_enrollment(
        &self,
        user_id: Uuid,
        secret: String,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<OtpEnrollment, CoreError>> + Send;

    /// Atomically claim the live enrolment for this user, or return `None` when there
    /// is none, it has expired, or it has already been claimed.
    ///
    /// Single-use is enforced here rather than by the caller: two concurrent
    /// `verify_otp` calls must not both succeed against the same candidate secret.
    fn consume_enrollment(
        &self,
        user_id: Uuid,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<OtpEnrollment>, CoreError>> + Send;

    /// Drop any pending enrolment for this user, used when an enrolment is abandoned.
    fn clear_enrollments(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

/// Input for self-service (Bearer-authenticated) passkey registration options.
/// Unlike the login-flow variant it does not carry a session code: the pending
/// challenge is keyed by user id instead of an auth session.
pub struct PasskeyRegisterOptionsSelfServiceInput {
    pub rp_info: WebAuthnRpInfo,
}

/// Input for self-service (Bearer-authenticated) passkey registration
/// completion.
pub struct PasskeyRegisterSelfServiceInput {
    pub rp_info: WebAuthnRpInfo,
    pub credential: RegisterPublicKeyCredential,
}

/// Input for completing a password reset using a recovery code instead of the
/// email reset token.
pub struct CompletePasswordResetWithRecoveryCodeInput {
    pub realm_name: String,
    pub email: String,
    pub code: String,
    pub format: String,
    pub new_password: String,
}

/// Input for re-authenticating a signed-in user before a sensitive operation
/// (e.g. re-setting up 2FA). Requires the account password, and the current
/// OTP code when the user already has an authenticator configured.
pub struct ReauthenticateInput {
    pub password: String,
    pub otp_code: Option<String>,
}

#[cfg_attr(test, mockall::automock)]
pub trait RecoveryCodeRepository: Send + Sync {
    fn generate_recovery_code(&self) -> MfaRecoveryCode;
    fn generate_n_recovery_code(&self, n: usize) -> Vec<MfaRecoveryCode> {
        let mut out = Vec::<MfaRecoveryCode>::with_capacity(n);
        for _ in 0..n {
            out.push(self.generate_recovery_code());
        }
        out
    }

    /// Returns a string safe for long term storage
    /// Generally this is just hashing the code using an internal hasher
    fn secure_for_storage(
        &self,
        code: &MfaRecoveryCode,
    ) -> impl Future<Output = Result<HashResult, CoreError>> + Send;

    /// Compares the given human-readable formatted code against a stored credential
    fn verify(
        &self,
        in_code: &MfaRecoveryCode,
        secret_data: &str,
        hash_iterations: u32,
        algorithm: &str,
        salt: &str,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub trait RecoveryCodeFormatter: Send + Sync {
    /// Returns a formatted string representing the code
    fn format(code: &MfaRecoveryCode) -> String;
    /// Returns wether or not a user string matches the expected format
    /// for this formatter.
    /// `decode` implementations must call this beforehand
    fn validate(code: &str) -> bool;
    /// Builds a code from a user string
    fn decode(code: String) -> Result<MfaRecoveryCode, CoreError>;
}

pub trait TridentService: Send + Sync {
    fn generate_recovery_code(
        &self,
        identity: Identity,
        input: GenerateRecoveryCodeInput,
    ) -> impl Future<Output = Result<GenerateRecoveryCodeOutput, CoreError>> + Send;
    fn burn_recovery_code(
        &self,
        identity: Identity,
        input: BurnRecoveryCodeInput,
    ) -> impl Future<Output = Result<BurnRecoveryCodeOutput, CoreError>> + Send;
    fn webauthn_public_key_create_options(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyCreateOptionsInput,
    ) -> impl Future<Output = Result<WebAuthnPublicKeyCreateOptionsOutput, CoreError>> + Send;
    fn webauthn_public_key_create(
        &self,
        identity: Identity,
        input: WebAuthnValidatePublicKeyInput,
    ) -> impl Future<Output = Result<WebAuthnValidatePublicKeyOutput, CoreError>> + Send;
    fn webauthn_public_key_request_options(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyRequestOptionsInput,
    ) -> impl Future<Output = Result<WebAuthnPublicKeyRequestOptionsOutput, CoreError>> + Send;
    fn webauthn_public_key_authenticate(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyAuthenticateInput,
    ) -> impl Future<Output = Result<WebAuthnPublicKeyAuthenticateOutput, CoreError>> + Send;

    fn challenge_otp(
        &self,
        identity: Identity,
        input: ChallengeOtpInput,
    ) -> impl Future<Output = Result<ChallengeOtpOutput, CoreError>> + Send;
    fn setup_otp(
        &self,
        identity: Identity,
        input: SetupOtpInput,
    ) -> impl Future<Output = Result<SetupOtpOutput, CoreError>> + Send;
    fn update_password(
        &self,
        identity: Identity,
        input: UpdatePasswordInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn verify_otp(
        &self,
        identity: Identity,
        input: VerifyOtpInput,
    ) -> impl Future<Output = Result<VerifyOtpOutput, CoreError>> + Send;
    fn passkey_request_options(
        &self,
        input: PasskeyRequestOptionsInput,
    ) -> impl Future<Output = Result<WebAuthnPublicKeyRequestOptionsOutput, CoreError>> + Send;

    fn passkey_authenticate(
        &self,
        input: PasskeyAuthenticateInput,
    ) -> impl Future<Output = Result<PasskeyAuthenticateOutput, CoreError>> + Send;

    fn generate_magic_link(
        &self,
        input: MagicLinkInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn verify_magic_link(
        &self,
        input: VerifyMagicLinkInput,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;

    fn request_password_reset(
        &self,
        input: RequestPasswordResetInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn complete_password_reset(
        &self,
        input: CompletePasswordResetInput,
    ) -> impl Future<Output = Result<CompletePasswordResetOutput, CoreError>> + Send;

    fn verify_reset_token(
        &self,
        input: VerifyResetTokenInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Start a passkey registration from an authenticated (Bearer) workspace,
    /// without requiring a `FERRISKEY_SESSION` cookie or a temporary
    /// login-action token. The pending challenge is stored keyed by user id.
    fn passkey_register_options_self_service(
        &self,
        identity: Identity,
        input: PasskeyRegisterOptionsSelfServiceInput,
    ) -> impl Future<Output = Result<WebAuthnPublicKeyCreateOptionsOutput, CoreError>> + Send;

    /// Complete a passkey registration started via
    /// `passkey_register_options_self_service`.
    fn passkey_register_self_service(
        &self,
        identity: Identity,
        input: PasskeyRegisterSelfServiceInput,
    ) -> impl Future<Output = Result<WebAuthnValidatePublicKeyOutput, CoreError>> + Send;

    /// Complete a password reset using a recovery code instead of the email
    /// reset token. Consumes the matched recovery code and returns the same
    /// output as `complete_password_reset`.
    fn complete_password_reset_with_recovery_code(
        &self,
        input: CompletePasswordResetWithRecoveryCodeInput,
    ) -> impl Future<Output = Result<CompletePasswordResetOutput, CoreError>> + Send;

    /// Re-authenticate a signed-in user by verifying the account password and,
    /// when an authenticator is configured, the current OTP code. Used before
    /// sensitive self-service operations such as re-setting up 2FA.
    fn reauthenticate(
        &self,
        identity: Identity,
        input: ReauthenticateInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// List the signed-in user's own credentials (otp, passkey, recovery
    /// codes). Bearer self-service variant that does not require realm admin
    /// permissions, unlike the admin `GET /users/{id}/credentials` endpoint.
    fn list_credentials_self_service(
        &self,
        identity: Identity,
    ) -> impl Future<Output = Result<Vec<CredentialOverview>, CoreError>> + Send;

    /// Delete one of the signed-in user's own credentials. Bearer self-service
    /// variant; the credential must belong to the authenticated user.
    fn delete_credential_self_service(
        &self,
        identity: Identity,
        credential_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait MagicLinkRepository: Send + Sync {
    fn create_magic_link(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        magic_token_id: Uuid,
        magic_token_hash: &HashResult,
        expires_at: DateTime<Utc>,
        auth_session_code: Option<Uuid>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_by_token_id(
        &self,
        magic_token_id: Uuid,
    ) -> impl Future<Output = Result<Option<MagicLink>, CoreError>> + Send;

    fn delete_by_token_id(
        &self,
        magic_token_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn cleanup_expired(&self, realm_id: Uuid)
    -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait PasswordResetTokenRepository: Send + Sync {
    fn create(
        &self,
        token: &PasswordResetToken,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_by_token_id(
        &self,
        token_id: Uuid,
    ) -> impl Future<Output = Result<Option<PasswordResetToken>, CoreError>> + Send;

    fn delete_by_token_id(
        &self,
        token_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete_all_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn count_active_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<i64, CoreError>> + Send;

    fn cleanup_expired(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
}
