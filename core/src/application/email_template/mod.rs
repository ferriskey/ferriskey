use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::{
            entities::app_errors::CoreError,
            pagination::{Page, PageRequest},
        },
        email_template::{
            entities::{EmailTemplate, EmailTemplateFilter, EmailTemplateSortField},
            ports::{
                CreateEmailTemplateInput, DeleteEmailTemplateInput, EmailTemplateService,
                GetEmailTemplateInput, ImportEmailTemplateInput, RenderEmailTemplateInput,
                UpdateEmailTemplateInput,
            },
        },
    },
};

impl EmailTemplateService for ApplicationService {
    async fn list_templates(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<EmailTemplateFilter, EmailTemplateSortField>,
    ) -> Result<Page<EmailTemplate>, CoreError> {
        self.email_template_service
            .list_templates(identity, realm_name, request)
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

    async fn render_template_html(
        &self,
        identity: Identity,
        input: RenderEmailTemplateInput,
    ) -> Result<String, CoreError> {
        self.email_template_service
            .render_template_html(identity, input)
            .await
    }

    async fn import_template(
        &self,
        identity: Identity,
        input: ImportEmailTemplateInput,
    ) -> Result<EmailTemplate, CoreError> {
        self.email_template_service
            .import_template(identity, input)
            .await
    }
}
