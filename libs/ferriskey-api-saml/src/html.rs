use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};

pub const HTML_CONTENT_TYPE: &str = "text/html; charset=utf-8";

pub fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            other => escaped.push(other),
        }
    }

    escaped
}

pub fn auto_submit_form(acs_url: &str, saml_response: &str, relay_state: Option<&str>) -> String {
    let mut inputs = hidden_input("SAMLResponse", saml_response);

    if let Some(relay_state) = relay_state {
        inputs.push_str(&hidden_input("RelayState", relay_state));
    }

    format!(
        concat!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">",
            "<title>Signing you in</title></head><body>",
            "<form method=\"post\" action=\"{}\">{}",
            "<noscript><p>Your browser did not submit this form automatically.</p>",
            "<button type=\"submit\">Continue</button></noscript>",
            "</form><script>document.forms[0].submit();</script>",
            "</body></html>",
        ),
        escape_html(acs_url),
        inputs,
    )
}

pub fn error_page(status: StatusCode, message: &'static str) -> Response {
    let body = format!(
        concat!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">",
            "<title>Single sign-on failed</title></head><body>",
            "<h1>Single sign-on failed</h1><p>{}</p>",
            "</body></html>",
        ),
        escape_html(message),
    );

    html_page(status, body)
}

pub fn html_page(status: StatusCode, body: String) -> Response {
    (
        status,
        [
            (CONTENT_TYPE, HTML_CONTENT_TYPE),
            (CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

fn hidden_input(name: &'static str, value: &str) -> String {
    format!(
        r#"<input type="hidden" name="{}" value="{}"/>"#,
        name,
        escape_html(value)
    )
}

#[cfg(test)]
mod tests {
    use super::{auto_submit_form, error_page, escape_html};

    use axum::http::StatusCode;
    use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};

    const ACS_URL: &str = "https://chat.example.com/omniauth/saml/callback?account_id=1";
    const SAML_RESPONSE: &str = "PHNhbWxwOlJlc3BvbnNlLz4=";

    #[test]
    fn every_markup_significant_character_is_escaped() {
        assert_eq!(escape_html(r#"& < > " '"#), "&amp; &lt; &gt; &quot; &#x27;");
    }

    #[test]
    fn text_without_markup_is_left_alone() {
        assert_eq!(
            escape_html("/app/accounts/1/dashboard"),
            "/app/accounts/1/dashboard"
        );
    }

    #[test]
    fn an_attacker_chosen_relay_state_cannot_break_out_of_the_attribute_it_sits_in() {
        let page = auto_submit_form(
            ACS_URL,
            SAML_RESPONSE,
            Some(r#""><script>alert(1)</script>"#),
        );

        assert!(
            !page.contains("<script>alert(1)</script>"),
            "the relay state was injected as markup: {page}"
        );
        assert!(
            page.contains(
                r#"name="RelayState" value="&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;""#
            ),
            "the relay state was not escaped inside its attribute: {page}"
        );
    }

    #[test]
    fn an_attacker_chosen_acs_url_cannot_break_out_of_the_action_attribute() {
        let page = auto_submit_form(
            r#"https://chat.example.com/cb"><script>alert(1)</script>"#,
            SAML_RESPONSE,
            None,
        );

        assert!(
            !page.contains("<script>alert(1)</script>"),
            "the acs url was injected as markup: {page}"
        );
        assert!(
            page.contains(
                r#"action="https://chat.example.com/cb&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;""#
            ),
            "the acs url was not escaped inside its attribute: {page}"
        );
    }

    #[test]
    fn an_ampersand_in_the_acs_url_query_string_is_escaped_without_being_lost() {
        let page = auto_submit_form(ACS_URL, SAML_RESPONSE, None);

        assert!(
            page.contains(
                r#"action="https://chat.example.com/omniauth/saml/callback?account_id=1""#
            ),
            "a plain acs url must survive verbatim: {page}"
        );
        assert!(
            auto_submit_form("https://sp.example.com/cb?a=1&b=2", SAML_RESPONSE, None)
                .contains(r#"action="https://sp.example.com/cb?a=1&amp;b=2""#),
            "the ampersand must be escaped as an entity"
        );
    }

    #[test]
    fn the_form_posts_the_response_to_the_assertion_consumer_service() {
        let page = auto_submit_form(ACS_URL, SAML_RESPONSE, Some("/app/accounts/1/dashboard"));

        assert!(page.contains(r#"<form method="post" action="https://chat.example.com/omniauth/saml/callback?account_id=1">"#));
        assert!(page.contains(
            r#"<input type="hidden" name="SAMLResponse" value="PHNhbWxwOlJlc3BvbnNlLz4="/>"#
        ));
        assert!(page.contains(
            r#"<input type="hidden" name="RelayState" value="/app/accounts/1/dashboard"/>"#
        ));
    }

    #[test]
    fn no_relay_state_input_is_emitted_when_the_service_provider_sent_none() {
        let page = auto_submit_form(ACS_URL, SAML_RESPONSE, None);

        assert!(
            !page.contains("RelayState"),
            "an absent relay state must not be echoed back as an empty one: {page}"
        );
    }

    #[test]
    fn an_empty_relay_state_is_still_carried_because_the_service_provider_asked_for_it() {
        let page = auto_submit_form(ACS_URL, SAML_RESPONSE, Some(""));

        assert!(page.contains(r#"<input type="hidden" name="RelayState" value=""/>"#));
    }

    #[test]
    fn the_flow_survives_with_javascript_disabled() {
        let page = auto_submit_form(ACS_URL, SAML_RESPONSE, None);

        assert!(
            page.contains("<noscript>") && page.contains(r#"<button type="submit">"#),
            "a browser without javascript must have something to click: {page}"
        );
    }

    #[test]
    fn the_page_is_served_as_html_and_never_cached() {
        let response = error_page(StatusCode::BAD_REQUEST, "This request was refused.");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(
            response
                .headers()
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("no-store")
        );
    }
}
