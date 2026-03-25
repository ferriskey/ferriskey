use std::future::Future;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    email_template::entities::{EmailTemplate, EmailType},
    realm::entities::Realm,
};

pub trait EmailTemplateService: Send + Sync {
    fn get_templates_by_realm(
        &self,
        identity: Identity,
        input: GetEmailTemplatesInput,
    ) -> impl Future<Output = Result<Vec<EmailTemplate>, CoreError>> + Send;

    fn get_template(
        &self,
        identity: Identity,
        input: GetEmailTemplateInput,
    ) -> impl Future<Output = Result<EmailTemplate, CoreError>> + Send;

    fn create_template(
        &self,
        identity: Identity,
        input: CreateEmailTemplateInput,
    ) -> impl Future<Output = Result<EmailTemplate, CoreError>> + Send;

    fn update_template(
        &self,
        identity: Identity,
        input: UpdateEmailTemplateInput,
    ) -> impl Future<Output = Result<EmailTemplate, CoreError>> + Send;

    fn delete_template(
        &self,
        identity: Identity,
        input: DeleteEmailTemplateInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn activate_template(
        &self,
        identity: Identity,
        input: ActivateEmailTemplateInput,
    ) -> impl Future<Output = Result<EmailTemplate, CoreError>> + Send;

    fn get_active_template_html(
        &self,
        realm_id: Uuid,
        email_type: EmailType,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait EmailTemplateRepository: Send + Sync {
    fn fetch_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<EmailTemplate>, CoreError>> + Send;

    fn get_by_id(
        &self,
        template_id: Uuid,
    ) -> impl Future<Output = Result<Option<EmailTemplate>, CoreError>> + Send;

    fn get_active_by_type(
        &self,
        realm_id: Uuid,
        email_type: String,
    ) -> impl Future<Output = Result<Option<EmailTemplate>, CoreError>> + Send;

    fn create(
        &self,
        realm_id: Uuid,
        name: String,
        email_type: String,
        structure: serde_json::Value,
        mjml: String,
    ) -> impl Future<Output = Result<EmailTemplate, CoreError>> + Send;

    fn update(
        &self,
        template_id: Uuid,
        name: String,
        structure: serde_json::Value,
        mjml: String,
    ) -> impl Future<Output = Result<EmailTemplate, CoreError>> + Send;

    fn delete(&self, template_id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn activate(
        &self,
        realm_id: Uuid,
        email_type: String,
        template_id: Uuid,
    ) -> impl Future<Output = Result<EmailTemplate, CoreError>> + Send;
}

/// Trait for rendering a builder structure JSON into an intermediate format (e.g. MJML)
/// and then into final HTML. This abstraction allows swapping the renderer
/// without changing domain logic.
pub trait TemplateRenderer: Send + Sync {
    /// Converts the builder structure (JSON) into an intermediate representation (e.g. MJML string).
    fn render_to_intermediate(&self, structure: &serde_json::Value) -> Result<String, CoreError>;

    /// Converts the intermediate representation into final HTML.
    fn render_to_html(&self, intermediate: &str) -> Result<String, CoreError>;
}

pub trait EmailTemplatePolicy: Send + Sync {
    fn can_view_email_template(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_email_template(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub struct GetEmailTemplatesInput {
    pub realm_name: String,
}

pub struct GetEmailTemplateInput {
    pub realm_name: String,
    pub template_id: Uuid,
}

pub struct CreateEmailTemplateInput {
    pub realm_name: String,
    pub name: String,
    pub email_type: EmailType,
    pub structure: serde_json::Value,
}

pub struct UpdateEmailTemplateInput {
    pub realm_name: String,
    pub template_id: Uuid,
    pub name: String,
    pub structure: serde_json::Value,
}

pub struct DeleteEmailTemplateInput {
    pub realm_name: String,
    pub template_id: Uuid,
}

pub struct ActivateEmailTemplateInput {
    pub realm_name: String,
    pub template_id: Uuid,
}
