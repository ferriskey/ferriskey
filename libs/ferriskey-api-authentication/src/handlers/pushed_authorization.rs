use axum::{
    Form, Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::CACHE_CONTROL},
    response::IntoResponse,
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::request_context::RequestContext;
use ferriskey_api_core::url::FullUrl;
use ferriskey_core::domain::authentication::pushed_authorization::entities::{
    AuthorizationParameters, PushAuthorizationRequestInput, PushedAuthorizationResponse,
};
use ferriskey_core::domain::authentication::value_objects::CodeChallengeMethod;
use ferriskey_core::domain::common::entities::app_errors::CoreError;
use serde::Deserialize;
use tracing::{instrument, warn};
use utoipa::ToSchema;

use super::auth::Prompt;
use super::authorization_server_metadata::realm_issuer;
use crate::client_credentials::{ClientAuthParams, assertion_audiences, presented_client};

pub const PUSHED_AUTHORIZATION_REQUEST_PATH: &str = "/protocol/openid-connect/ext/par/request";

/// Form body of a pushed authorization request (RFC 9126 §2.1): the client
/// authentication of the token endpoint plus the parameters of `/auth`.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct PushedAuthorizationRequestForm {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub client_assertion_type: Option<String>,
    pub client_assertion: Option<String>,
    pub response_type: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    /// `S256` or `plain`.
    pub code_challenge_method: Option<String>,
    pub prompt: Option<String>,
    pub max_age: Option<i64>,
    pub resource: Option<String>,
    /// Refused: a pushed request cannot point at another one.
    pub request_uri: Option<String>,
    /// Refused: request objects (RFC 9101) are not supported.
    pub request: Option<String>,
}

fn invalid_request(description: &str) -> ApiError {
    ApiError::OAuthError {
        error: "invalid_request".into(),
        error_description: description.to_string().into(),
    }
}

fn required<'a>(value: &'a Option<String>, name: &str) -> Result<&'a str, ApiError> {
    value
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_request(&format!("{name} is required.")))
}

/// Reads the authorization parameters out of the form, refusing what `/auth`
/// would refuse before it looks at the client.
fn authorization_parameters(
    form: PushedAuthorizationRequestForm,
) -> Result<AuthorizationParameters, ApiError> {
    if form.request_uri.is_some() {
        return Err(invalid_request("request_uri cannot be pushed."));
    }

    if form.request.is_some() {
        return Err(ApiError::OAuthError {
            error: "request_not_supported".into(),
            error_description: "Request objects are not supported.".into(),
        });
    }

    let response_type = required(&form.response_type, "response_type")?.to_string();
    let redirect_uri = required(&form.redirect_uri, "redirect_uri")?.to_string();

    if Prompt::parse(form.prompt.as_deref()).is_none() {
        return Err(invalid_request("prompt=none cannot be combined."));
    }

    let code_challenge_method = form
        .code_challenge_method
        .as_deref()
        .map(str::parse::<CodeChallengeMethod>)
        .transpose()
        .map_err(|_| invalid_request("code_challenge_method is not supported."))?;

    Ok(AuthorizationParameters {
        response_type,
        redirect_uri,
        scope: form.scope,
        state: form.state,
        nonce: form.nonce,
        code_challenge: form.code_challenge,
        code_challenge_method,
        prompt: form.prompt,
        max_age: form.max_age,
        resource: form.resource,
    })
}

