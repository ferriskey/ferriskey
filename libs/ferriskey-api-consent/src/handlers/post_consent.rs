use axum::extract::{Path, State};
use axum::http::{HeaderMap, header::SET_COOKIE};
use axum_cookie::CookieManager;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::sso_cookie;
use ferriskey_api_core::url::FullUrl;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct PostConsentRequest {
    #[validate(length(min = 1, message = "consent_token is required"))]
    pub consent_token: String,
    #[serde(default)]
    pub approved_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PostConsentResponse {
    pub redirect_url: String,
}

fn browser_session(cookie: &CookieManager) -> Option<uuid::Uuid> {
    cookie
        .get("FERRISKEY_SESSION")
        .and_then(|value| uuid::Uuid::parse_str(value.value()).ok())
}

#[utoipa::path(
    post,
    summary = "Submit a consent decision",
    path = "/auth/consent",
    tag = "consent",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = PostConsentRequest,
    responses(
        (status = 200, description = "Decision stored, redirect to the client", body = PostConsentResponse),
        (status = 404, description = "Unknown or expired consent token", body = ApiErrorResponse),
    )
)]

pub async fn post_consent(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<PostConsentRequest>,
) -> Result<(HeaderMap, Response<PostConsentResponse>), ApiError> {
    let decision = state
        .service
        .submit_consent_decision(
            &realm_name,
            &payload.consent_token,
            payload.approved_scopes,
            browser_session(&cookie),
        )
        .await
        .map_err(ApiError::from)?;

    let mut headers = HeaderMap::new();
    if let Some(secret) = decision.sso_cookie {
        headers.append(
            SET_COOKIE,
            sso_cookie::set(
                secret,
                decision.sso_session_max_age_secs,
                base_url.starts_with("https"),
            )?,
        );
    }

    Ok((
        headers,
        Response::OK(PostConsentResponse {
            redirect_url: decision.redirect_url,
        }),
    ))
}
