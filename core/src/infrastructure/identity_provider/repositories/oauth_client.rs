use std::time::Duration;

use reqwest::header::ACCEPT;
use reqwest::redirect;
use reqwest::{Client, Url};
use tokio::net::lookup_host;
use tracing::instrument;

use ferriskey_webhook::endpoint::{PrivateEndpoints, allows_cleartext, is_forbidden_address};

use crate::domain::abyss::identity_provider::broker::{
    BrokeredUserInfo, OAuthClient, OAuthTokenResponse,
};
use crate::domain::common::entities::app_errors::CoreError;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const USER_AGENT: &str = concat!("FerrisKey/", env!("CARGO_PKG_VERSION"));

/// HTTP client implementation for OAuth operations with external IdPs
#[derive(Debug, Clone)]
pub struct ReqwestOAuthClient {
    private_endpoints: PrivateEndpoints,
}

impl ReqwestOAuthClient {
    pub fn new(private_endpoints: PrivateEndpoints) -> Self {
        Self { private_endpoints }
    }

    async fn secured_client(&self, raw_url: &str) -> Result<Client, CoreError> {
        let url = Url::parse(raw_url).map_err(|_| CoreError::InvalidProviderUrl)?;

        let host = url
            .host_str()
            .ok_or(CoreError::InvalidProviderUrl)?
            .to_string();

        let port = url
            .port_or_known_default()
            .ok_or(CoreError::InvalidProviderUrl)?;

        let mut resolved = lookup_host((host.as_str(), port)).await.map_err(|error| {
            tracing::error!(%host, %error, "identity provider host did not resolve");
            CoreError::InvalidProviderUrl
        })?;

        let addr = resolved
            .find(|candidate| !is_forbidden_address(candidate.ip(), self.private_endpoints))
            .ok_or_else(|| {
                tracing::error!(%host, "identity provider host resolved to a forbidden address");
                CoreError::InvalidProviderUrl
            })?;

        if url.scheme() != "https" && !allows_cleartext(addr.ip(), self.private_endpoints) {
            tracing::error!(%host, "identity provider endpoint may not use cleartext http");
            return Err(CoreError::InvalidProviderUrl);
        }

        Client::builder()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(redirect::Policy::none())
            .resolve(&host, addr)
            .build()
            .map_err(|error| {
                tracing::error!(%host, %error, "failed to build the identity provider http client");
                CoreError::InvalidProviderUrl
            })
    }
}

impl Default for ReqwestOAuthClient {
    fn default() -> Self {
        Self::new(PrivateEndpoints::default())
    }
}

impl OAuthClient for ReqwestOAuthClient {
    #[instrument(skip(self, client_secret, code_verifier), fields(token_url = %token_url))]
    async fn exchange_code(
        &self,
        token_url: &str,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
        client_secret: &str,
        code_verifier: Option<&str>,
    ) -> Result<OAuthTokenResponse, CoreError> {
        let client = self.secured_client(token_url).await?;

        let mut params = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];

        // Add PKCE code verifier if present
        if let Some(verifier) = code_verifier {
            params.push(("code_verifier", verifier));
        }