/// The errors of `/auth`, as the JSON errors of the token endpoint
/// (RFC 9126 §2.3).
fn pushed_request_error(error: CoreError) -> ApiError {
    match error {
        CoreError::InvalidRedirectUri => {
            invalid_request("redirect_uri is not registered for this client.")
        }
        CoreError::PkceRequired => invalid_request("This client must send an S256 code_challenge."),
        CoreError::InvalidClient | CoreError::ClientNotFound => ApiError::OAuthError {
            error: "unauthorized_client".into(),
            error_description: "This client cannot send authorization requests.".into(),
        },
        other => ApiError::from(other),
    }
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/ext/par/request",
    tag = "auth",
    summary = "Pushed authorization request",
    description = "Takes the parameters of an authorization request in the body, authenticates the client like the token endpoint, checks the request like the authorization endpoint and returns a single-use `request_uri` to send to `/protocol/openid-connect/auth` (RFC 9126).",
    request_body(
        content = PushedAuthorizationRequestForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 201, body = PushedAuthorizationResponse),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 401, description = "Client authentication failed", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
#[instrument(skip(state, form, headers, context), fields(realm_name = %realm_name))]
pub async fn push_authorization_request(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    context: RequestContext,
    headers: HeaderMap,
    Form(form): Form<PushedAuthorizationRequestForm>,
) -> Result<impl IntoResponse, ApiError> {
    let issuer = realm_issuer(&base_url, &state.args.server.root_path, &realm_name);
    let presented = presented_client(
        &headers,
        ClientAuthParams {
            client_id: form.client_id.as_deref(),
            client_secret: form.client_secret.as_deref(),
            client_assertion_type: form.client_assertion_type.as_deref(),
            client_assertion: form.client_assertion.as_deref(),
        },
        || assertion_audiences(&issuer, PUSHED_AUTHORIZATION_REQUEST_PATH),
    )?;

    if presented.client_id.is_empty() {
        return Err(invalid_request("client_id is required."));
    }

    if form
        .client_id
        .as_deref()
        .is_some_and(|client_id| client_id != presented.client_id)
    {
        return Err(invalid_request(
            "client_id does not match the authenticated client.",
        ));
    }

    let parameters = authorization_parameters(form)?;
    let client_id = presented.client_id;

    let response = state
        .service
        .push_authorization_request(PushAuthorizationRequestInput {
            realm_name,
            client_id: client_id.clone(),
            client_credential: presented.credential,
            parameters,
            ip_address: context.ip_address,
            user_agent: context.user_agent,
            issuer: Some(issuer),
        })
        .await
        .map_err(|error| {
            warn!(client_id = %client_id, error = %error, "Pushed authorization request refused");
            pushed_request_error(error)
        })?;

    Ok((
        StatusCode::CREATED,
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(response),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> PushedAuthorizationRequestForm {
        PushedAuthorizationRequestForm {
            response_type: Some("code".to_string()),
            redirect_uri: Some("https://app.example/cb".to_string()),
            ..Default::default()
        }
    }

    fn oauth_error(error: ApiError) -> String {
        match error {
            ApiError::OAuthError { error, .. } => error.into_owned(),
            other => panic!("not an OAuth error: {other:?}"),
        }
    }

    #[test]
    fn the_parameters_of_a_complete_form_are_kept() {
        let parameters = authorization_parameters(PushedAuthorizationRequestForm {
            scope: Some("openid".to_string()),
            state: Some("xyz".to_string()),
            code_challenge: Some("challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
            prompt: Some("login".to_string()),
            max_age: Some(60),
            ..form()
        })
        .expect("valid form");

        assert_eq!(parameters.response_type, "code");
        assert_eq!(parameters.redirect_uri, "https://app.example/cb");
        assert_eq!(
            parameters.code_challenge_method,
            Some(CodeChallengeMethod::S256)
        );
        assert_eq!(parameters.prompt.as_deref(), Some("login"));
        assert_eq!(parameters.max_age, Some(60));
    }

    #[test]
    fn response_type_and_redirect_uri_are_required() {
        for incomplete in [
            PushedAuthorizationRequestForm {
                response_type: None,
                ..form()
            },
            PushedAuthorizationRequestForm {
                redirect_uri: Some(String::new()),
                ..form()
            },
        ] {
            assert_eq!(
                oauth_error(authorization_parameters(incomplete).expect_err("refused")),
                "invalid_request"
            );
        }
    }

    #[test]
    fn a_request_uri_cannot_be_pushed() {
        let error = authorization_parameters(PushedAuthorizationRequestForm {
            request_uri: Some("urn:ietf:params:oauth:request_uri:abc".to_string()),
            ..form()
        })
        .expect_err("refused");

        assert_eq!(oauth_error(error), "invalid_request");
    }

    #[test]
    fn a_request_object_is_not_supported() {
        let error = authorization_parameters(PushedAuthorizationRequestForm {
            request: Some("a.b.c".to_string()),
            ..form()
        })
        .expect_err("refused");

        assert_eq!(oauth_error(error), "request_not_supported");
    }

    #[test]
    fn an_invalid_prompt_or_challenge_method_is_refused() {
        for invalid in [
            PushedAuthorizationRequestForm {
                prompt: Some("none login".to_string()),
                ..form()
            },
            PushedAuthorizationRequestForm {
                code_challenge_method: Some("S512".to_string()),
                ..form()
            },
        ] {
            assert_eq!(
                oauth_error(authorization_parameters(invalid).expect_err("refused")),
                "invalid_request"
            );
        }
    }
}
