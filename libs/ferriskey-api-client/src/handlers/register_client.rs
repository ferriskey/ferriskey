use std::net::IpAddr;

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::request_context::RequestContext;
use ferriskey_core::domain::client_registration::entities::{
    ClientRegistrationRequest, ClientRegistrationResponse, RegistrationError,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct RegistrationErrorResponse {
    pub error: &'static str,
    pub error_description: String,
}

fn error_response(
    status: StatusCode,
    error: &'static str,
    description: impl Into<String>,
) -> Response {
    (
        status,
        Json(RegistrationErrorResponse {
            error,
            error_description: description.into(),
        }),
    )
        .into_response()
}

fn render(error: RegistrationError) -> Response {
    match error {
        RegistrationError::Disabled => {
            error_response(StatusCode::NOT_FOUND, "not_found", "not found")
        }
        RegistrationError::RateLimited => {
            let mut response = error_response(
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_requests",
                "too many registration requests, retry later",
            );
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
            response
        }
        RegistrationError::InvalidRedirectUri(description) => {
            error_response(StatusCode::BAD_REQUEST, "invalid_redirect_uri", description)
        }
        RegistrationError::InvalidClientMetadata(description) => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_client_metadata",
            description,
        ),
        RegistrationError::Internal => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "the client could not be registered",
        ),
    }
}

#[utoipa::path(
    post,
    path = "/register",
    summary = "Register a client dynamically (RFC 7591)",
    description = "Anonymous dynamic client registration. Available only when the realm enables it, and rate limited per IP address.",
    responses(
        (status = 201, body = ClientRegistrationResponse, description = "Client registered"),
        (status = 400, body = RegistrationErrorResponse, description = "invalid_redirect_uri or invalid_client_metadata"),
        (status = 404, body = RegistrationErrorResponse, description = "Dynamic registration is disabled for the realm"),
        (status = 429, body = RegistrationErrorResponse, description = "Rate limit exceeded"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client",
    request_body = ClientRegistrationRequest,
)]
pub async fn register_client(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    context: RequestContext,
    body: Bytes,
) -> Response {
    let source_ip = context
        .ip_address
        .as_deref()
        .and_then(|raw| raw.parse::<IpAddr>().ok());

    let request: ClientRegistrationRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(_) => {
            return render(RegistrationError::InvalidClientMetadata(
                "the body is not a valid client metadata document",
            ));
        }
    };

    match state
        .service
        .register_client(&realm_name, source_ip, request)
        .await
    {
        Ok(registered) => (StatusCode::CREATED, Json(registered)).into_response(),
        Err(error) => render(error),
    }
}
