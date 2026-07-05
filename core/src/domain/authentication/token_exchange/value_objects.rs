//! DTOs for the token exchange grant use cases (RFC 8693).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::authentication::token_exchange::entities::TokenType;

/// Input for the token endpoint with
/// `grant_type=urn:ietf:params:oauth:grant-type:token-exchange`
/// (RFC 8693 §2.1).
///
/// The token-type fields carry the raw URNs as received; the service parses
/// them with [`TokenType::from_urn`] so an unknown URN surfaces as
/// `unsupported_token_type` rather than a deserialization error.
pub struct TokenExchangeInput {
    /// The token being exchanged.
    pub subject_token: String,
    /// Token-type URN of `subject_token`.
    pub subject_token_type: String,
    /// Requested type for the issued token. Defaults to `access_token` when
    /// absent.
    pub requested_token_type: Option<String>,
    /// Target audience for the issued token.
    pub audience: Option<String>,
    /// Target resource URI for the issued token.
    pub resource: Option<String>,
    /// Requested scope; must be a subset of the subject token's scope.
    pub scope: Option<String>,
}

/// Output of a successful token exchange (RFC 8693 §2.2.1).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenExchangeOutput {
    pub access_token: String,
    /// Token-type URN of the issued token.
    pub issued_token_type: TokenType,
    /// `Bearer`, per RFC 8693 §2.2.1 for access tokens.
    pub token_type: String,
    /// Issued token lifetime, in seconds.
    #[schema(example = 300)]
    pub expires_in: i64,
    /// Scope of the issued token; omitted when identical to the subject
    /// token's scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::token_exchange::entities::TokenType;

    #[test]
    fn output_serializes_issued_token_type_as_urn_and_omits_absent_scope() {
        let output = TokenExchangeOutput {
            access_token: "new-token".to_string(),
            issued_token_type: TokenType::AccessToken,
            token_type: "Bearer".to_string(),
            expires_in: 300,
            scope: None,
        };

        let json = serde_json::to_value(&output).expect("output should serialize");
        assert_eq!(
            json["issued_token_type"],
            "urn:ietf:params:oauth:token-type:access_token"
        );
        assert_eq!(json["token_type"], "Bearer");
        assert_eq!(json["expires_in"], 300);
        assert!(
            json.get("scope").is_none(),
            "absent scope should be omitted from the RFC 8693 response"
        );
    }
}
