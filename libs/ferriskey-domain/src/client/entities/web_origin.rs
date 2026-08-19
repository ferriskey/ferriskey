//! Web origins registered for a client, and the CORS decision they feed.
//!
//! An origin is not a URL. `https://app.example.com/callback` is a redirect URI;
//! the origin a browser sends in the `Origin` header is `https://app.example.com`
//! — scheme, host, and port only, serialized per the URL Standard. Storing one
//! where the other is expected produces a value that never matches any preflight,
//! silently, which is why [`Origin`] cannot be built by any route that skips
//! validation — `serde` included, via `try_from`.
//!
//! Normalization follows the browser's own serialization, since that is what the
//! comparison is against: lowercased scheme and host, default port omitted
//! (`:443` for https, `:80` for http), nothing after the authority.
//!
//! `*` is rejected rather than accepted-and-ignored. The Fetch standard forbids
//! `Access-Control-Allow-Origin: *` alongside `Access-Control-Allow-Credentials:
//! true`, and FerrisKey sets the latter, so a stored `*` could never be honoured.
//! Refusing it at the boundary turns a silently dead configuration into an error
//! the administrator sees while typing it.
//!
//! Two doors lead in, and they validate differently on purpose.
//! [`Origin::try_from`] takes what an administrator typed and demands it already
//! *be* an origin — credentials, path, query or fragment are all refusals.
//! [`Origin::from_url`] takes a URL that legitimately has more than an origin, a
//! redirect URI, and extracts the origin a browser would send for it; there,
//! credentials and path belong to the URI and are discarded rather than refused.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidOrigin {
    #[error("`{0}` is not an absolute URL")]
    NotAUrl(String),

    #[error("origin scheme must be http or https, got `{0}`")]
    UnsupportedScheme(String),

    #[error("an origin carries no path, query or fragment")]
    NotAnOrigin,

    #[error("an origin carries no credentials")]
    HasCredentials,

    #[error("`*` cannot be allowed while credentials are allowed")]
    Wildcard,
}