        let response = client
            .post(token_url)
            // GitHub (and some other IdPs) default to form-urlencoded token
            // responses unless the client explicitly asks for JSON.
            .header(ACCEPT, "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, "token exchange request failed");
                CoreError::IdpTokenExchangeFailed("upstream request failed".to_string())
            })?;

        if !response.status().is_success() {
            tracing::error!(status = %response.status(), "token exchange returned a non-success status");
            return Err(CoreError::IdpTokenExchangeFailed(
                "upstream request failed".to_string(),
            ));
        }

        let token_response: OAuthTokenResponse = response.json().await.map_err(|error| {
            tracing::error!(%error, "failed to parse the token response");
            CoreError::IdpTokenExchangeFailed("upstream request failed".to_string())
        })?;

        Ok(token_response)
    }

    #[instrument(skip(self), fields(jwks_url = %jwks_url))]
    async fn fetch_jwks(&self, jwks_url: &str) -> Result<serde_json::Value, CoreError> {
        let client = self.secured_client(jwks_url).await?;

        let response = client.get(jwks_url).send().await.map_err(|error| {
            tracing::error!(%error, "jwks request failed");
            CoreError::InvalidIdToken
        })?;

        if !response.status().is_success() {
            tracing::error!(status = %response.status(), "jwks request returned a non-success status");
            return Err(CoreError::InvalidIdToken);
        }

        response.json().await.map_err(|error| {
            tracing::error!(%error, "failed to parse the jwks document");
            CoreError::InvalidIdToken
        })
    }

    #[instrument(skip(self, access_token), fields(userinfo_url = %userinfo_url))]
    async fn fetch_userinfo(
        &self,
        userinfo_url: &str,
        access_token: &str,
    ) -> Result<BrokeredUserInfo, CoreError> {
        let client = self.secured_client(userinfo_url).await?;

        let response = client
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, "userinfo request failed");
                CoreError::IdpUserInfoFailed("upstream request failed".to_string())
            })?;

        if !response.status().is_success() {
            tracing::error!(status = %response.status(), "userinfo request returned a non-success status");
            return Err(CoreError::IdpUserInfoFailed(
                "upstream request failed".to_string(),
            ));
        }

        // Parse the userinfo response - it can have various field names depending on the IdP
        let json: serde_json::Value = response.json().await.map_err(|error| {
            tracing::error!(%error, "failed to parse the userinfo response");
            CoreError::IdpUserInfoFailed("upstream request failed".to_string())
        })?;

        let user_info = BrokeredUserInfo {
            subject: json["sub"]
                .as_str()
                .map(|s| s.to_string())
                // GitHub returns the user id as a JSON number, not a string.
                .or_else(|| match &json["id"] {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                })
                .ok_or_else(|| CoreError::IdpUserInfoFailed("Missing subject claim".to_string()))?,
            email: json["email"].as_str().map(|s| s.to_string()),
            email_verified: json["email_verified"].as_bool(),
            name: json["name"].as_str().map(|s| s.to_string()),
            given_name: json["given_name"]
                .as_str()
                .or_else(|| json["first_name"].as_str())
                .map(|s| s.to_string()),
            family_name: json["family_name"]
                .as_str()
                .or_else(|| json["last_name"].as_str())
                .map(|s| s.to_string()),
            preferred_username: json["preferred_username"]
                .as_str()
                .or_else(|| json["username"].as_str())
                .or_else(|| json["login"].as_str())
                .map(|s| s.to_string()),
            picture: json["picture"]
                .as_str()
                .or_else(|| json["avatar_url"].as_str())
                .map(|s| s.to_string()),
        };

        Ok(user_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reqwest_oauth_client_new() {
        let _client = ReqwestOAuthClient::new(PrivateEndpoints::from_allowed(false));
        // Just verify it can be created without panic
    }

    #[test]
    fn test_reqwest_oauth_client_default() {
        let _client = ReqwestOAuthClient::default();
        // Just verify it can be created without panic
    }

    #[test]
    fn forbidden_addresses_are_rejected_when_private_endpoints_disabled() {
        let policy = PrivateEndpoints::from_allowed(false);

        assert!(is_forbidden_address("127.0.0.1".parse().unwrap(), policy));
        assert!(is_forbidden_address("::1".parse().unwrap(), policy));
        assert!(is_forbidden_address(
            "169.254.169.254".parse().unwrap(),
            policy
        ));
    }

    #[tokio::test]
    async fn fetch_userinfo_rejects_loopback_target_without_reflecting_upstream() {
        let client = ReqwestOAuthClient::new(PrivateEndpoints::from_allowed(false));

        let error = client
            .fetch_userinfo("http://127.0.0.1:1/userinfo", "unused-access-token")
            .await
            .expect_err("a userinfo endpoint on loopback must be refused");

        assert!(matches!(error, CoreError::InvalidProviderUrl));
        assert_eq!(error.to_string(), "Invalid provider URL");
    }

    #[tokio::test]
    async fn exchange_code_rejects_loopback_target_without_reflecting_upstream() {
        let client = ReqwestOAuthClient::new(PrivateEndpoints::from_allowed(false));

        let error = client
            .exchange_code(
                "http://127.0.0.1:1/token",
                "authorization-code",
                "https://app.example/callback",
                "client-id",
                "client-secret",
                None,
            )
            .await
            .expect_err("a token endpoint on loopback must be refused");

        assert!(matches!(error, CoreError::InvalidProviderUrl));
    }
}
