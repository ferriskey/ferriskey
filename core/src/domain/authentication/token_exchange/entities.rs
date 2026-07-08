//! Domain entities for OAuth 2.0 Token Exchange (RFC 8693).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

/// Token type identifiers defined by RFC 8693 §3, serialized as their URNs.
///
/// All three URNs parse, but only `access_token` is handled for now (see
/// #1050): [`TokenType::ensure_supported`] rejects the others with
/// `unsupported_token_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum TokenType {
    #[serde(rename = "urn:ietf:params:oauth:token-type:access_token")]
    AccessToken,

    #[serde(rename = "urn:ietf:params:oauth:token-type:id_token")]
    IdToken,

    #[serde(rename = "urn:ietf:params:oauth:token-type:jwt")]
    Jwt,
}

impl TokenType {
    /// Parses an RFC 8693 token-type URN.
    ///
    /// Unknown URNs map to `unsupported_token_type` (RFC 8693 §2.2.2), not to
    /// a generic parse error, so the token endpoint can forward the code
    /// verbatim.
    pub fn from_urn(urn: &str) -> Result<Self, TokenExchangeError> {
        match urn {
            "urn:ietf:params:oauth:token-type:access_token" => Ok(Self::AccessToken),
            "urn:ietf:params:oauth:token-type:id_token" => Ok(Self::IdToken),
            "urn:ietf:params:oauth:token-type:jwt" => Ok(Self::Jwt),
            _ => Err(TokenExchangeError::UnsupportedTokenType),
        }
    }

    /// Returns the token type if the exchange supports it.
    ///
    /// Only `access_token` is handled for now; `id_token` and `jwt` parse but
    /// are rejected with `unsupported_token_type`.
    pub fn ensure_supported(self) -> Result<Self, TokenExchangeError> {
        match self {
            Self::AccessToken => Ok(self),
            Self::IdToken | Self::Jwt => Err(TokenExchangeError::UnsupportedTokenType),
        }
    }
}

/// Errors surfaced by the token exchange grant (RFC 8693).
///
/// The `Display` strings intentionally match the `error` codes defined by
/// RFC 8693 §2.2.2 so the HTTP layer can forward them verbatim in the token
/// endpoint response (same convention as `DeviceFlowError`).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TokenExchangeError {
    /// The requested or supplied token type is valid but not supported by
    /// this authorization server (RFC 8693 §2.2.2 `unsupported_token_type`).
    #[error("unsupported_token_type")]
    UnsupportedTokenType,
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCESS_TOKEN_URN: &str = "urn:ietf:params:oauth:token-type:access_token";
    const ID_TOKEN_URN: &str = "urn:ietf:params:oauth:token-type:id_token";
    const JWT_URN: &str = "urn:ietf:params:oauth:token-type:jwt";

    #[test]
    fn token_type_serde_round_trips_rfc8693_urns() {
        for (token_type, urn) in [
            (TokenType::AccessToken, ACCESS_TOKEN_URN),
            (TokenType::IdToken, ID_TOKEN_URN),
            (TokenType::Jwt, JWT_URN),
        ] {
            let json = format!("\"{urn}\"");
            let parsed: TokenType = serde_json::from_str(&json)
                .expect("RFC 8693 URN should deserialize into TokenType");
            assert_eq!(parsed, token_type);
            let serialized =
                serde_json::to_string(&token_type).expect("TokenType should serialize");
            assert_eq!(serialized, json);
        }
    }

    #[test]
    fn from_urn_parses_known_urns() {
        assert_eq!(
            TokenType::from_urn(ACCESS_TOKEN_URN),
            Ok(TokenType::AccessToken)
        );
        assert_eq!(TokenType::from_urn(ID_TOKEN_URN), Ok(TokenType::IdToken));
        assert_eq!(TokenType::from_urn(JWT_URN), Ok(TokenType::Jwt));
    }

    #[test]
    fn from_urn_unknown_maps_to_unsupported_token_type() {
        assert_eq!(
            TokenType::from_urn("urn:ietf:params:oauth:token-type:saml2"),
            Err(TokenExchangeError::UnsupportedTokenType)
        );
    }

    #[test]
    fn ensure_supported_accepts_only_access_token() {
        assert_eq!(
            TokenType::AccessToken.ensure_supported(),
            Ok(TokenType::AccessToken)
        );
        assert_eq!(
            TokenType::IdToken.ensure_supported(),
            Err(TokenExchangeError::UnsupportedTokenType)
        );
        assert_eq!(
            TokenType::Jwt.ensure_supported(),
            Err(TokenExchangeError::UnsupportedTokenType)
        );
    }

    #[test]
    fn unsupported_token_type_displays_rfc8693_error_code() {
        assert_eq!(
            TokenExchangeError::UnsupportedTokenType.to_string(),
            "unsupported_token_type"
        );
    }
}
