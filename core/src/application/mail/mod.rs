use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        realm::{
            entities::SmtpConfig,
            ports::{
                DeleteSmtpConfigInput, GetSmtpConfigInput, MailService, UpsertSmtpConfigInput,
            },
        },
    },
};

impl MailService for ApplicationService {
    async fn get_smtp_config(
        &self,
        identity: Identity,
        input: GetSmtpConfigInput,
    ) -> Result<SmtpConfig, CoreError> {
        self.mail_service.get_smtp_config(identity, input).await
    }

    async fn upsert_smtp_config(
        &self,
        identity: Identity,
        input: UpsertSmtpConfigInput,
    ) -> Result<SmtpConfig, CoreError> {
        self.mail_service.upsert_smtp_config(identity, input).await
    }

    async fn delete_smtp_config(
        &self,
        identity: Identity,
        input: DeleteSmtpConfigInput,
    ) -> Result<(), CoreError> {
        self.mail_service.delete_smtp_config(identity, input).await
    }
}
