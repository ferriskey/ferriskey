use axum::{
    Extension,
    extract::{Path, Query, State},
    response::Response as AxumResponse,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::{
        entities::EmailTemplate,
        ports::{EmailTemplateService, GetEmailTemplateInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::export::{ExportEnvelope, download, slugify};
use ferriskey_api_core::app_state::AppState;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EmailTemplateExportFormat {
    #[default]
    Json,
    Mjml,
}

#[derive(Debug, Default, Deserialize, IntoParams)]
pub struct ExportTemplateQuery {
    #[serde(default)]
    pub format: EmailTemplateExportFormat,
}

#[utoipa::path(
    get,
    path = "/{template_id}/export",
    tag = "email-template",
    summary = "Export email template",
    description = "Returns the template as a downloadable file: the builder JSON envelope (`format=json`, \
                   the default) or the rendered MJML markup (`format=mjml`).",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("template_id" = Uuid, Path, description = "Email template ID"),
        ExportTemplateQuery,
    ),
    responses(
        (status = 200, description = "Email template exported successfully", content_type = "application/json", body = ExportEnvelope),
        (status = 404, description = "Email template not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn export_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    Query(query): Query<ExportTemplateQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<AxumResponse, ApiError> {
    let template = state
        .service
        .get_template(
            identity,
            GetEmailTemplateInput {
                realm_name,
                template_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    match query.format {
        EmailTemplateExportFormat::Json => {
            template_envelope(&template).into_download().map_err(|e| {
                ApiError::InternalServerError(format!("failed to serialize template: {e}").into())
            })
        }
        // MJML leaves as the markup itself, not wrapped in an envelope: it is
        // what a mail designer opens in their own tooling.
        EmailTemplateExportFormat::Mjml => Ok(download(
            template.mjml.into_bytes(),
            "text/plain; charset=utf-8",
            &format!("{}.mjml", slugify(&template.name)),
        )),
    }
}

/// Builds the export envelope. `structure` is persisted as `{ children: [...] }`,
/// while the envelope carries the bare `BuilderNode[]` the builder works with.
fn template_envelope(template: &EmailTemplate) -> ExportEnvelope {
    let tree = template
        .structure
        .get("children")
        .cloned()
        .unwrap_or_else(|| template.structure.clone());

    ExportEnvelope::email_template(template.name.clone(), template.email_type.to_string(), tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ferriskey_core::domain::email_template::entities::EmailType;

    fn template(structure: serde_json::Value) -> EmailTemplate {
        EmailTemplate {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            name: "Reset password".to_string(),
            email_type: EmailType::ResetPassword,
            structure,
            mjml: "<mjml><mj-body></mj-body></mjml>".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn envelope_unwraps_the_persisted_children_field() {
        let tree = serde_json::json!([{ "type": "mj-section", "children": [] }]);
        let envelope = template_envelope(&template(serde_json::json!({ "children": tree })));

        assert_eq!(envelope.ferriskey, "email-template");
        assert_eq!(envelope.email_type.as_deref(), Some("reset_password"));
        assert_eq!(envelope.tree, tree);
    }

    #[test]
    fn envelope_falls_back_to_the_raw_structure() {
        let structure = serde_json::json!([{ "type": "mj-section" }]);
        let envelope = template_envelope(&template(structure.clone()));

        assert_eq!(envelope.tree, structure);
    }
}