impl From<InvalidOrigin> for CoreError {
    fn from(error: InvalidOrigin) -> Self {
        CoreError::InvalidWebOrigin(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Origin(String);

impl Origin {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The origin a browser would send for `url`, discarding everything the
    /// origin does not carry.
    ///
    /// Only http and https have a serializable origin here; every other scheme is
    /// refused rather than allowed to serialize to the opaque `"null"`.
    pub(crate) fn from_url(url: &Url) -> Result<Self, InvalidOrigin> {
        match url.scheme() {
            "http" | "https" => Ok(Self(url.origin().ascii_serialization())),
            other => Err(InvalidOrigin::UnsupportedScheme(other.to_string())),
        }
    }
}

impl TryFrom<&str> for Origin {
    type Error = InvalidOrigin;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();

        if value == "*" {
            return Err(InvalidOrigin::Wildcard);
        }

        let url = Url::parse(value).map_err(|_| InvalidOrigin::NotAUrl(value.to_string()))?;
        let origin = Self::from_url(&url)?;

        if !url.username().is_empty() || url.password().is_some() {
            return Err(InvalidOrigin::HasCredentials);
        }

        if !matches!(url.path(), "" | "/") || url.query().is_some() || url.fragment().is_some() {
            return Err(InvalidOrigin::NotAnOrigin);
        }

        Ok(origin)
    }
}

impl TryFrom<String> for Origin {
    type Error = InvalidOrigin;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl FromStr for Origin {
    type Err = InvalidOrigin;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl From<Origin> for String {
    fn from(origin: Origin) -> Self {
        origin.0
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub enum WebOriginValue {
    DerivedFromRedirectUris,
    Explicit(Origin),
}

impl WebOriginValue {
    /// Keycloak spells "derive my origins from my redirect URIs" as `+`, and
    /// administrators migrating from it type that. Kept as a stored value rather
    /// than a boolean on the client so it stays visible in the list and auditable.
    pub const DERIVED_SENTINEL: &'static str = "+";
}

impl FromStr for WebOriginValue {
    type Err = InvalidOrigin;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == Self::DERIVED_SENTINEL {
            return Ok(Self::DerivedFromRedirectUris);
        }

        Origin::try_from(s).map(Self::Explicit)
    }
}

impl TryFrom<String> for WebOriginValue {
    type Error = InvalidOrigin;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<WebOriginValue> for String {
    fn from(value: WebOriginValue) -> Self {
        value.to_string()
    }
}

impl fmt::Display for WebOriginValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DerivedFromRedirectUris => f.write_str(Self::DERIVED_SENTINEL),
            Self::Explicit(origin) => origin.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, ToSchema)]
pub struct WebOrigin {
    pub id: Uuid,
    pub client_id: Uuid,
    pub value: WebOriginValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WebOrigin {
    pub fn new(client_id: Uuid, value: WebOriginValue) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            client_id,
            value,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(value: &str) -> Origin {
        Origin::try_from(value).expect("test fixture must be a valid origin")
    }

    #[test]
    fn keeps_scheme_host_and_non_default_port() {
        assert_eq!(
            origin("https://app.example.com:8443").as_str(),
            "https://app.example.com:8443"
        );
    }

    #[test]
    fn drops_the_default_https_port() {
        assert_eq!(
            origin("https://app.example.com:443").as_str(),
            "https://app.example.com"
        );
    }

    #[test]
    fn drops_the_default_http_port() {
        assert_eq!(
            origin("http://app.example.com:80").as_str(),
            "http://app.example.com"
        );
    }

    #[test]
    fn lowercases_scheme_and_host() {
        assert_eq!(
            origin("HTTPS://App.Example.COM").as_str(),
            "https://app.example.com"
        );
    }

    #[test]
    fn drops_the_root_trailing_slash() {
        assert_eq!(
            origin("https://app.example.com/").as_str(),
            "https://app.example.com"
        );
    }

    #[test]
    fn rejects_a_path() {
        assert_eq!(
            Origin::try_from("https://app.example.com/callback"),
            Err(InvalidOrigin::NotAnOrigin)
        );
    }

    #[test]
    fn rejects_a_query() {
        assert_eq!(
            Origin::try_from("https://app.example.com/?next=1"),
            Err(InvalidOrigin::NotAnOrigin)
        );
    }

    #[test]
    fn rejects_a_fragment() {
        assert_eq!(
            Origin::try_from("https://app.example.com/#top"),
            Err(InvalidOrigin::NotAnOrigin)
        );
    }

    #[test]
    fn rejects_credentials_in_the_authority() {
        assert_eq!(
            Origin::try_from("https://user:secret@app.example.com"),
            Err(InvalidOrigin::HasCredentials)
        );
    }

    #[test]
    fn rejects_a_password_carrying_no_username() {
        assert_eq!(
            Origin::try_from("https://:secret@app.example.com"),
            Err(InvalidOrigin::HasCredentials)
        );
    }

    #[test]
    fn rejects_the_wildcard() {
        assert_eq!(Origin::try_from("*"), Err(InvalidOrigin::Wildcard));
    }

    #[test]
    fn rejects_the_wildcard_padded_with_whitespace() {
        assert_eq!(Origin::try_from("  *  "), Err(InvalidOrigin::Wildcard));
    }

    #[test]
    fn rejects_a_scheme_that_is_otherwise_shaped_like_an_origin() {
        assert_eq!(
            Origin::try_from("ftp://files.example.com"),
            Err(InvalidOrigin::UnsupportedScheme("ftp".to_string()))
        );
    }

    #[test]
    fn rejects_a_non_http_scheme() {
        assert_eq!(
            Origin::try_from("file:///etc/passwd"),
            Err(InvalidOrigin::UnsupportedScheme("file".to_string()))
        );
    }

    #[test]
    fn rejects_the_null_origin() {
        assert_eq!(
            Origin::try_from("null"),
            Err(InvalidOrigin::NotAUrl("null".to_string()))
        );
    }

    #[test]
    fn rejects_an_empty_value() {
        assert_eq!(
            Origin::try_from(""),
            Err(InvalidOrigin::NotAUrl(String::new()))
        );
    }

    #[test]
    fn deserializing_cannot_smuggle_a_wildcard_past_validation() {
        assert!(serde_json::from_str::<Origin>("\"*\"").is_err());
    }

    #[test]
    fn deserializing_cannot_smuggle_an_arbitrary_string_past_validation() {
        assert!(serde_json::from_str::<Origin>("\"not an origin at all\"").is_err());
    }

    #[test]
    fn deserializing_normalizes_exactly_as_parsing_does() {
        assert_eq!(
            serde_json::from_str::<Origin>("\"HTTPS://App.Example.COM:443\"")
                .expect("a valid origin must deserialize"),
            origin("https://app.example.com")
        );
    }

    #[test]
    fn the_sentinel_parses_to_the_derived_variant() {
        assert_eq!(
            WebOriginValue::from_str("+"),
            Ok(WebOriginValue::DerivedFromRedirectUris)
        );
    }

    #[test]
    fn the_derived_variant_renders_back_to_the_sentinel() {
        assert_eq!(WebOriginValue::DerivedFromRedirectUris.to_string(), "+");
    }

    #[test]
    fn an_explicit_value_round_trips_through_its_stored_form() {
        let stored = WebOriginValue::Explicit(origin("https://app.example.com:443")).to_string();

        assert_eq!(stored, "https://app.example.com");
        assert_eq!(
            WebOriginValue::from_str(&stored),
            Ok(WebOriginValue::Explicit(origin("https://app.example.com")))
        );
    }

    #[test]
    fn an_invalid_value_does_not_parse() {
        assert_eq!(
            WebOriginValue::from_str("https://app.example.com/callback"),
            Err(InvalidOrigin::NotAnOrigin)
        );
    }

    #[test]
    fn a_value_serializes_to_the_same_form_it_is_stored_as() {
        let explicit = WebOriginValue::Explicit(origin("https://app.example.com"));

        assert_eq!(
            serde_json::to_string(&explicit).expect("serialization is infallible for a String"),
            "\"https://app.example.com\""
        );
        assert_eq!(
            serde_json::to_string(&WebOriginValue::DerivedFromRedirectUris)
                .expect("serialization is infallible for a String"),
            "\"+\""
        );
    }

    #[test]
    fn deserializing_a_value_rejects_what_parsing_rejects() {
        assert!(
            serde_json::from_str::<WebOriginValue>("\"https://app.example.com/callback\"").is_err()
        );
    }

    #[test]
    fn a_rejected_origin_reaches_the_caller_as_a_core_error_carrying_the_reason() {
        let error = Origin::try_from("https://app.example.com/callback")
            .expect_err("a path makes this an invalid origin");
        let reason = error.to_string();

        let converted = CoreError::from(error);

        assert!(matches!(&converted, CoreError::InvalidWebOrigin(_)));
        assert!(converted.to_string().contains(&reason));
    }
}
