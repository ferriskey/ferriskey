use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use ferriskey_api_core::api_entities::api_error::ApiError;
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::url::FullUrl;
use ferriskey_core::domain::common::entities::app_errors::CoreError;
use ferriskey_core::domain::saml::ports::SamlService;
use tracing::warn;

use crate::html::error_page;
use crate::metadata::{SAML_METADATA_CONTENT_TYPE, idp_metadata_document};
use crate::origin::saml_public_base_url;

const UNKNOWN_REALM: &str = "This realm does not publish SAML metadata.";
const UNAVAILABLE: &str = "SAML metadata is not available for this realm.";

#[utoipa::path(
    get,
    path = "/protocol/saml/descriptor",
    tag = "saml",
    summary = "Publish the realm SAML metadata",
    description = "Returns the IdP entity descriptor a service provider needs to trust this realm: its entity id, its single sign-on endpoints and the certificate that signs assertions.",
    params(("realm_name" = String, Path, description = "Realm name")),
    responses(
        (status = 200, description = "The realm entity descriptor", content_type = "application/samlmetadata+xml"),
        (status = 404, description = "The realm does not exist, rendered as HTML"),
        (status = 500, description = "Internal Server Error, rendered as HTML")
    )
)]
pub async fn saml_descriptor(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
) -> Result<Response, ApiError> {
    let public_base_url = saml_public_base_url(
        state.args.server.public_url.as_deref(),
        &base_url,
        &state.args.server.root_path,
    );

    let certificate = match state
        .service
        .idp_signing_certificate(realm_name.clone())
        .await
    {
        Ok(certificate) => certificate,
        Err(error) => return Ok(refusal(&realm_name, error)),
    };

    let document =
        idp_metadata_document(&public_base_url, &realm_name, certificate).map_err(|error| {
            warn!(realm = %realm_name, %error, "the realm entity descriptor could not be rendered");
            ApiError::InternalServerError("saml metadata could not be rendered".into())
        })?;

    Ok(([(CONTENT_TYPE, SAML_METADATA_CONTENT_TYPE)], document).into_response())
}

fn refusal(realm_name: &str, error: CoreError) -> Response {
    let (status, message) = match &error {
        CoreError::InvalidRealm | CoreError::NotFound => (StatusCode::NOT_FOUND, UNKNOWN_REALM),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, UNAVAILABLE),
    };

    warn!(
        realm = %realm_name,
        %error,
        "refusing to publish saml metadata"
    );

    error_page(status, message)
}

#[cfg(test)]
mod tests {
    use super::refusal;

    use axum::http::StatusCode;
    use ferriskey_core::domain::common::entities::app_errors::CoreError;

    #[test]
    fn an_unknown_realm_is_a_not_found_rather_than_a_server_error() {
        assert_eq!(
            refusal("nope", CoreError::InvalidRealm).status(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn a_realm_without_signing_material_is_reported_as_our_failure() {
        assert_eq!(
            refusal("master", CoreError::RealmKeyNotFound).status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
