use std::collections::HashSet;

/// Standard OpenID Connect scopes
pub const SCOPE_OPENID: &str = "openid";
pub const SCOPE_PROFILE: &str = "profile";
pub const SCOPE_EMAIL: &str = "email";
pub const SCOPE_ADDRESS: &str = "address";
pub const SCOPE_PHONE: &str = "phone";
pub const SCOPE_OFFLINE_ACCESS: &str = "offline_access";

pub const DEFAULT_SCOPES: &[&str] = &[SCOPE_OPENID, SCOPE_PROFILE, SCOPE_EMAIL];

#[derive(Debug, Clone)]
pub struct ScopeManager {
    allowed_scopes: HashSet<String>,
}

impl ScopeManager {
    /// Create a new ScopeManager with standard OIDC scopes
    pub fn new() -> Self {
        let mut allowed_scopes = HashSet::new();
        allowed_scopes.insert(SCOPE_OPENID.to_string());
        allowed_scopes.insert(SCOPE_PROFILE.to_string());
        allowed_scopes.insert(SCOPE_EMAIL.to_string());
        allowed_scopes.insert(SCOPE_ADDRESS.to_string());
        allowed_scopes.insert(SCOPE_PHONE.to_string());
        allowed_scopes.insert(SCOPE_OFFLINE_ACCESS.to_string());

        Self { allowed_scopes }
    }

    /// Add custom scopes (for client-specific scopes)
    pub fn with_custom_scopes(mut self, custom_scopes: Vec<String>) -> Self {
        self.allowed_scopes.extend(custom_scopes);
        self
    }

    /// Parse and validate requested scopes
    /// Returns the validated scopes that are allowed
    pub fn validate_and_filter(&self, requested_scope: Option<String>) -> String {
        let requested = match requested_scope {
            Some(scope_str) if !scope_str.trim().is_empty() => scope_str
                .split_whitespace()
                .map(String::from)
                .collect::<HashSet<_>>(),
            _ => {
                // Si aucun scope n'est demandé, utiliser les scopes par défaut
                return DEFAULT_SCOPES.join(" ");
            }
        };

        // Filtrer pour garder seulement les scopes autorisés
        let validated: Vec<String> = requested
            .iter()
            .filter(|scope| self.allowed_scopes.contains(*scope))
            .cloned()
            .collect();

        // S'assurer que 'openid' est toujours présent pour OIDC
        let mut final_scopes = validated;
        if !final_scopes.contains(&SCOPE_OPENID.to_string()) {
            final_scopes.insert(0, SCOPE_OPENID.to_string());
        }

        final_scopes.join(" ")
    }

    /// Check if a specific scope is present in a scope string
    pub fn has_scope(scope_string: &str, scope: &str) -> bool {
        scope_string.split_whitespace().any(|s| s == scope)
    }

    /// Extract scopes as a vector
    pub fn parse_scopes(scope_string: &str) -> Vec<String> {
        scope_string.split_whitespace().map(String::from).collect()
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::authentication::scope::ScopeManager;

    #[test]
    fn test_validate_with_valid_scopes() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(Some("openid profile email".to_string()));
        assert_eq!(result, "openid profile email");
    }

    #[test]
    fn test_validate_with_invalid_scopes() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(Some("openid invalid_scope".to_string()));
        assert_eq!(result, "openid");
    }

    #[test]
    fn test_validate_without_openid_adds_it() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(Some("profile email".to_string()));
        assert!(result.starts_with("openid"));
    }

    #[test]
    fn test_default_scopes_when_none_requested() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(None);
        assert_eq!(result, "openid profile email");
    }
}
