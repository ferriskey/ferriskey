use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::{
        entities::{EmailTemplate, EmailType},
        ports::{EmailTemplateService, EmailTemplateSource, ImportEmailTemplateInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::validators::ImportEmailTemplateValidator;
use ferriskey_api_core::api_entities::export::{ExportEnvelope, ensure_importable};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ImportEmailTemplateResponse {
    pub data: EmailTemplate,
}

#[utoipa::path(
    post,
    path = "/import",
    tag = "email-template",
    summary = "Import email template",
    description = "Creates an email template from an exported JSON structure or from raw MJML markup. \
                   MJML is parsed back into a builder structure so the imported template stays editable; \
                   anything outside <mj-body> (such as <mj-head>) is not representable in the builder and is dropped.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = ImportEmailTemplateValidator,
    responses(
        (status = 201, description = "Email template imported successfully", body = ImportEmailTemplateResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn import_template(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ImportEmailTemplateValidator>,
) -> Result<Response<ImportEmailTemplateResponse>, ApiError> {
    ensure_importable(
        payload.ferriskey.as_deref(),
        payload.version,
        ExportEnvelope::EMAIL_TEMPLATE,
    )?;

    let email_type = payload
        .email_type
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ApiError::BadRequest("email_type is required".into()))?;
    let email_type = EmailType::try_from(email_type).map_err(ApiError::from)?;
    let source = template_source(&payload.structure, &payload.tree, &payload.mjml)?;

    let template = state
        .service
        .import_template(
            identity,
            ImportEmailTemplateInput {
                realm_name,
                name: payload.name,
                email_type,
                source,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(ImportEmailTemplateResponse {
        data: template,
    }))
}

/// Picks the single body source out of the payload. A bare `tree` array is
/// wrapped into the `{ children: [...] }` structure the renderer expects.
fn template_source(
    structure: &Option<serde_json::Value>,
    tree: &Option<serde_json::Value>,
    mjml: &Option<String>,
) -> Result<EmailTemplateSource, ApiError> {
    match (structure, tree, mjml) {
        (Some(structure), None, None) => Ok(EmailTemplateSource::Structure(structure.clone())),
        (None, Some(tree), None) => Ok(EmailTemplateSource::Structure(
            serde_json::json!({ "children": tree }),
        )),
        (None, None, Some(mjml)) => Ok(EmailTemplateSource::Mjml(mjml.clone())),
        (None, None, None) => Err(ApiError::BadRequest(
            "one of 'structure', 'tree' or 'mjml' is required".into(),
        )),
        _ => Err(ApiError::BadRequest(
            "only one of 'structure', 'tree' or 'mjml' may be provided".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_a_bare_tree_into_a_structure() {
        let tree = serde_json::json!([{ "type": "mj-section", "children": [] }]);

        let source = template_source(&None, &Some(tree.clone()), &None).expect("valid payload");

        let EmailTemplateSource::Structure(structure) = source else {
            panic!("expected a structure source");
        };
        assert_eq!(structure, serde_json::json!({ "children": tree }));
    }

    #[test]
    fn rejects_an_empty_payload() {
        assert!(template_source(&None, &None, &None).is_err());
    }

    #[test]
    fn rejects_more_than_one_source() {
        let structure = Some(serde_json::json!({ "children": [] }));
        let mjml = Some("<mjml><mj-body></mj-body></mjml>".to_string());

        assert!(template_source(&structure, &None, &mjml).is_err());
    }
}
