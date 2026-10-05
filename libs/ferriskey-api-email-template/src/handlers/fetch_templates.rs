use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    common::pagination::{DateRange, PageRequest},
    email_template::{
        entities::{EmailTemplate, EmailTemplateFilter, EmailTemplateSortField, EmailType},
        ports::EmailTemplateService,
    },
};
use serde::Deserialize;
use utoipa::IntoParams;

use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    list_query::{ListQuery, PaginationParams},
    paginated::Paginated,
    response::Response,
};
use ferriskey_api_core::app_state::AppState;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct EmailTemplateListParams {
    pub search: Option<String>,
    pub name: Option<String>,
    #[param(inline)]
    pub email_type: Option<EmailType>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
}

impl From<EmailTemplateListParams> for EmailTemplateFilter {
    fn from(params: EmailTemplateListParams) -> Self {
        Self {
            search: params.search,
            name: params.name,
            email_type: params.email_type,
            created: DateRange::new(params.created_from, params.created_to),
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "email-template",
    summary = "Fetch email templates",
    description = "Returns one page of the realm's email templates, each with its structure and mjml. search and the name filter match case-insensitively anywhere in the name; email_type matches exactly; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        PaginationParams,
        EmailTemplateListParams,
        ("order_by" = inline(Option<EmailTemplateSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of email templates", body = Paginated<EmailTemplate>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn fetch_templates(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<EmailTemplateListParams, EmailTemplateSortField>,
) -> Result<Response<Paginated<EmailTemplate>>, ApiError> {
    let request = PageRequest {
        filter: EmailTemplateFilter::from(request.filter),
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_templates(identity, realm_name, request)
        .await?;

    Ok(Response::OK(Paginated::from(page)))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<EmailTemplateListParams, EmailTemplateSortField>(
            "order_by=email_type&order=asc&search=wel&name=welcome&email_type=magic_link&created_from=2026-01-01T00:00:00Z&created_to=2026-02-01T00:00:00%2B02:00",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, EmailTemplateSortField::EmailType);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            EmailTemplateFilter::from(request.filter),
            EmailTemplateFilter {
                search: Some("wel".to_string()),
                name: Some("welcome".to_string()),
                email_type: Some(EmailType::MagicLink),
                created: DateRange::new(
                    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                    Utc.with_ymd_and_hms(2026, 1, 31, 22, 0, 0).single(),
                ),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_and_email_type_parses() {
        for value in ["name", "email_type", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<EmailTemplateListParams, EmailTemplateSortField>(&format!(
                    "order_by={value}"
                ))
                .is_ok(),
                "{value}"
            );
        }
        for value in ["reset_password", "magic_link", "email_verification"] {
            assert!(
                parse_list_query::<EmailTemplateListParams, EmailTemplateSortField>(&format!(
                    "email_type={value}"
                ))
                .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_columns_and_email_types_are_refused() {
        for query in [
            "realm_id=x",
            "mjml=x",
            "structure=x",
            "order_by=mjml",
            "order_by=structure",
            "email_type=welcome",
            "search=a&search=b",
            "created_from=2026-10-05",
            "created_to=2026-10-05",
        ] {
            assert!(
                parse_list_query::<EmailTemplateListParams, EmailTemplateSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}
