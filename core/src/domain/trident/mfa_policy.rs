use ferriskey_domain::{realm::RealmSetting, role::entities::Role};

use crate::domain::user::entities::RequiredAction;

/// Returns `true` when the user is subject to MFA enforcement.
///
/// A user must use MFA when:
/// - the realm-level `require_mfa` flag is set, OR
/// - any of their assigned roles has `require_mfa` set.
pub fn user_requires_mfa(settings: Option<&RealmSetting>, roles: &[Role]) -> bool {
    settings.is_some_and(|s| s.require_mfa) || roles.iter().any(|r| r.require_mfa)
}

/// Given that the user is subject to MFA enforcement, decide the next action.
///
/// - If the user has **no** MFA credential yet → inject `ConfigureOtp` so the
///   login flow blocks until the user sets up an authenticator.
/// - If they already have an MFA credential the existing OTP-challenge gate in
///   `determine_next_step` handles prompting for the code; return `None` here.
pub fn required_action_for_mfa(has_mfa_credential: bool) -> Option<RequiredAction> {
    if has_mfa_credential {
        None
    } else {
        Some(RequiredAction::ConfigureOtp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ferriskey_domain::{
        realm::{RealmId, RealmSetting},
        role::entities::Role,
    };
    use uuid::Uuid;

    fn make_realm_setting(require_mfa: bool) -> RealmSetting {
        let id = RealmId::new(Uuid::new_v4());
        let mut s = RealmSetting::new(id, None);
        s.require_mfa = require_mfa;
        s
    }

    fn make_role(require_mfa: bool) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            description: None,
            permissions: vec![],
            realm_id: RealmId::new(Uuid::new_v4()),
            client_id: None,
            client: None,
            require_mfa,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn realm_require_mfa_true_triggers_enforcement() {
        let settings = make_realm_setting(true);
        assert!(user_requires_mfa(Some(&settings), &[]));
    }

    #[test]
    fn role_require_mfa_true_triggers_enforcement() {
        let settings = make_realm_setting(false);
        let role = make_role(true);
        assert!(user_requires_mfa(Some(&settings), &[role]));
    }

    #[test]
    fn both_false_no_enforcement() {
        let settings = make_realm_setting(false);
        let role = make_role(false);
        assert!(!user_requires_mfa(Some(&settings), &[role]));
    }

    #[test]
    fn no_settings_no_enforcement() {
        let role = make_role(false);
        assert!(!user_requires_mfa(None, &[role]));
    }

    #[test]
    fn no_settings_role_requires_mfa() {
        let role = make_role(true);
        assert!(user_requires_mfa(None, &[role]));
    }

    #[test]
    fn no_mfa_credential_returns_configure_otp() {
        let action = required_action_for_mfa(false);
        assert_eq!(action, Some(RequiredAction::ConfigureOtp));
    }

    #[test]
    fn has_mfa_credential_returns_none() {
        let action = required_action_for_mfa(true);
        assert_eq!(action, None);
    }
}
