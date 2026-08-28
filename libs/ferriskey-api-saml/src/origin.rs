use ferriskey_api_core::url::{public_base_url, root_scoped_base_url};

pub fn saml_public_base_url(
    configured_public_url: Option<&str>,
    root_path: &str,
) -> Option<String> {
    let configured = configured_public_url
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    Some(root_scoped_base_url(
        &public_base_url(Some(configured), ""),
        root_path,
    ))
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
    fn a_configured_public_url_is_what_the_signed_entity_id_is_built_from() {
        assert_eq!(
            saml_public_base_url(Some("https://auth.example.com"), ""),
            Some("https://auth.example.com".to_string())
        );
    }

    #[test]
    fn saml_refuses_to_run_without_a_stable_origin_rather_than_following_the_host_header() {
        assert_eq!(saml_public_base_url(None, ""), None);
        assert_eq!(saml_public_base_url(Some("   "), ""), None);
    }

    #[test]
    fn the_root_path_is_appended_so_the_continue_url_points_at_a_route_that_exists() {
        assert_eq!(
            saml_public_base_url(Some("https://auth.example.com"), "/api"),
            Some("https://auth.example.com/api".to_string())
        );
    }

    #[test]
    fn a_trailing_slash_on_the_configured_origin_never_doubles_up() {
        assert_eq!(
            saml_public_base_url(Some("https://auth.example.com/"), "/api"),
            Some("https://auth.example.com/api".to_string())
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
