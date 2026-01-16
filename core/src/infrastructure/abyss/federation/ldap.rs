use std::collections::HashMap;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use ldap3::{Ldap, LdapConnAsync, LdapConnSettings, Scope, SearchEntry, SearchResult};
use serde::Deserialize;
use tokio::time::timeout;
use tracing::{info, instrument, warn};

use crate::domain::abyss::federation::entities::{FederatedUser, FederationProvider};
use crate::domain::abyss::federation::value_objects::TestConnectionResult;
use crate::domain::common::entities::app_errors::CoreError;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct LdapClientImpl;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LdapConfig {
    connection: LdapConnection,
    bind: LdapBind,
    search: LdapSearch,
    attributes: LdapAttributes,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LdapConnection {
    server_url: String,
    #[allow(dead_code)]
    port: i32,
    use_tls: bool,
    use_starttls: bool,
    connection_timeout_seconds: u64,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LdapBind {
    bind_dn: String,
    bind_password_encrypted: String, // TODO: Decrypt this
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LdapSearch {
    base_dn: String,
    user_search_filter: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LdapAttributes {
    username: String,
    email: String,
    first_name: String,
    last_name: String,
    /// Field to use as external_id (e.g., "entryUUID" for OpenLDAP, "objectGUID" for AD)
    /// If None, will fall back to using DN
    #[serde(default)]
    external_id_attribute: Option<String>,
}

impl LdapClientImpl {
    #[allow(dead_code)]
    fn parse_config(provider: &FederationProvider) -> Result<LdapConfig, CoreError> {
        serde_json::from_value(provider.config.clone())
            .map_err(|e| CoreError::Configuration(format!("Invalid LDAP config: {}", e)))
    }

    /// Decrypt the encrypted bind password
    /// TODO: Integrate with a proper encryption service (e.g., KMS, Vault)
    #[allow(dead_code)]
    fn decrypt_password(encrypted: &str) -> Result<String, CoreError> {
        // Placeholder: In production, call your encryption service
        // For now, assume the password is base64 encoded or plaintext
        tracing::warn!("Using placeholder password decryption - implement proper KMS integration");

        // Try base64 decode as a simple obfuscation
        match BASE64_STANDARD.decode(encrypted) {
            Ok(decoded) => String::from_utf8(decoded)
                .map_err(|e| CoreError::Configuration(format!("Invalid password encoding: {}", e))),
            Err(_) => {
                // If not base64, assume plaintext (for development only!)
                tracing::warn!("Password not base64 encoded, using as plaintext");
                Ok(encrypted.to_string())
            }
        }
    }

    #[allow(dead_code)]
    fn build_url(config: &LdapConfig) -> String {
        let mut url = config.connection.server_url.clone();

        if !url.starts_with("ldap://") && !url.starts_with("ldaps://") {
            if config.connection.use_tls {
                url = format!("ldaps://{}", url);
            } else {
                url = format!("ldap://{}", url);
            }
        }

        // Basic port handling
        if !url.contains("://") {
            let schema = if config.connection.use_tls {
                "ldaps"
            } else {
                "ldap"
            };
            url = format!(
                "{}://{}:{}",
                schema, config.connection.server_url, config.connection.port
            );
        }

        url
    }

    #[instrument(skip(self, provider))]
    #[allow(dead_code)]
    pub async fn connect(&self, provider: &FederationProvider) -> Result<Ldap, CoreError> {
        let config = Self::parse_config(provider)?;
        let url = Self::build_url(&config);

        let timeout_duration = Duration::from_secs(config.connection.connection_timeout_seconds);

        // Configure connection settings for StartTLS
        let settings = if config.connection.use_starttls {
            LdapConnSettings::new().set_starttls(true)
        } else {
            LdapConnSettings::new()
        };

        let (conn, ldap) = timeout(
            timeout_duration,
            LdapConnAsync::with_settings(settings, &url),
        )
        .await
        .map_err(|_| CoreError::External("LDAP Connection timeout".to_string()))?
        .map_err(|e| CoreError::External(format!("LDAP Connection failed: {}", e)))?;

        // StartTLS is handled by the connection settings
        // The ldap3 crate will automatically negotiate TLS if set_starttls(true) is used

        ldap3::drive!(conn);

        Ok(ldap)
    }

    #[instrument(skip(self, provider))]
    #[allow(dead_code)]
    pub async fn bind(&self, provider: &FederationProvider) -> Result<Ldap, CoreError> {
        let mut ldap = self.connect(provider).await?;
        let config = Self::parse_config(provider)?;

        // Decrypt the password before using it
        let password = Self::decrypt_password(&config.bind.bind_password_encrypted)?;

        ldap.simple_bind(&config.bind.bind_dn, &password)
            .await
            .map_err(|e| CoreError::External(format!("LDAP Bind failed: {}", e)))?;

        Ok(ldap)
    }

    #[instrument(skip(self, provider, password))]
    #[allow(dead_code)]
    pub async fn authenticate_user(
        &self,
        provider: &FederationProvider,
        username: &str,
        password: &str,
    ) -> Result<FederatedUser, CoreError> {
        // 0. CRITICAL: Reject empty passwords to prevent anonymous/unauthenticated binds
        if password.is_empty() {
            warn!(
                "Rejected LDAP authentication attempt with empty password for user: {}",
                username
            );
            return Err(CoreError::FederationAuthenticationFailed(
                "Empty password not allowed".to_string(),
            ));
        }

        // 1. Bind as admin to search for user
        let mut ldap = self.bind(provider).await?;
        let config = Self::parse_config(provider)?;

        // 2. Search for user DN
        let filter = config
            .search
            .user_search_filter
            .replace("{0}", username)
            .replace("{username}", username);

        let SearchResult(rs, _res) = ldap
            .search(
                &config.search.base_dn,
                Scope::Subtree,
                &filter,
                vec!["*"], // Fetch all attributes to map them
            )
            .await
            .map_err(|e| CoreError::External(format!("LDAP User Search failed: {}", e)))?;

        let mut matched_entry: Option<SearchEntry> = None;
        for entry in rs {
            let search_entry = SearchEntry::construct(entry);
            let attr_name = &config.attributes.username;
            let attr_value = search_entry
                .attrs
                .get(attr_name)
                .and_then(|vals| vals.first())
                .map(|v| v.as_str());

            if attr_value == Some(username) {
                matched_entry = Some(search_entry);
                break;
            }
        }

        let search_entry = matched_entry.ok_or(CoreError::FederationAuthenticationFailed(
            format!("User {} not found in LDAP", username),
        ))?;
        let user_dn = search_entry.dn.clone();

        info!(
            "Found user '{}' in LDAP with DN: '{}' (attributes: {:?})",
            username,
            user_dn,
            search_entry.attrs.keys().collect::<Vec<_>>()
        );

        // Must unbind admin connection before rebinding as user
        let _ = ldap.unbind().await;

        // 3. CRITICAL: Create NEW connection and bind with user DN and password to verify credentials
        // Using the same connection can cause issues with some LDAP servers
        let mut user_ldap = self.connect(provider).await?;

        info!(
            "Attempting LDAP bind for user '{}' with DN: '{}' and password length: {}",
            username,
            user_dn,
            password.len()
        );

        let bind_result = user_ldap.simple_bind(&user_dn, password).await;

        match bind_result {
            Ok(bind_response) => {
                // Check bind result code - 0 = success, anything else = failure
                if bind_response.rc != 0 {
                    warn!(
                        "LDAP bind failed for user {} with result code: {}",
                        username, bind_response.rc
                    );
                    let _ = user_ldap.unbind().await;
                    return Err(CoreError::FederationAuthenticationFailed(
                        "Invalid LDAP credentials".to_string(),
                    ));
                }
            }
            Err(e) => {
                warn!("LDAP bind error for user {}: {}", username, e);
                let _ = user_ldap.unbind().await;
                return Err(CoreError::FederationAuthenticationFailed(
                    "Invalid LDAP credentials".to_string(),
                ));
            }
        }

        // 4. Map to FederatedUser
        let federated_user = self.map_entry_to_user(&search_entry, &config.attributes)?;

        // Cleanup
        let _ = user_ldap.unbind().await;

        Ok(federated_user)
    }

    #[instrument(skip(self, provider))]
    #[allow(dead_code)]
    pub async fn search_users(
        &self,
        provider: &FederationProvider,
        filter_override: Option<&str>,
    ) -> Result<Vec<FederatedUser>, CoreError> {
        let mut ldap = self.bind(provider).await?;
        let config = Self::parse_config(provider)?;

        let filter = filter_override
            .unwrap_or(&config.search.user_search_filter)
            .replace("{0}", "*")
            .replace("{username}", "*");

        let SearchResult(rs, _res) = ldap
            .search(&config.search.base_dn, Scope::Subtree, &filter, vec!["*"])
            .await
            .map_err(|e| CoreError::External(format!("LDAP Search failed: {}", e)))?;

        let mut users = Vec::new();
        for entry in rs {
            let search_entry = SearchEntry::construct(entry);
            if let Ok(user) = self.map_entry_to_user(&search_entry, &config.attributes) {
                users.push(user);
            }
        }

        let _ = ldap.unbind().await;

        Ok(users)
    }

    #[instrument(skip(self, provider))]
    #[allow(dead_code)]
    pub async fn get_user_by_username(
        &self,
        provider: &FederationProvider,
        username: &str,
    ) -> Result<Option<FederatedUser>, CoreError> {
        let mut ldap = self.bind(provider).await?;
        let config = Self::parse_config(provider)?;

        let filter = config
            .search
            .user_search_filter
            .replace("{0}", username)
            .replace("{username}", username);

        let SearchResult(rs, _res) = ldap
            .search(&config.search.base_dn, Scope::Subtree, &filter, vec!["*"])
            .await
            .map_err(|e| CoreError::External(format!("LDAP Search failed: {}", e)))?;

        let user = if let Some(entry) = rs.into_iter().next() {
            let search_entry = SearchEntry::construct(entry);
            Some(self.map_entry_to_user(&search_entry, &config.attributes)?)
        } else {
            None
        };

        let _ = ldap.unbind().await;
        Ok(user)
    }

    #[instrument(skip(self, provider))]
    #[allow(dead_code)]
    pub async fn test_connection(
        &self,
        provider: &FederationProvider,
    ) -> Result<TestConnectionResult, CoreError> {
        let config = Self::parse_config(provider)?;

        // 1. Test Connectivity
        let conn_start = std::time::Instant::now();
        let mut ldap = match self.connect(provider).await {
            Ok(l) => l,
            Err(e) => {
                return Ok(TestConnectionResult {
                    success: false,
                    message: format!("Failed to connect: {}", e),
                    details: None,
                });
            }
        };

        // 2. Test Bind
        let password = Self::decrypt_password(&config.bind.bind_password_encrypted)?;
        if let Err(e) = ldap.simple_bind(&config.bind.bind_dn, &password).await {
            return Ok(TestConnectionResult {
                success: false,
                message: format!("Failed to bind: {}", e),
                details: Some(
                    serde_json::json!({ "latency_ms": conn_start.elapsed().as_millis() as u64 }),
                ),
            });
        }

        // 3. Test Search (Base DN access)
        if let Err(e) = ldap
            .search(
                &config.search.base_dn,
                Scope::Base,
                "(objectClass=*)",
                vec!["1.1"],
            )
            .await
        {
            return Ok(TestConnectionResult {
                success: false,
                message: format!("Failed to search base DN: {}", e),
                details: Some(
                    serde_json::json!({ "latency_ms": conn_start.elapsed().as_millis() as u64 }),
                ),
            });
        }

        // 4. Test User Count (Optional but good)
        // Just verify we can use the filter
        let filter = config
            .search
            .user_search_filter
            .replace("{0}", "*")
            .replace("{username}", "*");
        let _ = ldap
            .search(&config.search.base_dn, Scope::Subtree, &filter, vec!["1.1"])
            .await;

        let latency = conn_start.elapsed().as_millis() as u64;
        let _ = ldap.unbind().await;

        Ok(TestConnectionResult {
            success: true,
            message: "Successfully connected and authenticated".to_string(),
            details: Some(serde_json::json!({ "latency_ms": latency })),
        })
    }

    #[allow(dead_code)]
    fn map_entry_to_user(
        &self,
        entry: &SearchEntry,
        attributes: &LdapAttributes,
    ) -> Result<FederatedUser, CoreError> {
        let get_attr = |name: &str| -> Option<String> {
            entry.attrs.get(name).and_then(|vals| vals.first()).cloned()
        };

        let username = get_attr(&attributes.username).ok_or_else(|| {
            CoreError::External(format!(
                "Missing username attribute: {}",
                attributes.username
            ))
        })?;

        // Optional fields
        let email = get_attr(&attributes.email);
        let first_name = get_attr(&attributes.first_name);
        let last_name = get_attr(&attributes.last_name);

        // Determine external_id based on configuration
        let external_id = if let Some(ref id_attr) = attributes.external_id_attribute {
            // Try to use configured attribute (e.g., entryUUID, objectGUID)
            get_attr(id_attr).unwrap_or_else(|| {
                warn!(
                    "Configured external_id attribute '{}' not found, falling back to DN",
                    id_attr
                );
                entry.dn.clone()
            })
        } else {
            // Default to DN if no specific attribute is configured
            entry.dn.clone()
        };

        let mut all_attributes = HashMap::new();
        for (k, v) in &entry.attrs {
            all_attributes.insert(k.clone(), v.clone());
        }

        Ok(FederatedUser {
            external_id,
            username,
            email,
            first_name,
            last_name,
            attributes: Some(all_attributes),
        })
    }
}
