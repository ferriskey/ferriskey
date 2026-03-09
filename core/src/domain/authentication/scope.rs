use std::collections::HashSet;
use std::fmt;

/// Standard OpenID Connect scopes
pub const SCOPE_OPENID: &str = "openid";
pub const SCOPE_PROFILE: &str = "profile";
pub const SCOPE_EMAIL: &str = "email";
pub const SCOPE_ADDRESS: &str = "address";
pub const SCOPE_PHONE: &str = "phone";
pub const SCOPE_OFFLINE_ACCESS: &str = "offline_access";
pub const SCOPE_INTROSPECT: &str = "introspect";

pub const DEFAULT_SCOPES: &[&str] = &[SCOPE_PROFILE, SCOPE_EMAIL];

/// Typed representation of all standard OIDC scopes supported by FerrisKey.
///
/// Use [`OidcScope::is_standard`] to test whether an arbitrary string is a
/// standard scope, and [`OidcScope::all`] to iterate over the full set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OidcScope {
    OpenId,
    Profile,
    Email,
    Address,
    Phone,
    OfflineAccess,
    Introspect,
}

impl OidcScope {
    /// The wire-format string value of this scope (e.g. `"openid"`).
    pub fn as_str(self) -> &'static str {
        match self {
            OidcScope::OpenId => SCOPE_OPENID,
            OidcScope::Profile => SCOPE_PROFILE,
            OidcScope::Email => SCOPE_EMAIL,
            OidcScope::Address => SCOPE_ADDRESS,
            OidcScope::Phone => SCOPE_PHONE,
            OidcScope::OfflineAccess => SCOPE_OFFLINE_ACCESS,
            OidcScope::Introspect => SCOPE_INTROSPECT,
        }
    }

    /// All standard OIDC scopes in a stable order.
    pub fn all() -> &'static [OidcScope] {
        &[
            OidcScope::OpenId,
            OidcScope::Profile,
            OidcScope::Email,
            OidcScope::Address,
            OidcScope::Phone,
            OidcScope::OfflineAccess,
            OidcScope::Introspect,
        ]
    }

    /// Returns `true` when `scope` matches any standard OIDC scope string.
    pub fn is_standard(scope: &str) -> bool {
        Self::all().iter().any(|s| s.as_str() == scope)
    }
}

impl fmt::Display for OidcScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for OidcScope {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            SCOPE_OPENID => Ok(OidcScope::OpenId),
            SCOPE_PROFILE => Ok(OidcScope::Profile),
            SCOPE_EMAIL => Ok(OidcScope::Email),
            SCOPE_ADDRESS => Ok(OidcScope::Address),
            SCOPE_PHONE => Ok(OidcScope::Phone),
            SCOPE_OFFLINE_ACCESS => Ok(OidcScope::OfflineAccess),
            SCOPE_INTROSPECT => Ok(OidcScope::Introspect),
            _ => Err(()),
        }
    }
}

