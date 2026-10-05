use std::fmt::Display;

use axum::{extract::FromRequestParts, http::request::Parts};
use ferriskey_core::domain::common::pagination::{
    MAX_PAGE_LIMIT, PageLimit, PageNumber, PageRequest, Sort, SortOrder,
};
use serde::{
    Deserialize,
    de::{DeserializeOwned, IntoDeserializer, value},
};
use utoipa::IntoParams;
use uuid::Uuid;

use super::api_error::{ApiError, ApiErrorBody};

pub struct ListQuery<F, S>(pub PageRequest<F, S>);

impl<F, S, St> FromRequestParts<St> for ListQuery<F, S>
where
    F: DeserializeOwned,
    S: DeserializeOwned + Default,
    St: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &St) -> Result<Self, Self::Rejection> {
        parse_list_query(parts.uri.query().unwrap_or_default()).map(ListQuery)
    }
}

pub fn parse_list_query<F, S>(query: &str) -> Result<PageRequest<F, S>, ApiError>
where
    F: DeserializeOwned,
    S: DeserializeOwned + Default,
{
    let pairs: Vec<(String, String)> =
        serde_urlencoded::from_str(query).map_err(|e| invalid("query", e))?;
    let mut page = PageNumber::default();
    let mut limit = PageLimit::default();
    let mut field = S::default();
    let mut order = SortOrder::default();
    let mut filters = Vec::with_capacity(pairs.len());

    for (key, value) in pairs {
        match key.as_str() {
            "page" => {
                page = value
                    .parse::<u32>()
                    .map_err(|e| invalid("page", e))
                    .and_then(|n| PageNumber::try_from(n).map_err(|e| invalid("page", e)))?
            }
            "limit" => {
                limit = value
                    .parse::<u32>()
                    .map_err(|e| invalid("limit", e))
                    .and_then(|n| PageLimit::try_from(n).map_err(|e| invalid("limit", e)))?
            }
            "order_by" => field = from_str(&value).map_err(|e| invalid("order_by", e))?,
            "order" => order = from_str(&value).map_err(|e| invalid("order", e))?,
            _ if value.is_empty() => {}
            _ => filters.push((key, value)),
        }
    }

    let filter = parse_filter(&filters)?;

    Ok(PageRequest {
        page,
        limit,
        sort: Sort { field, order },
        filter,
    })
}

fn parse_filter<F: DeserializeOwned>(filters: &[(String, String)]) -> Result<F, ApiError> {
    let encoded = serde_urlencoded::to_string(filters).map_err(|e| invalid("query", e))?;
    serde_urlencoded::from_str(&encoded).map_err(|e| {
        let param = filters
            .iter()
            .find(|pair| {
                serde_urlencoded::to_string([pair])
                    .ok()
                    .and_then(|alone| serde_urlencoded::from_str::<F>(&alone).ok())
                    .is_none()
            })
            .map_or("filter", |(key, _)| key.as_str());
        invalid(param, e)
    })
}

fn from_str<T: DeserializeOwned>(raw: &str) -> Result<T, value::Error> {
    T::deserialize(raw.into_deserializer())
}

pub fn parse_id_list(param: &str, raw: &str) -> Result<Vec<Uuid>, ApiError> {
    let ids = raw
        .split(',')
        .map(|part| Uuid::parse_str(part.trim()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| invalid(param, e))?;
    if ids.len() > MAX_PAGE_LIMIT as usize {
        return Err(invalid(param, format!("at most {MAX_PAGE_LIMIT} ids")));
    }
    Ok(ids)
}

fn invalid(param: &str, detail: impl Display) -> ApiError {
    ApiError::BadRequest(ApiErrorBody::new(
        format!("Invalid query parameter `{param}`: {detail}"),
        "invalid_query",
    ))
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    #[param(minimum = 1, default = 1)]
    pub page: Option<u32>,
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub limit: Option<u32>,
    #[param(inline)]
    pub order: Option<SortOrder>,
}

#[cfg(test)]
mod tests {
    use ferriskey_core::domain::common::pagination::SortOrder;
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Default, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Filter {
        username: Option<String>,
        enabled: Option<bool>,
    }

    #[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum SortField {
        Username,
        #[default]
        CreatedAt,
    }

    fn parse(query: &str) -> Result<PageRequest<Filter, SortField>, ApiError> {
        parse_list_query(query)
    }

    fn message(error: ApiError) -> String {
        match error {
            ApiError::BadRequest(body) => body.message.into_owned(),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[test]
    fn empty_query_gives_defaults() {
        let request = parse("").expect("defaults");
        assert_eq!(request.page.get(), 1);
        assert_eq!(request.limit.get(), 20);
        assert_eq!(request.sort.field, SortField::CreatedAt);
        assert_eq!(request.sort.order, SortOrder::Desc);
        assert_eq!(request.filter, Filter::default());
    }

    #[test]
    fn every_param_is_read() {
        let request = parse("page=3&limit=50&order_by=username&order=asc&username=jo&enabled=true")
            .expect("valid");
        assert_eq!(request.page.get(), 3);
        assert_eq!(request.limit.get(), 50);
        assert_eq!(request.sort.field, SortField::Username);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            request.filter,
            Filter {
                username: Some("jo".into()),
                enabled: Some(true)
            }
        );
    }

    #[test]
    fn encoded_values_are_decoded() {
        let request = parse("username=a%25b%20c").expect("valid");
        assert_eq!(request.filter.username.as_deref(), Some("a%b c"));
    }

    #[test]
    fn empty_filter_value_is_ignored() {
        let request = parse("username=&enabled=").expect("valid");
        assert_eq!(request.filter, Filter::default());
    }

    #[test]
    fn invalid_params_name_themselves() {
        for (query, param) in [
            ("limit=101", "limit"),
            ("limit=0", "limit"),
            ("limit=abc", "limit"),
            ("limit=-1", "limit"),
            ("page=0", "page"),
            ("page=x", "page"),
            ("order=up", "order"),
            ("order_by=secret", "order_by"),
            ("foo=bar", "foo"),
            ("enabled=maybe", "enabled"),
            ("username=a&username=b", "username"),
        ] {
            let error = parse(query).expect_err(query);
            let text = message(error);
            assert!(text.contains(param), "{query}: {text}");
        }
    }

    #[test]
    fn rejection_carries_the_invalid_query_reason() {
        match parse("limit=101").expect_err("too large") {
            ApiError::BadRequest(body) => assert_eq!(body.reason, Some("invalid_query")),
            other => panic!("{other:?}"),
        }
    }

    fn ids(count: usize) -> String {
        (0..count)
            .map(|_| uuid::Uuid::new_v4().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn id_lists_up_to_the_page_limit_are_parsed() {
        let first = uuid::Uuid::new_v4();
        let second = uuid::Uuid::new_v4();
        assert_eq!(
            parse_id_list("ids", &format!("{first}, {second}")).expect("two ids"),
            vec![first, second]
        );
        assert_eq!(
            parse_id_list("ids", &ids(100)).map(|ids| ids.len()).ok(),
            Some(100)
        );
    }

    #[test]
    fn invalid_id_lists_name_the_parameter() {
        for raw in [ids(101), "nope".to_string(), ",".to_string()] {
            match parse_id_list("role_ids", &raw) {
                Err(ApiError::BadRequest(body)) => {
                    assert!(body.message.contains("`role_ids`"), "{}", body.message);
                    assert_eq!(body.reason, Some("invalid_query"));
                }
                other => panic!("{raw}: {other:?}"),
            }
        }
    }
}
