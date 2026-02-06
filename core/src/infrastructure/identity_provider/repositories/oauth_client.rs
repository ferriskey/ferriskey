use reqwest::Client;
use tracing::instrument;

use crate::domain::abyss::identity_provider::broker::{
    BrokeredUserInfo, OAuthClient, OAuthTokenResponse,
};
use crate::domain::common::entities::app_errors::CoreError;

/// HTTP client implementation for OAuth operations with external IdPs
#[derive(Debug, Clone)]
pub struct ReqwestOAuthClient {
    client: Client,
}

impl ReqwestOAuthClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    #[allow(unused)]
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestOAuthClient {
    fn default() -> Self {
        Self::new()
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

        let response = self
            .client
            .post(token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Token exchange request failed: {}", e);
                CoreError::IdpTokenExchangeFailed(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Token exchange failed with status {}: {}", status, body);
            return Err(CoreError::IdpTokenExchangeFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let token_response: OAuthTokenResponse = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse token response: {}", e);
            CoreError::IdpTokenExchangeFailed(format!("Failed to parse response: {}", e))
        })?;

        Ok(token_response)
    }

    #[instrument(skip(self, access_token), fields(userinfo_url = %userinfo_url))]
    async fn fetch_userinfo(
        &self,
        userinfo_url: &str,
        access_token: &str,
    ) -> Result<BrokeredUserInfo, CoreError> {
        let response = self
            .client
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Userinfo request failed: {}", e);
                CoreError::IdpUserInfoFailed(format!("Request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Userinfo request failed with status {}: {}", status, body);
            return Err(CoreError::IdpUserInfoFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        // Parse the userinfo response - it can have various field names depending on the IdP
        let json: serde_json::Value = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse userinfo response: {}", e);
            CoreError::IdpUserInfoFailed(format!("Failed to parse response: {}", e))
        })?;

        let user_info = BrokeredUserInfo {
            subject: json["sub"]
                .as_str()
                .or_else(|| json["id"].as_str())
                .ok_or_else(|| CoreError::IdpUserInfoFailed("Missing subject claim".to_string()))?
                .to_string(),
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
        let _client = ReqwestOAuthClient::new();
        // Just verify it can be created without panic
    }

    #[test]
    fn test_reqwest_oauth_client_default() {
        let _client = ReqwestOAuthClient::default();
        // Just verify it can be created without panic
    }
}
