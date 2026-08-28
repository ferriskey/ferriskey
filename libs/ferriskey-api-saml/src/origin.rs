use ferriskey_api_core::url::{public_base_url, root_scoped_base_url};

pub fn saml_public_base_url(
    configured_public_url: Option<&str>,
    request_base_url: &str,
    root_path: &str,
) -> String {
    root_scoped_base_url(
        &public_base_url(configured_public_url, request_base_url),
        root_path,
    )
}

pub fn webapp_login_url(webapp_url: &str, realm_name: &str, login_url: &str) -> String {
    format!(
        "{}/realms/{}/authentication/login{}",
        webapp_url.trim_end_matches('/'),
        realm_name,
        login_url
    )
}

#[cfg(test)]
mod tests {
    use super::{saml_public_base_url, webapp_login_url};

    #[test]
    fn a_configured_public_url_wins_so_the_signed_entity_id_never_follows_the_host_header() {
        assert_eq!(
            saml_public_base_url(
                Some("https://auth.example.com"),
                "https://node-7.internal:3333",
                ""
            ),
            "https://auth.example.com"
        );
    }

    #[test]
    fn the_request_origin_is_used_when_nothing_is_configured() {
        assert_eq!(
            saml_public_base_url(None, "https://node-7.internal:3333", ""),
            "https://node-7.internal:3333"
        );
    }

    #[test]
    fn the_root_path_is_appended_so_the_continue_url_points_at_a_route_that_exists() {
        assert_eq!(
            saml_public_base_url(Some("https://auth.example.com"), "http://ignored", "/api"),
            "https://auth.example.com/api"
        );
        assert_eq!(
            saml_public_base_url(None, "https://auth.example.com", "/api"),
            "https://auth.example.com/api"
        );
    }

    #[test]
    fn a_trailing_slash_on_the_configured_origin_never_doubles_up() {
        assert_eq!(
            saml_public_base_url(Some("https://auth.example.com/"), "http://ignored", "/api"),
            "https://auth.example.com/api"
        );
    }

    #[test]
    fn the_login_url_joins_the_webapp_origin_without_doubling_the_separator() {
        assert_eq!(
            webapp_login_url(
                "https://login.example.com/",
                "demo",
                "?client_id=sp&redirect_uri=https://auth.example.com/realms/demo/protocol/saml/continue&state=relay"
            ),
            "https://login.example.com/realms/demo/authentication/login?client_id=sp&redirect_uri=https://auth.example.com/realms/demo/protocol/saml/continue&state=relay"
        );
    }
}