impl From<OidcScope> for &'static str {
    fn from(scope: OidcScope) -> Self {
        scope.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct ScopeManager {
    allowed_scopes: HashSet<String>,
}

impl ScopeManager {
    /// Create a new ScopeManager with standard OIDC scopes
    pub fn new() -> Self {
        let allowed_scopes = OidcScope::all()
            .iter()
            .map(|s| s.as_str().to_string())
            .collect();

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
        let scopes_set = match requested_scope {
            Some(scope_str) if !scope_str.trim().is_empty() => scope_str
                .split_whitespace()
                .filter(|scope| self.allowed_scopes.contains(*scope))
                .map(String::from)
                .collect::<HashSet<_>>(),
            _ => DEFAULT_SCOPES
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>(),
        };

        let mut final_scopes: Vec<String> = scopes_set.into_iter().collect();
        final_scopes.sort_by(|a, b| {
            if a == SCOPE_OPENID {
                std::cmp::Ordering::Less
            } else if b == SCOPE_OPENID {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        final_scopes.join(" ")
    }

    /// Merge requested scopes with default scopes
    /// Returns: default scopes + additional valid requested scopes
    ///
    /// Example:
    /// - Defaults: "profile email"
    /// - Requested: "openid"
    /// - Result: "openid profile email"
    pub fn merge_with_defaults(&self, requested_scope: Option<String>) -> String {
        let mut scopes = DEFAULT_SCOPES
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();

        if let Some(scope_str) = requested_scope
            && !scope_str.trim().is_empty()
        {
            let requested: HashSet<String> = scope_str
                .split_whitespace()
                .filter(|scope| self.allowed_scopes.contains(*scope))
                .map(String::from)
                .collect();

            scopes.extend(requested);
        }

        let mut final_scope: Vec<String> = scopes.into_iter().collect();

        final_scope.sort_by(|a, b| {
            if a == SCOPE_OPENID {
                std::cmp::Ordering::Less
            } else if b == SCOPE_OPENID {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        final_scope.join(" ")
    }

    /// Check if a specific scope is present in a scope string
    pub fn has_scope(scope_string: &str, scope: &str) -> bool {
        scope_string.split_whitespace().any(|s| s == scope)
    }

    /// Extract scopes as a vector
    pub fn parse_scopes(scope_string: &str) -> Vec<String> {
        scope_string.split_whitespace().map(String::from).collect()
    }

    /// Get all allowed scopes as a space-separated string
    pub fn allowed_scopes(&self) -> String {
        let mut scopes: Vec<String> = self.allowed_scopes.iter().cloned().collect();
        scopes.sort();
        scopes.join(" ")
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
        let result = manager.validate_and_filter(Some("profile email".to_string()));
        assert!(result.contains("profile"));
        assert!(result.contains("email"));
    }

    #[test]
    fn test_validate_with_openid_explicitly() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(Some("openid profile email".to_string()));
        assert!(result.contains("openid"));
        assert!(result.contains("profile"));
        assert!(result.contains("email"));

        assert!(result.starts_with("openid"));
    }

    #[test]
    fn test_validate_with_invalid_scopes() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(Some("profile invalid_scope".to_string()));
        assert_eq!(result, "profile");
    }

    #[test]
    fn test_merge_with_defaults_allows_introspect_scope() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(Some("introspect".to_string()));
        assert!(result.contains("introspect"));
    }

    #[test]
    fn test_validate_without_openid_doesnt_add_it() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(Some("profile email".to_string()));

        assert!(!result.contains("openid"));
        assert!(result.contains("profile"));
        assert!(result.contains("email"));
    }

    #[test]
    fn test_default_scopes_when_none_requested() {
        let manager = ScopeManager::new();
        let result = manager.validate_and_filter(None);

        assert!(result.contains("profile"));
        assert!(result.contains("email"));
        assert!(!result.contains("openid"));
    }

    #[test]
    fn test_merge_with_defaults_adds_extra_scopes() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(Some("address phone".to_string()));

        assert!(result.contains("profile"), "Should include default profile");
        assert!(result.contains("email"), "Should include default email");
        assert!(
            result.contains("address"),
            "Should include requested address"
        );
        assert!(result.contains("phone"), "Should include requested phone");
        assert!(!result.contains("openid"), "Should NOT auto-add openid");
    }

    #[test]
    fn test_merge_with_defaults_filters_invalid() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(Some("address invalid_scope".to_string()));

        assert!(result.contains("profile"));
        assert!(result.contains("email"));
        assert!(result.contains("address"));
        assert!(!result.contains("invalid_scope"));
    }

    #[test]
    fn test_merge_with_defaults_no_duplicates() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(Some("profile address".to_string()));

        let count = result.matches("profile").count();
        assert_eq!(count, 1, "profile should appear only once");
    }

    #[test]
    fn test_merge_with_defaults_empty_string() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(Some("   ".to_string()));

        assert!(result.contains("profile"), "Should include default profile");
        assert!(result.contains("email"), "Should include default email");
        assert!(!result.contains("openid"), "Should NOT auto-add openid");
    }

    #[test]
    fn test_merge_with_defaults_none() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(None);

        assert!(result.contains("profile"));
        assert!(result.contains("email"));
        assert!(!result.contains("openid"));
    }

    #[test]
    fn test_merge_includes_openid_if_explicitly_requested() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(Some("openid address".to_string()));

        assert!(result.contains("openid"));
        assert!(result.contains("profile")); // defaults
        assert!(result.contains("email")); // defaults
        assert!(result.contains("address")); // requested

        assert!(result.starts_with("openid"));
    }

    #[test]
    fn test_openid_first_when_present() {
        let manager = ScopeManager::new();
        let result = manager.merge_with_defaults(Some("phone address openid".to_string()));

        assert!(
            result.starts_with("openid"),
            "openid should always be first when present"
        );
    }

    #[test]
    fn test_has_scope_helper() {
        assert!(ScopeManager::has_scope("openid profile email", "profile"));
        assert!(ScopeManager::has_scope("openid profile email", "openid"));
        assert!(!ScopeManager::has_scope("profile email", "openid"));
    }

    #[test]
    fn test_parse_scopes_helper() {
        let scopes = ScopeManager::parse_scopes("openid profile email");
        assert_eq!(scopes, vec!["openid", "profile", "email"]);
    }

    // --- Scope validation (RFC 6749 §3.3) ---

    #[test]
    fn test_is_standard_recognizes_all_oidc_scopes() {
        use super::OidcScope;
        for scope in OidcScope::all() {
            assert!(
                OidcScope::is_standard(scope.as_str()),
                "Expected '{}' to be a standard scope",
                scope.as_str()
            );
        }
    }

    #[test]
    fn test_is_standard_rejects_unknown_scope() {
        use super::OidcScope;
        assert!(!OidcScope::is_standard("custom_scope"));
        assert!(!OidcScope::is_standard("read:data"));
        assert!(!OidcScope::is_standard(""));
        assert!(!OidcScope::is_standard("PROFILE")); // case-sensitive
    }

    #[test]
    fn test_offline_access_is_standard_scope() {
        // offline_access is a standard OIDC scope — the client_credentials guard
        // must be enforced explicitly in the grant handler, not here.
        use super::OidcScope;
        assert!(OidcScope::is_standard(super::SCOPE_OFFLINE_ACCESS));
    }

    #[test]
    fn test_validate_and_filter_drops_unknown_scope() {
        let manager = ScopeManager::new();
        // Unknown scope is silently dropped by validate_and_filter (used for
        // authorize endpoint); the token endpoint uses resolve_scopes_for_client
        // which errors instead.
        let result = manager.validate_and_filter(Some("profile unknown_custom_scope".to_string()));
        assert!(result.contains("profile"));
        assert!(!result.contains("unknown_custom_scope"));
    }

    #[test]
    fn test_with_custom_scopes_allows_registered_custom_scope() {
        let manager = ScopeManager::new().with_custom_scopes(vec!["my_api".to_string()]);
        let result = manager.validate_and_filter(Some("profile my_api".to_string()));
        assert!(result.contains("profile"));
        assert!(result.contains("my_api"));
    }

    #[test]
    fn test_with_custom_scopes_still_rejects_unregistered_scope() {
        let manager = ScopeManager::new().with_custom_scopes(vec!["my_api".to_string()]);
        let result = manager.validate_and_filter(Some("profile other_scope".to_string()));
        assert!(result.contains("profile"));
        assert!(!result.contains("other_scope"));
    }
}
