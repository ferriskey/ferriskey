use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::session::entities::{SessionFilter, SessionSortField, UserSession};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        list_query::{ListQuery, PaginationParams},
        paginated::Paginated,
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSessionDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub persistent: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

impl From<UserSession> for UserSessionDto {
    fn from(session: UserSession) -> Self {
        Self {
            id: session.id,
            user_id: session.user_id,
            realm_id: session.realm_id,
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
pub struct UserSessionListParams {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub persistent: Option<bool>,
}

impl From<UserSessionListParams> for SessionFilter {
    fn from(params: UserSessionListParams) -> Self {
        Self {
            ip_address: params.ip_address,
            user_agent: params.user_agent,
            persistent: params.persistent,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{user_id}/sessions",
    tag = "user",
    summary = "List the sessions of a user",
    description = "Returns one page of the user's sessions in the realm, expired ones included. ip_address and user_agent match case-insensitively anywhere in the value and never match a session where the value is unknown; persistent matches exactly. Filters combine with AND. Sorting on last_seen_at puts never-seen sessions last in ascending order and first in descending order. Requires ManageUsers, ManageRealm or ViewUsers, unless the caller is the user themselves. A user that is not in the realm is a 404.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
        PaginationParams,
        UserSessionListParams,
        ("order_by" = inline(Option<SessionSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of sessions", body = Paginated<UserSessionDto>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm or user not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn list_user_sessions(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<UserSessionListParams, SessionSortField>,
) -> Result<Response<Paginated<UserSessionDto>>, ApiError> {
    let page = state
        .service
        .list_user_sessions(
            identity,
            realm_name,
            user_id,
            request.map_filter(Into::into),
        )
        .await?;

    Ok(Response::OK(Paginated::from(
        page.map(UserSessionDto::from),
    )))
}

#[cfg(test)]
mod tests {
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<UserSessionListParams, SessionSortField>(
            "order_by=last_seen_at&order=asc&ip_address=10.0&user_agent=firefox&persistent=true",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, SessionSortField::LastSeenAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            SessionFilter::from(request.filter),
            SessionFilter {
                ip_address: Some("10.0".to_string()),
                user_agent: Some("firefox".to_string()),
                persistent: Some(true),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["last_seen_at", "expires_at", "created_at"] {
            assert!(
                parse_list_query::<UserSessionListParams, SessionSortField>(&format!(
                    "order_by={value}"
                ))
                .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_and_columns_are_refused() {
        for query in [
            "search=firefox",
            "sso_token_hash=x",
            "order_by=user_agent",
            "order_by=sso_token_hash",
            "persistent=maybe",
        ] {
            assert!(
                parse_list_query::<UserSessionListParams, SessionSortField>(query).is_err(),
                "{query}"
            );
        }
    }

    #[test]
    fn the_sso_token_hash_is_not_serialized() {
        let mut session = UserSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            chrono::Duration::hours(1),
            None,
        );
        session.sso_token_hash = Some("secret-hash".to_string());

        let json = serde_json::to_string(&UserSessionDto::from(session)).expect("serializes");

        assert!(!json.contains("secret-hash"), "{json}");
        assert!(!json.contains("sso_token_hash"), "{json}");
    }
}
