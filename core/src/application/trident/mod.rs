use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        trident::ports::{
            BurnRecoveryCodeInput, BurnRecoveryCodeOutput, ChallengeOtpInput, ChallengeOtpOutput,
            CompletePasswordResetInput, CompletePasswordResetOutput, GenerateRecoveryCodeInput,
            GenerateRecoveryCodeOutput, MagicLinkInput, PasskeyAuthenticateInput,
            PasskeyAuthenticateOutput, PasskeyRequestOptionsInput, RequestPasswordResetInput,
            SetupOtpInput, SetupOtpOutput, TridentService, UpdatePasswordInput,
            VerifyMagicLinkInput, VerifyOtpInput, VerifyOtpOutput, VerifyResetTokenInput,
            WebAuthnPublicKeyAuthenticateInput, WebAuthnPublicKeyAuthenticateOutput,
            WebAuthnPublicKeyCreateOptionsInput, WebAuthnPublicKeyCreateOptionsOutput,
            WebAuthnPublicKeyRequestOptionsInput, WebAuthnPublicKeyRequestOptionsOutput,
            WebAuthnValidatePublicKeyInput, WebAuthnValidatePublicKeyOutput,
        },
    },
};

impl TridentService for ApplicationService {
    async fn burn_recovery_code(
        &self,
        identity: Identity,
        input: BurnRecoveryCodeInput,
    ) -> Result<BurnRecoveryCodeOutput, CoreError> {
        self.trident_service
            .burn_recovery_code(identity, input)
            .await
    }

    async fn challenge_otp(
        &self,
        identity: Identity,
        input: ChallengeOtpInput,
    ) -> Result<ChallengeOtpOutput, CoreError> {
        self.trident_service.challenge_otp(identity, input).await
    }

    async fn generate_recovery_code(
        &self,
        identity: Identity,
        input: GenerateRecoveryCodeInput,
    ) -> Result<GenerateRecoveryCodeOutput, CoreError> {
        self.trident_service
            .generate_recovery_code(identity, input)
            .await
    }

    async fn setup_otp(
        &self,
        identity: Identity,
        input: SetupOtpInput,
    ) -> Result<SetupOtpOutput, CoreError> {
        self.trident_service.setup_otp(identity, input).await
    }

    async fn update_password(
        &self,
        identity: Identity,
        input: UpdatePasswordInput,
    ) -> Result<(), CoreError> {
        self.trident_service.update_password(identity, input).await
    }

    async fn verify_otp(
        &self,
        identity: Identity,
        input: VerifyOtpInput,
    ) -> Result<VerifyOtpOutput, CoreError> {
        self.trident_service.verify_otp(identity, input).await
    }

    async fn webauthn_public_key_authenticate(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyAuthenticateInput,
    ) -> Result<WebAuthnPublicKeyAuthenticateOutput, CoreError> {
        self.trident_service
            .webauthn_public_key_authenticate(identity, input)
            .await
    }

    async fn webauthn_public_key_create(
        &self,
        identity: Identity,
        input: WebAuthnValidatePublicKeyInput,
    ) -> Result<WebAuthnValidatePublicKeyOutput, CoreError> {
        self.trident_service
            .webauthn_public_key_create(identity, input)
            .await
    }

    async fn webauthn_public_key_create_options(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyCreateOptionsInput,
    ) -> Result<WebAuthnPublicKeyCreateOptionsOutput, CoreError> {
        self.trident_service
            .webauthn_public_key_create_options(identity, input)
            .await
    }

    async fn webauthn_public_key_request_options(
        &self,
        identity: Identity,
        input: WebAuthnPublicKeyRequestOptionsInput,
    ) -> Result<WebAuthnPublicKeyRequestOptionsOutput, CoreError> {
        self.trident_service
            .webauthn_public_key_request_options(identity, input)
            .await
    }

    async fn passkey_request_options(
        &self,
        input: PasskeyRequestOptionsInput,
    ) -> Result<WebAuthnPublicKeyRequestOptionsOutput, CoreError> {
        self.trident_service.passkey_request_options(input).await
    }

    async fn passkey_authenticate(
        &self,
        input: PasskeyAuthenticateInput,
    ) -> Result<PasskeyAuthenticateOutput, CoreError> {
        self.trident_service.passkey_authenticate(input).await
    }

    async fn generate_magic_link(&self, input: MagicLinkInput) -> Result<(), CoreError> {
        self.trident_service.generate_magic_link(input).await
    }

    async fn verify_magic_link(&self, input: VerifyMagicLinkInput) -> Result<String, CoreError> {
        self.trident_service.verify_magic_link(input).await
    }

    async fn request_password_reset(
        &self,
        input: RequestPasswordResetInput,
    ) -> Result<(), CoreError> {
        self.trident_service.request_password_reset(input).await
    }

    async fn complete_password_reset(
        &self,
        input: CompletePasswordResetInput,
    ) -> Result<CompletePasswordResetOutput, CoreError> {
        self.trident_service.complete_password_reset(input).await
    }

    async fn verify_reset_token(&self, input: VerifyResetTokenInput) -> Result<(), CoreError> {
        self.trident_service.verify_reset_token(input).await
    }
}
