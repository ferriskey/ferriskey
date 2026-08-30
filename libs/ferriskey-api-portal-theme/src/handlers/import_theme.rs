use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::portal_theme::{ImportPortalThemeInput, ImportPortalThemeLayout};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_theme::entities::{PortalPageType, PortalTheme},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::validators::ImportPortalThemeValidator;
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    export::{ThemeExportEnvelope, ensure_importable},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportPortalThemeResponse {
    pub data: PortalTheme,
}

/// Reads the page trees out of the file.
///
/// Pages are optional and named by page type: a file that carries only some of
/// them imports the ones it has, and the theme is simply not activatable until
/// the rest are filled in — which is the same state a hand-built theme is in.
/// An unknown page name is refused rather than dropped, since silently losing a
/// page is how an import looks successful and renders wrong.
fn theme_pages(
    pages: serde_json::Value,
) -> Result<Vec<(PortalPageType, serde_json::Value)>, ApiError> {
    let serde_json::Value::Object(pages) = pages else {
        return Err(ApiError::BadRequest("'pages' must be an object".into()));
    };

    pages
        .into_iter()
        .map(|(name, tree)| {
            let page_type = serde_json::from_value::<PortalPageType>(serde_json::Value::String(
                name.clone(),
            ))
            .map_err(|_| ApiError::BadRequest(format!("unknown page type '{name}'").into()))?;

            Ok((page_type, tree))
        })
        .collect()
}

#[utoipa::path(
    post,
    path = "/portal/themes/import",
    tag = "portal-theme",
    summary = "Import a portal theme",
    description = "Creates a portal theme from an exported JSON envelope: its design tokens, its page \
                   trees, and the layout carried inside the file, which is recreated and bound to the \
                   new theme.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = ImportPortalThemeValidator,
    responses(
        (status = 201, description = "Theme imported successfully", body = ImportPortalThemeResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn import_theme(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ImportPortalThemeValidator>,
) -> Result<Response<ImportPortalThemeResponse>, ApiError> {
    ensure_importable(
        payload.ferriskey.as_deref(),
        payload.version,
        ThemeExportEnvelope::KIND,
    )?;

    let config = serde_json::from_value(payload.config)
        .map_err(|e| ApiError::BadRequest(format!("invalid theme config: {e}").into()))?;

    let theme = state
        .service
        .import_portal_theme(
            identity,
            ImportPortalThemeInput {
                realm_name,
                name: payload.name,
                config,
                pages: theme_pages(payload.pages)?,
                layout: payload.layout.map(|layout| ImportPortalThemeLayout {
                    name: layout.name,
                    tree: layout.tree,
                }),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(ImportPortalThemeResponse { data: theme }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_names_are_read_as_page_types() {
        let pages = serde_json::json!({ "login": [], "device_verify": [] });

        let parsed = theme_pages(pages).expect("both names are valid page types");

        assert_eq!(parsed.len(), 2);
        assert!(parsed.iter().any(|(pt, _)| *pt == PortalPageType::Login));
        assert!(
            parsed
                .iter()
                .any(|(pt, _)| *pt == PortalPageType::DeviceVerify)
        );
    }

    #[test]
    fn an_unknown_page_name_is_refused_rather_than_dropped() {
        let pages = serde_json::json!({ "login": [], "checkout": [] });

        assert!(
            theme_pages(pages).is_err(),
            "a page the server cannot place must fail the import"
        );
    }

    #[test]
    fn pages_must_be_an_object() {
        assert!(theme_pages(serde_json::json!([])).is_err());
    }
}
