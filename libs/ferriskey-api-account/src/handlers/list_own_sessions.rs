use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        list_query::{ListQuery, PaginationParams},
        paginated::Paginated,
        response::Response,
    },
    app_state::AppState,
    caller_session::CallerSession,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::session::entities::{SessionFilter, SessionSortField, UserSession};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct OwnSessionDto {
    pub id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub persistent: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    /// Whether this is the session the request was made with.
    pub current: bool,
}

impl OwnSessionDto {
    fn new(session: UserSession, current_session_id: Uuid) -> Self {
        Self {
            current: session.id == current_session_id,
            id: session.id,
            user_agent: session.user_agent,
            ip_address: session.ip_address,
            persistent: session.persistent,
            created_at: session.created_at,
            expires_at: session.expires_at,
            last_seen_at: session.last_seen_at,
        }
    }
}

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct OwnSessionListParams {}

#[utoipa::path(
    get,
    path = "/me/sessions",
    tag = "account",
    summary = "List the caller's own sessions",
    description = "Returns one page of the caller's live sessions in the realm, with the device (user agent), IP address, creation date and last use of each, and which one is the session the request was made with. Expired sessions are left out. Sorting on last_seen_at puts never-seen sessions last in ascending order and first in descending order. The user id is taken from the access token, never from the URL, and the token must be bound to a session.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        ("order_by" = inline(Option<SessionSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of sessions", body = Paginated<OwnSessionDto>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "The token is not bound to a session", body = ApiErrorResponse),
        (status = 403, description = "The caller is not a regular user", body = ApiErrorResponse),
        (status = 404, description = "No such realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn list_own_sessions(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(current_session_id): CallerSession,
    ListQuery(request): ListQuery<OwnSessionListParams, SessionSortField>,
) -> Result<Response<Paginated<OwnSessionDto>>, ApiError> {
    let page = state
        .service
        .list_own_sessions(
            identity,
            realm_name,
            request.map_filter(|OwnSessionListParams {}| SessionFilter::default()),
        )
        .await?;

    Ok(Response::OK(Paginated::from(page.map(|session| {
        OwnSessionDto::new(session, current_session_id)
    }))))
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use ferriskey_api_core::api_entities::list_query::parse_list_query;

    use super::*;

    fn session() -> UserSession {
        UserSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some("Mozilla/5.0".to_string()),
            Some("203.0.113.7".to_string()),
            Duration::hours(1),
            None,
        )
    }

    #[test]
    fn only_the_session_the_request_was_made_with_is_current() {
        let mine = session();
        let other = session();
        let mine_id = mine.id;

        assert!(OwnSessionDto::new(mine, mine_id).current);
        assert!(!OwnSessionDto::new(other, mine_id).current);
    }

    #[test]
    fn nothing_that_identifies_the_account_or_resumes_the_session_is_serialized() {
        let mut session = session();
        session.sso_token_hash = Some("secret-hash".to_string());
        let (user_id, realm_id) = (session.user_id, session.realm_id);

        let json = serde_json::to_string(&OwnSessionDto::new(session, Uuid::new_v4()))
            .expect("serializes");

        for hidden in [
            "secret-hash",
            "sso_token_hash",
            "user_id",
            "realm_id",
            &user_id.to_string(),
            &realm_id.to_string(),
        ] {
            assert!(!json.contains(hidden), "{hidden} leaked in {json}");
        }
    }

    #[test]
    fn the_listing_takes_no_filter_and_only_documented_sort_columns() {
        for query in [
            "user_id=3f1c2a54-0a3c-4c8e-8a53-6a6a1d4f9c10",
            "search=fox",
            "order_by=sso_token_hash",
            "order_by=user_agent",
        ] {
            assert!(
                parse_list_query::<OwnSessionListParams, SessionSortField>(query).is_err(),
                "{query}"
            );
        }

        assert!(
            parse_list_query::<OwnSessionListParams, SessionSortField>(
                "page=2&limit=10&order_by=last_seen_at&order=desc"
            )
            .is_ok()
        );
    }
}
