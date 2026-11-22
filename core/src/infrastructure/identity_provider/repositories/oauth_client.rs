use std::net::SocketAddr;
use std::time::Duration;

use reqwest::header::ACCEPT;
use reqwest::redirect;
use reqwest::{Client, Url};
use serde::Deserialize;
use tokio::net::lookup_host;
use tracing::{instrument, warn};

use ferriskey_webhook::endpoint::{PrivateEndpoints, allows_cleartext, is_forbidden_address};

use crate::domain::abyss::identity_provider::broker::{
    BrokeredUserInfo, OAuthClient, OAuthTokenResponse,
};
use crate::domain::common::entities::app_errors::CoreError;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const USER_AGENT: &str = concat!("FerrisKey/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddressRejection {
    AllForbidden,
    CleartextNotAllowed,
}

pub(crate) fn permitted_addresses(
    resolved: impl Iterator<Item = SocketAddr>,
    requires_tls: bool,
    policy: PrivateEndpoints,
) -> Result<Vec<SocketAddr>, AddressRejection> {
    let permitted: Vec<SocketAddr> = resolved
        .filter(|candidate| !is_forbidden_address(candidate.ip(), policy))
        .collect();

    if permitted.is_empty() {
        return Err(AddressRejection::AllForbidden);
    }

    if requires_tls {
        return Ok(permitted);
    }

    let cleartext: Vec<SocketAddr> = permitted
        .into_iter()
        .filter(|candidate| allows_cleartext(candidate.ip(), policy))
        .collect();

    if cleartext.is_empty() {
        return Err(AddressRejection::CleartextNotAllowed);
    }

    Ok(cleartext)
}

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

        let resolved = lookup_host((host.as_str(), port)).await.map_err(|error| {
            tracing::error!(%host, %error, "identity provider host did not resolve");
            CoreError::InvalidProviderUrl
        })?;

        let addresses =
            permitted_addresses(resolved, url.scheme() == "https", self.private_endpoints)
                .map_err(|rejection| {
                    match rejection {
                        AddressRejection::AllForbidden => tracing::error!(
                            %host,
                            "identity provider host resolved to a forbidden address"
                        ),
                        AddressRejection::CleartextNotAllowed => tracing::error!(
                            %host,
                            "identity provider endpoint may not use cleartext http"
                        ),
                    }

                    CoreError::InvalidProviderUrl
                })?;

        Client::builder()
            .no_proxy()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(redirect::Policy::none())
            .resolve_to_addrs(&host, &addresses)
            .build()
            .map_err(|error| {
                tracing::error!(%host, %error, "failed to build the identity provider http client");
                CoreError::InvalidProviderUrl
            })
    }

    async fn fetch_github_emails(
        &self,
        emails_url: &str,
        access_token: &str,
    ) -> Option<Vec<GitHubEmail>> {
        let client = self
            .secured_client(emails_url)
            .await
            .inspect_err(|error| warn!(%emails_url, %error, "github_emails_client_rejected"))
            .ok()?;

        let response = client
            .get(emails_url)
            .header(ACCEPT, "application/json")
            .bearer_auth(access_token)
            .send()
            .await
            .inspect_err(|error| warn!(%emails_url, %error, "github_emails_request_failed"))
            .ok()?;

        if !response.status().is_success() {
            warn!(%emails_url, status = %response.status(), "github_emails_bad_status");
            return None;
        }

        response
            .json::<Vec<GitHubEmail>>()
            .await
            .inspect_err(|error| warn!(%emails_url, %error, "github_emails_parse_failed"))
            .ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GitHubEmail {
    email: String,
    #[serde(default)]
    verified: bool,
    #[serde(default)]
    primary: bool,
}

fn github_emails_url(userinfo_url: &str, provider_id: Option<&str>) -> Option<String> {
    let mut url = Url::parse(userinfo_url).ok()?;
    let is_github = provider_id.is_some_and(|id| id.eq_ignore_ascii_case("github"))
        || url
            .host_str()
            .is_some_and(|host| host.eq_ignore_ascii_case("api.github.com"));

    if !is_github {
        return None;
    }

    let trimmed_path = url.path().trim_end_matches('/');
    if !trimmed_path.ends_with("/user") {
        return None;
    }

    let emails_path = format!("{trimmed_path}/emails");
    url.set_path(&emails_path);
    url.set_query(None);
    Some(url.to_string())
}

fn is_github_noreply(email: &str) -> bool {
    email
        .rsplit_once('@')
        .is_some_and(|(_, domain)| domain.eq_ignore_ascii_case("users.noreply.github.com"))
}

fn select_github_email(emails: Vec<GitHubEmail>) -> Option<GitHubEmail> {
    let mut non_empty: Vec<GitHubEmail> = emails
        .into_iter()
        .filter(|entry| !entry.email.trim().is_empty())
        .collect();

    let pos = non_empty
        .iter()
        .position(|entry| entry.primary && entry.verified)
        .or_else(|| non_empty.iter().position(|entry| entry.primary))
        .or_else(|| {
            non_empty
                .iter()
                .position(|entry| entry.verified && !is_github_noreply(&entry.email))
        })
        .or_else(|| non_empty.iter().position(|entry| entry.verified))
        .or((!non_empty.is_empty()).then_some(0))?;

    Some(non_empty.swap_remove(pos))
}

fn resolve_github_email(
    profile_email: Option<String>,
    emails: Vec<GitHubEmail>,
) -> (Option<String>, Option<bool>) {
    if let Some(current) = profile_email.filter(|e| !e.trim().is_empty()) {
        let verified = emails
            .iter()
            .filter(|entry| !entry.email.trim().is_empty())
            .find(|entry| entry.email.eq_ignore_ascii_case(&current))
            .map(|entry| entry.verified);
        return (Some(current), verified);
    }

    match select_github_email(emails) {
        Some(entry) => (Some(entry.email), Some(entry.verified)),
        None => (None, None),
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
        provider_id: Option<&str>,
    ) -> Result<BrokeredUserInfo, CoreError> {
        let client = self.secured_client(userinfo_url).await?;

        let response = client
            .get(userinfo_url)
            .header(ACCEPT, "application/json")
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

        let mut user_info = BrokeredUserInfo {
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
            email: json["email"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string()),
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

        if user_info.email_verified.is_none()
            && let Some(emails_url) = github_emails_url(userinfo_url, provider_id)
            && let Some(emails) = self.fetch_github_emails(&emails_url, access_token).await
        {
            let (email, email_verified) = resolve_github_email(user_info.email, emails);
            user_info.email = email;
            user_info.email_verified = email_verified;
        }

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

    fn addr(raw: &str) -> SocketAddr {
        raw.parse().expect("the test address literal must parse")
    }

    #[test]
    fn every_permitted_address_is_pinned_so_a_dead_one_can_fall_back() {
        let resolved = [addr("[2606:4700::1111]:443"), addr("93.184.216.34:443")];

        let addresses = permitted_addresses(
            resolved.into_iter(),
            true,
            PrivateEndpoints::from_allowed(false),
        )
        .expect("public addresses must be permitted");

        assert_eq!(addresses, resolved);
    }

    #[test]
    fn a_forbidden_address_is_dropped_without_discarding_its_permitted_siblings() {
        let resolved = [addr("169.254.169.254:443"), addr("93.184.216.34:443")];

        let addresses = permitted_addresses(
            resolved.into_iter(),
            true,
            PrivateEndpoints::from_allowed(false),
        )
        .expect("the public sibling must survive");

        assert_eq!(addresses, [addr("93.184.216.34:443")]);
    }

    #[test]
    fn a_host_resolving_only_to_forbidden_addresses_is_refused() {
        let resolved = [addr("127.0.0.1:443"), addr("169.254.169.254:443")];

        let rejection = permitted_addresses(
            resolved.into_iter(),
            true,
            PrivateEndpoints::from_allowed(false),
        )
        .expect_err("every address is forbidden under the default policy");

        assert_eq!(rejection, AddressRejection::AllForbidden);
    }

    #[test]
    fn cleartext_is_refused_when_private_endpoints_are_not_opted_in() {
        let resolved = [addr("93.184.216.34:80")];

        let rejection = permitted_addresses(
            resolved.into_iter(),
            false,
            PrivateEndpoints::from_allowed(false),
        )
        .expect_err("cleartext towards a public address must be refused");

        assert_eq!(rejection, AddressRejection::CleartextNotAllowed);
    }

    #[test]
    fn cleartext_pins_loopback_only_and_leaves_its_public_sibling_out() {
        let resolved = [addr("93.184.216.34:80"), addr("127.0.0.1:80")];

        let addresses = permitted_addresses(
            resolved.into_iter(),
            false,
            PrivateEndpoints::from_allowed(true),
        )
        .expect("loopback may be reached in cleartext once private endpoints are allowed");

        assert_eq!(addresses, [addr("127.0.0.1:80")]);
    }

    #[tokio::test]
    async fn fetch_userinfo_rejects_loopback_target_without_reflecting_upstream() {
        let client = ReqwestOAuthClient::new(PrivateEndpoints::from_allowed(false));

        let error = client
            .fetch_userinfo("http://127.0.0.1:1/userinfo", "unused-access-token", None)
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

    #[test]
    fn github_emails_url_resolves_for_github_com_and_enterprise_only() {
        assert_eq!(
            github_emails_url("https://api.github.com/user", Some("github")).as_deref(),
            Some("https://api.github.com/user/emails")
        );
        assert_eq!(
            github_emails_url("https://api.github.com/user/", None).as_deref(),
            Some("https://api.github.com/user/emails")
        );
        assert_eq!(
            github_emails_url("https://ghe.example.com/api/v3/user", Some("github")).as_deref(),
            Some("https://ghe.example.com/api/v3/user/emails")
        );
        assert_eq!(
            github_emails_url("https://gitlab.com/api/v4/user", Some("gitlab")),
            None
        );
        assert_eq!(
            github_emails_url(
                "https://www.googleapis.com/oauth2/v3/userinfo",
                Some("google")
            ),
            None
        );
    }

    #[test]
    fn resolve_github_email_marks_matching_public_email_as_verified() {
        let emails = vec![
            GitHubEmail {
                email: "primary@example.com".to_string(),
                verified: true,
                primary: true,
            },
            GitHubEmail {
                email: "public@example.com".to_string(),
                verified: true,
                primary: false,
            },
        ];

        let (email, verified) =
            resolve_github_email(Some("Public@Example.com".to_string()), emails);

        assert_eq!(email.as_deref(), Some("Public@Example.com"));
        assert_eq!(verified, Some(true));
    }

    #[test]
    fn resolve_github_email_selects_primary_verified_when_profile_email_is_private() {
        let emails = vec![
            GitHubEmail {
                email: "12345+octocat@users.noreply.github.com".to_string(),
                verified: true,
                primary: false,
            },
            GitHubEmail {
                email: "octocat@github.com".to_string(),
                verified: true,
                primary: true,
            },
        ];

        let (email, verified) = resolve_github_email(None, emails);

        assert_eq!(email.as_deref(), Some("octocat@github.com"));
        assert_eq!(verified, Some(true));
    }

    #[test]
    fn resolve_github_email_keeps_unverified_primary_instead_of_picking_noreply() {
        let emails = vec![
            GitHubEmail {
                email: "12345+octocat@users.noreply.github.com".to_string(),
                verified: true,
                primary: false,
            },
            GitHubEmail {
                email: "unverified@example.com".to_string(),
                verified: false,
                primary: true,
            },
        ];

        let (email, verified) = resolve_github_email(None, emails);

        assert_eq!(email.as_deref(), Some("unverified@example.com"));
        assert_eq!(verified, Some(false));
    }

    async fn spawn_mock_github_server(
        user_body: &'static str,
        emails_status: &'static str,
        emails_body: &'static str,
    ) -> SocketAddr {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            for _ in 0..2 {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let first_line = request.lines().next().unwrap_or_default();

                let (status, body) = if first_line.starts_with("GET /user/emails ") {
                    (emails_status, emails_body)
                } else {
                    ("200 OK", user_body)
                };

                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });

        addr
    }

    #[tokio::test]
    async fn fetch_userinfo_populates_verified_email_from_github_emails_endpoint() {
        let addr = spawn_mock_github_server(
            r#"{"id":42,"login":"octocat","email":null,"name":"Monalisa Octocat"}"#,
            "200 OK",
            r#"[{"email":"octocat@github.com","primary":true,"verified":true,"visibility":"private"}]"#,
        )
        .await;

        let client = ReqwestOAuthClient::new(PrivateEndpoints::from_allowed(true));
        let user_info = client
            .fetch_userinfo(&format!("http://{addr}/user"), "gh-token", Some("github"))
            .await
            .unwrap();

        assert_eq!(user_info.subject, "42");
        assert_eq!(user_info.preferred_username.as_deref(), Some("octocat"));
        assert_eq!(user_info.email.as_deref(), Some("octocat@github.com"));
        assert_eq!(user_info.email_verified, Some(true));
    }

    #[tokio::test]
    async fn fetch_userinfo_gracefully_falls_back_when_github_emails_endpoint_fails() {
        let addr = spawn_mock_github_server(
            r#"{"id":42,"login":"octocat","email":"octocat@github.com"}"#,
            "403 Forbidden",
            r#"{"message":"Resource not accessible by integration"}"#,
        )
        .await;

        let client = ReqwestOAuthClient::new(PrivateEndpoints::from_allowed(true));
        let user_info = client
            .fetch_userinfo(&format!("http://{addr}/user"), "gh-token", Some("github"))
            .await
            .unwrap();

        assert_eq!(user_info.subject, "42");
        assert_eq!(user_info.email.as_deref(), Some("octocat@github.com"));
        assert_eq!(user_info.email_verified, None);
    }
}
