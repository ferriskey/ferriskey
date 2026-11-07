use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header::SET_COOKIE};
use axum::response::IntoResponse;
use axum_cookie::CookieManager;
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::authentication::AuthenticateResponse;
use ferriskey_api_core::request_context::RequestContext;
use ferriskey_api_core::url::{FullUrl, WebappUrl};
use ferriskey_core::domain::authentication::entities::AuthenticateOutput;
use ferriskey_core::domain::authentication::ports::{
    AccountSelectionService, ListAccountsInput, SelectAccountInput, SelectAccountOutcome,
    SessionOrigin,
};
use ferriskey_core::domain::device_session::entities::AccountEntry;
use ferriskey_core::domain::session::entities::SessionState;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::device_cookie::DEVICE_COOKIE;
use crate::sso_cookie::SSO_SESSION_COOKIE;

const AUTH_SESSION_COOKIE: &str = "FERRISKEY_SESSION";

#[derive(Debug, Clone, Copy, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountEntryState {
    Active,
    SoftExpired,
    Expired,
}

impl From<SessionState> for AccountEntryState {
    fn from(state: SessionState) -> Self {
        match state {
            SessionState::Active => Self::Active,
            SessionState::SoftExpired => Self::SoftExpired,
            SessionState::Expired => Self::Expired,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountEntryDto {
    pub session_id: Uuid,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_email: Option<String>,
    pub state: AccountEntryState,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_seen_at: Option<String>,
    pub is_active: bool,
    pub requires_reauthentication: bool,
}

impl From<AccountEntry> for AccountEntryDto {
    fn from(entry: AccountEntry) -> Self {
        Self {
            session_id: entry.session_id,
            display_name: entry.display_name,
            masked_email: entry.masked_email,
            state: entry.state.into(),
            last_seen_at: entry
                .last_seen_at
                .map(|seen| seen.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()),
            is_active: entry.is_active,
            requires_reauthentication: entry.requires_reauthentication,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountListResponse {
    pub data: Vec<AccountEntryDto>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListAccountsParams {
    pub client_id: String,
    #[serde(default)]
    pub max_age: Option<i64>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SelectAccountParams {
    #[serde(default)]
    pub max_age: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SelectAccountRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SelectAccountStatus {
    RequiresCredentials,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SelectAccountRequiresCredentials {
    pub status: SelectAccountStatus,
    pub login_hint: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(untagged)]
pub enum SelectAccountResponse {
    Resumed(AuthenticateResponse),
    RequiresCredentials(SelectAccountRequiresCredentials),
}

fn resumed_response(
    output: AuthenticateOutput,
    is_secure: bool,
) -> Result<axum::response::Response, ApiError> {
    let sso_session = output
        .sso_cookie
        .clone()
        .map(|secret| (secret, output.sso_session_max_age_secs));
    let body: AuthenticateResponse = output.into();

    let mut headers = HeaderMap::new();
    if let Some((secret, max_age_secs)) = sso_session {
        headers.append(
            SET_COOKIE,
            crate::sso_cookie::set(secret, max_age_secs, is_secure)?,
        );
    }

    Ok((
        StatusCode::OK,
        headers,
        Json(SelectAccountResponse::Resumed(body)),
    )
        .into_response())
}

fn cookie_value(cookie: &CookieManager, name: &str) -> Option<String> {
    cookie
        .get(name)
        .map(|c| c.value().trim().to_string())
        .filter(|value| !value.is_empty())
}

#[utoipa::path(
    get,
    path = "/login-actions/accounts",
    tag = "auth",
    summary = "List the accounts remembered by this browser",
    description = "Returns the accounts the browser previously signed in with, derived only from the FERRISKEY_DEVICE cookie. The list is empty, with a 200, when the cookie is absent, unknown, issued for another realm, or the realm has the account chooser disabled.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ListAccountsParams,
    ),
    responses(
        (status = 200, description = "Remembered accounts", body = AccountListResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn list_accounts(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    cookie: CookieManager,
    Query(params): Query<ListAccountsParams>,
) -> Result<Json<AccountListResponse>, ApiError> {
    let entries = state
        .service
        .list_accounts(ListAccountsInput {
            realm_name,
            device_token: cookie_value(&cookie, DEVICE_COOKIE),
            active_sso_token: cookie_value(&cookie, SSO_SESSION_COOKIE),
            max_age: params.max_age,
            client_id: params.client_id,
            session_code: cookie_value(&cookie, AUTH_SESSION_COOKIE)
                .and_then(|code| Uuid::parse_str(&code).ok()),
        })
        .await?;

    Ok(Json(AccountListResponse {
        data: entries.into_iter().map(AccountEntryDto::from).collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/login-actions/accounts/select",
    tag = "auth",
    summary = "Continue the authorization request as a remembered account",
    description = "Resumes the chosen account's session without credentials and answers like the login endpoint does, with the redirect target in `url` and the FERRISKEY_SSO cookie set. When the session cannot be resumed silently the response is 200 with status requires_credentials and a login_hint, and the credential form re-authenticates into that session.",
    request_body = SelectAccountRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        SelectAccountParams,
    ),
    responses(
        (status = 200, description = "The session was resumed (same shape as the login endpoint, sets the FERRISKEY_SSO cookie), or credentials are required", body = SelectAccountResponse),
        (status = 401, description = "Missing session cookie", body = ApiErrorResponse),
        (status = 404, description = "The session is not attached to this device", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn select_account(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    WebappUrl(webapp_url): WebappUrl,
    context: RequestContext,
    cookie: CookieManager,
    Query(params): Query<SelectAccountParams>,
    Json(payload): Json<SelectAccountRequest>,
) -> Result<axum::response::Response, ApiError> {
    let session_code = cookie_value(&cookie, AUTH_SESSION_COOKIE)
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".into()))?;
    let session_code = Uuid::parse_str(&session_code)
        .map_err(|_| ApiError::BadRequest("Invalid session code in cookie".into()))?;

    let outcome = state
        .service
        .select_account(SelectAccountInput {
            realm_name,
            device_token: cookie_value(&cookie, DEVICE_COOKIE),
            session_code,
            session_id: payload.session_id,
            max_age: params.max_age,
            webapp_base_url: webapp_url,
            origin: SessionOrigin {
                user_agent: context.user_agent,
                ip_address: context.ip_address,
            },
        })
        .await?;

    match outcome {
        SelectAccountOutcome::Resumed(output) => {
            resumed_response(*output, base_url.starts_with("https://"))
        }
        SelectAccountOutcome::RequiresCredentials { login_hint } => Ok((
            StatusCode::OK,
            Json(SelectAccountResponse::RequiresCredentials(
                SelectAccountRequiresCredentials {
                    status: SelectAccountStatus::RequiresCredentials,
                    login_hint,
                },
            )),
        )
            .into_response()),
    }
}

#[utoipa::path(
    delete,
    path = "/login-actions/accounts/{session_id}",
    tag = "auth",
    summary = "Remove an account from this browser",
    description = "Detaches the session from the device and stops it from being resumable from this browser. The session row and its tokens are left untouched.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("session_id" = Uuid, Path, description = "Session ID of the account entry"),
    ),
    responses(
        (status = 204, description = "Account removed from this device"),
        (status = 404, description = "The session is not attached to this device", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn remove_account(
    Path((realm_name, session_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    cookie: CookieManager,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .detach_account(realm_name, cookie_value(&cookie, DEVICE_COOKIE), session_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(masked_email: Option<&str>) -> AccountEntry {
        AccountEntry {
            session_id: Uuid::new_v4(),
            display_name: "Alice Doe".into(),
            masked_email: masked_email.map(str::to_owned),
            state: SessionState::Active,
            last_seen_at: None,
            is_active: true,
            requires_reauthentication: false,
        }
    }

    #[test]
    fn a_hidden_email_is_omitted_not_null() {
        let json = serde_json::to_value(AccountEntryDto::from(entry(None))).expect("serializes");

        assert!(json.get("masked_email").is_none());
    }

    #[test]
    fn an_entry_never_exposes_a_user_id() {
        let json = serde_json::to_value(AccountEntryDto::from(entry(Some("a••••@x.io"))))
            .expect("serializes");

        assert!(json.get("user_id").is_none());
        assert_eq!(json["masked_email"], "a••••@x.io");
        assert_eq!(json["state"], "active");
    }

    #[test]
    fn the_refusal_body_has_the_contract_shape() {
        let json = serde_json::to_value(SelectAccountRequiresCredentials {
            status: SelectAccountStatus::RequiresCredentials,
            login_hint: "alice".into(),
        })
        .expect("serializes");

        assert_eq!(
            json,
            serde_json::json!({ "status": "requires_credentials", "login_hint": "alice" })
        );
    }
}
