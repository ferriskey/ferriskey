use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use ferriskey_api_core::api_entities::api_error::ApiError;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::common::entities::app_errors::CoreError;
use ferriskey_core::domain::saml::entities::FinishSsoInput;
use ferriskey_core::domain::saml::ports::SamlService;
use ferriskey_saml::binding::encode_post;
use serde::Deserialize;
use tracing::warn;
use utoipa::{IntoParams, ToSchema};

use crate::html::{auto_submit_form, error_page, html_page};
use crate::origin::saml_public_base_url;

const MISCONFIGURED: &str =
    "Single sign-on is not configured on this server. Set SERVER_PUBLIC_URL.";

const EXPIRED_CONTINUATION: &str =
    "This sign-in link is no longer valid. Start again from your application.";
const UNAVAILABLE: &str =
    "The single sign-on could not be completed. Start again from your application.";

#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SamlContinueParams {
    pub code: String,
}

#[utoipa::path(
    get,
    path = "/protocol/saml/continue",
    tag = "saml",
    summary = "Deliver a SAML assertion to the service provider",
    description = "Exchanges the authorization code minted by the login flow for a signed SAML response and renders the self-submitting form that posts it to the assertion consumer service.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        SamlContinueParams
    ),
    responses(
        (status = 200, description = "The self-submitting HTML form carrying the signed response", content_type = "text/html"),
        (status = 400, description = "The authorization code was spent, expired or unknown, rendered as HTML"),
        (status = 500, description = "Internal Server Error, rendered as HTML")
    )
)]
pub async fn saml_continue(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<SamlContinueParams>,
) -> Result<Response, ApiError> {
    let Some(public_base_url) = saml_public_base_url(
        state.args.server.public_url.as_deref(),
        &state.args.server.root_path,
    ) else {
        warn!("refusing a saml request: SERVER_PUBLIC_URL is not configured");
        return Ok(error_page(StatusCode::INTERNAL_SERVER_ERROR, MISCONFIGURED));
    };

    let delivery = match state
        .service
        .finish_sso(FinishSsoInput {
            realm_name: realm_name.clone(),
            authorization_code: params.code,
            public_base_url,
        })
        .await
    {
        Ok(delivery) => delivery,
        Err(error) => return Ok(refusal(&realm_name, error)),
    };

    Ok(html_page(
        StatusCode::OK,
        auto_submit_form(
            delivery.acs_url.as_str(),
            &encode_post(&delivery.signed_response),
            delivery.relay_state.as_deref(),
        ),
    ))
}

fn refusal(realm_name: &str, error: CoreError) -> Response {
    let (status, message) = match &error {
        CoreError::InvalidAuthorizationCode
        | CoreError::MissingAuthorizationCode
        | CoreError::SessionExpired
        | CoreError::SessionNotFound
        | CoreError::InvalidSession
        | CoreError::InvalidRealm
        | CoreError::InvalidClient
        | CoreError::UserDisabled => (StatusCode::BAD_REQUEST, EXPIRED_CONTINUATION),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, UNAVAILABLE),
    };

    warn!(
        realm = %realm_name,
        %error,
        "refusing to deliver a saml assertion"
    );

    error_page(status, message)
}

#[cfg(test)]
mod tests {
    use super::refusal;

    use axum::http::StatusCode;
    use ferriskey_core::domain::common::entities::app_errors::CoreError;

    #[test]
    fn a_code_that_was_already_spent_is_a_client_error() {
        assert_eq!(
            refusal("master", CoreError::InvalidAuthorizationCode).status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn an_expired_login_is_a_client_error() {
        assert_eq!(
            refusal("master", CoreError::SessionExpired).status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn a_realm_that_cannot_sign_is_reported_as_our_failure() {
        assert_eq!(
            refusal("master", CoreError::RealmKeyNotFound).status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
