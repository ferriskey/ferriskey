use crate::{
    ApplicationService,
    domain::{
        account_security::{
            entities::{
                ChangeOwnPasswordInput, ConfirmOwnOtpEnrollmentInput,
                ConfirmOwnPasskeyRegistrationInput, DeleteOwnPasskeyInput, DisableOwnOtpInput,
                ListOwnCredentialsInput, OwnCredential, RequestElevationInput,
                RequestElevationOutput, StartOwnOtpEnrollmentInput, StartOwnOtpEnrollmentOutput,
                StartOwnPasskeyRegistrationInput, StartOwnPasskeyRegistrationOutput,
            },
            ports::AccountSecurityService,
        },
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
    },
};

impl AccountSecurityService for ApplicationService {
    async fn request_elevation(
        &self,
        identity: Identity,
        input: RequestElevationInput,
    ) -> Result<RequestElevationOutput, CoreError> {
        self.account_security_service
            .request_elevation(identity, input)
            .await
    }

    async fn change_own_password(
        &self,
        identity: Identity,
        input: ChangeOwnPasswordInput,
    ) -> Result<(), CoreError> {
        self.account_security_service
            .change_own_password(identity, input)
            .await
    }

    async fn start_own_otp_enrollment(
        &self,
        identity: Identity,
        input: StartOwnOtpEnrollmentInput,
    ) -> Result<StartOwnOtpEnrollmentOutput, CoreError> {
        self.account_security_service
            .start_own_otp_enrollment(identity, input)
            .await
    }

    async fn confirm_own_otp_enrollment(
        &self,
        identity: Identity,
        input: ConfirmOwnOtpEnrollmentInput,
    ) -> Result<(), CoreError> {
        self.account_security_service
            .confirm_own_otp_enrollment(identity, input)
            .await
    }

    async fn disable_own_otp(
        &self,
        identity: Identity,
        input: DisableOwnOtpInput,
    ) -> Result<(), CoreError> {
        self.account_security_service
            .disable_own_otp(identity, input)
            .await
    }

    async fn list_own_credentials(
        &self,
        identity: Identity,
        input: ListOwnCredentialsInput,
    ) -> Result<Vec<OwnCredential>, CoreError> {
        self.account_security_service
            .list_own_credentials(identity, input)
            .await
    }

    async fn start_own_passkey_registration(
        &self,
        identity: Identity,
        input: StartOwnPasskeyRegistrationInput,
    ) -> Result<StartOwnPasskeyRegistrationOutput, CoreError> {
        self.account_security_service
            .start_own_passkey_registration(identity, input)
            .await
    }

    async fn confirm_own_passkey_registration(
        &self,
        identity: Identity,
        input: ConfirmOwnPasskeyRegistrationInput,
    ) -> Result<(), CoreError> {
        self.account_security_service
            .confirm_own_passkey_registration(identity, input)
            .await
    }

    async fn delete_own_passkey(
        &self,
        identity: Identity,
        input: DeleteOwnPasskeyInput,
    ) -> Result<(), CoreError> {
        self.account_security_service
            .delete_own_passkey(identity, input)
            .await
    }
}
