use std::collections::HashSet;

use url::Url;

use crate::client::entities::web_origin::{Origin, WebOriginValue};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientOriginSources {
    pub web_origins: Vec<WebOriginValue>,
    pub enabled_redirect_uris: Vec<String>,
}

pub fn origin_of_redirect_uri(redirect_uri: &str) -> Option<Origin> {
    let url = Url::parse(redirect_uri).ok()?;

    Origin::from_url(&url).ok()
}

pub fn resolve_allowed_origins(sources: &[ClientOriginSources]) -> HashSet<Origin> {
    let mut allowed = HashSet::new();

    for client in sources {
        for value in &client.web_origins {
            match value {
                WebOriginValue::Explicit(origin) => {
                    allowed.insert(origin.clone());
                }
                WebOriginValue::DerivedFromRedirectUris => allowed.extend(
                    client
                        .enabled_redirect_uris
                        .iter()
                        .filter_map(|uri| origin_of_redirect_uri(uri)),
                ),
            }
        }
    }

    allowed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(value: &str) -> Origin {
        Origin::try_from(value).expect("test fixture must be a valid origin")
    }

    fn explicit(value: &str) -> WebOriginValue {
        WebOriginValue::Explicit(origin(value))
    }

    #[test]
    fn derives_the_origin_of_a_literal_redirect_uri() {
        assert_eq!(
            origin_of_redirect_uri("https://app.example.com/callback"),
            Some(origin("https://app.example.com"))
        );
    }

    #[test]
    fn derives_through_a_non_default_port() {
        assert_eq!(
            origin_of_redirect_uri("http://localhost:5555/callback"),
            Some(origin("http://localhost:5555"))
        );
    }

    #[test]
    fn derives_past_credentials_the_origin_itself_would_refuse() {
        assert_eq!(
            origin_of_redirect_uri("https://user:secret@app.example.com/callback"),
            Some(origin("https://app.example.com"))
        );
        assert!(Origin::try_from("https://user:secret@app.example.com").is_err());
    }

    #[test]
    fn does_not_derive_from_an_anchored_regex_redirect_uri() {
        assert_eq!(
            origin_of_redirect_uri(r"^https://app\.example\.com/.*$"),
            None
        );
    }

    #[test]
    fn does_not_derive_from_a_wildcard_shaped_redirect_uri() {
        assert_eq!(
            origin_of_redirect_uri("https://*.example.com/callback"),
            None
        );
    }

    #[test]
    fn does_not_derive_from_a_malformed_redirect_uri() {
        assert_eq!(origin_of_redirect_uri("not a uri"), None);
    }

    #[test]
    fn does_not_derive_from_a_non_http_redirect_uri() {
        assert_eq!(origin_of_redirect_uri("myapp://callback"), None);
    }

    #[test]
    fn resolves_an_explicit_origin() {
        let sources = vec![ClientOriginSources {
            web_origins: vec![explicit("https://app.example.com")],
            enabled_redirect_uris: vec![],
        }];

        assert_eq!(
            resolve_allowed_origins(&sources),
            HashSet::from([origin("https://app.example.com")])
        );
    }

    #[test]
    fn resolves_nothing_for_a_realm_without_clients() {
        assert_eq!(resolve_allowed_origins(&[]), HashSet::new());
    }

    #[test]
    fn the_sentinel_expands_the_redirect_uris_of_its_own_client() {
        let sources = vec![ClientOriginSources {
            web_origins: vec![WebOriginValue::DerivedFromRedirectUris],
            enabled_redirect_uris: vec![
                "https://app.example.com/callback".to_string(),
                "https://app.example.com/silent-renew".to_string(),
                "https://admin.example.com/callback".to_string(),
            ],
        }];

        assert_eq!(
            resolve_allowed_origins(&sources),
            HashSet::from([
                origin("https://app.example.com"),
                origin("https://admin.example.com"),
            ])
        );
    }

    #[test]
    fn the_sentinel_does_not_expand_another_clients_redirect_uris() {
        let sources = vec![
            ClientOriginSources {
                web_origins: vec![WebOriginValue::DerivedFromRedirectUris],
                enabled_redirect_uris: vec![],
            },
            ClientOriginSources {
                web_origins: vec![],
                enabled_redirect_uris: vec!["https://other.example.com/callback".to_string()],
            },
        ];

        assert_eq!(resolve_allowed_origins(&sources), HashSet::new());
    }

    #[test]
    fn the_sentinel_skips_regex_redirect_uris() {
        let sources = vec![ClientOriginSources {
            web_origins: vec![WebOriginValue::DerivedFromRedirectUris],
            enabled_redirect_uris: vec![
                r"^https://tenant-.*\.example\.com/callback$".to_string(),
                "https://app.example.com/callback".to_string(),
            ],
        }];

        assert_eq!(
            resolve_allowed_origins(&sources),
            HashSet::from([origin("https://app.example.com")])
        );
    }

    #[test]
    fn redirect_uris_alone_grant_nothing_without_the_sentinel() {
        let sources = vec![ClientOriginSources {
            web_origins: vec![],
            enabled_redirect_uris: vec!["https://app.example.com/callback".to_string()],
        }];

        assert_eq!(resolve_allowed_origins(&sources), HashSet::new());
    }

    #[test]
    fn one_client_can_combine_an_explicit_origin_and_the_sentinel() {
        let sources = vec![ClientOriginSources {
            web_origins: vec![
                explicit("https://static.example.com"),
                WebOriginValue::DerivedFromRedirectUris,
            ],
            enabled_redirect_uris: vec!["https://app.example.com/callback".to_string()],
        }];

        assert_eq!(
            resolve_allowed_origins(&sources),
            HashSet::from([
                origin("https://static.example.com"),
                origin("https://app.example.com"),
            ])
        );
    }

    #[test]
    fn unions_the_origins_of_every_client_in_the_realm() {
        let sources = vec![
            ClientOriginSources {
                web_origins: vec![explicit("https://app.example.com")],
                enabled_redirect_uris: vec![],
            },
            ClientOriginSources {
                web_origins: vec![explicit("https://admin.example.com")],
                enabled_redirect_uris: vec![],
            },
        ];

        assert_eq!(
            resolve_allowed_origins(&sources),
            HashSet::from([
                origin("https://app.example.com"),
                origin("https://admin.example.com"),
            ])
        );
    }
}
