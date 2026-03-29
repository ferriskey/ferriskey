use uuid::Uuid;

use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        email_template::{
            entities::EmailTemplate,
            ports::{
                CreateEmailTemplateInput, DeleteEmailTemplateInput, EmailTemplateService,
                GetEmailTemplateInput, GetEmailTemplatesInput, UpdateEmailTemplateInput,
            },
        },
    },
};

impl EmailTemplateService for ApplicationService {
    async fn get_templates_by_realm(
        &self,
        identity: Identity,
        input: GetEmailTemplatesInput,
    ) -> Result<Vec<EmailTemplate>, CoreError> {
        self.email_template_service
            .get_templates_by_realm(identity, input)
            .await
    }

    async fn get_template(
        &self,
        identity: Identity,
        input: GetEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        self.email_template_service
            .get_template(identity, input)
            .await
    }

    async fn create_template(
        &self,
        identity: Identity,
        input: CreateEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        self.email_template_service
            .create_template(identity, input)
            .await
    }

    async fn update_template(
        &self,
        identity: Identity,
        input: UpdateEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        self.email_template_service
            .update_template(identity, input)
            .await
    }

    async fn delete_template(
        &self,
        identity: Identity,
        input: DeleteEmailTemplateInput,
    ) -> Result<(), CoreError> {
        self.email_template_service
            .delete_template(identity, input)
            .await
    }

    async fn render_template_html(&self, template_id: Uuid) -> Result<String, CoreError> {
        self.email_template_service
            .render_template_html(template_id)
            .await
    }
}
