use axum::{
    Extension,
    extract::{Path, State},
    response::Response as AxumResponse,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::ports::{GetLayoutInput, PortalLayoutsService},
    portal_theme::{
        entities::{PortalPageType, PortalThemePages},
        ports::{GetThemeByIdInput, PortalThemeService},
    },
};
use uuid::Uuid;

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::export::{ThemeExportEnvelope, ThemeExportLayout};
use ferriskey_api_core::app_state::AppState;

/// Keys the pages by their canonical page-type name.
///
/// `PortalThemePages` serialises its fields in camelCase, while a page type is
/// `snake_case` on the wire — importing what the struct produces would fail on
/// the very file this endpoint just wrote. Building the map from the page types
/// themselves keeps the file format tied to the names the import understands,
/// whatever the struct does internally.
fn export_pages(pages: &PortalThemePages) -> serde_json::Value {
    let entries = PortalPageType::ALL.iter().filter_map(|page_type| {
        let key = serde_json::to_value(page_type).ok()?;
        Some((key.as_str()?.to_string(), pages.get(*page_type).clone()))
    });

    serde_json::Value::Object(entries.collect())
}

#[utoipa::path(
    get,
    path = "/portal/themes/{theme_id}/export",
    tag = "portal-theme",
    summary = "Export a portal theme",
    description = "Returns the theme as a downloadable JSON envelope: its design tokens, every page tree, \
                   and the layout it is framed by, carried by value so the file can be imported into \
                   another realm or deployment.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("theme_id" = Uuid, Path, description = "Portal theme ID"),
    ),
    responses(
        (status = 200, description = "Theme exported successfully", content_type = "application/json", body = ThemeExportEnvelope),
        (status = 404, description = "Theme not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn export_theme(
    Path((realm_name, theme_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<AxumResponse, ApiError> {
    let theme = state
        .service
        .get_theme_by_id(
            identity.clone(),
            GetThemeByIdInput {
                realm_name: realm_name.clone(),
                theme_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    // A layout the theme names but that no longer exists is not worth failing
    // an export over: the theme's own tokens and pages are what the file is
    // for, and the import falls back to the realm's default layout.
    let layout = match theme.layout_id {
        Some(layout_id) => state
            .service
            .get_layout(
                identity,
                GetLayoutInput {
                    realm_name,
                    layout_id,
                },
            )
            .await
            .ok()
            .map(|layout| ThemeExportLayout {
                name: layout.name,
                tree: layout.tree,
            }),
        None => None,
    };

    let config = serde_json::to_value(&theme.config).map_err(|e| {
        ApiError::InternalServerError(format!("failed to serialize theme: {e}").into())
    })?;
    let pages = export_pages(&theme.pages);

    ThemeExportEnvelope::new(theme.name, config, pages, layout)
        .into_download()
        .map_err(|e| {
            ApiError::InternalServerError(format!("failed to serialize theme: {e}").into())
        })
}
