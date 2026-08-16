use ferriskey_domain::user::entities::RequiredAction;
use ferriskey_domain::{realm::RealmSetting, role::entities::Role};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAuthStep {
    RequiredActions(Vec<RequiredAction>),
    OtpChallenge,
}

pub fn pending_auth_step(
    persisted_actions: &[RequiredAction],
    settings: Option<&RealmSetting>,
    roles: &[Role],
    has_otp_credential: bool,
    has_temporary_password: bool,
) -> Option<PendingAuthStep> {
    let mut effective = persisted_actions.to_vec();

    if has_temporary_password && !effective.contains(&RequiredAction::UpdatePassword) {
        effective.push(RequiredAction::UpdatePassword);
    }

    if !has_temporary_password
        && user_requires_mfa(settings, roles)
        && let Some(action) = required_action_for_mfa(has_otp_credential)
        && !effective.contains(&action)
    {
        effective.push(action);
    }

    if !effective.is_empty() {
        return Some(PendingAuthStep::RequiredActions(effective));
    }

    if has_otp_credential {
        return Some(PendingAuthStep::OtpChallenge);
    }

    None
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

    #[test]
    fn no_pending_step_for_a_plain_user() {
        let settings = make_realm_setting(false);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(&[], Some(&settings), &[role], false, false),
            None
        );
    }

    #[test]
    fn an_otp_credential_alone_demands_a_challenge() {
        let settings = make_realm_setting(false);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(&[], Some(&settings), &[role], true, false),
            Some(PendingAuthStep::OtpChallenge)
        );
    }

    #[test]
    fn a_persisted_action_outranks_the_otp_challenge() {
        let settings = make_realm_setting(false);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(
                &[RequiredAction::UpdatePassword],
                Some(&settings),
                &[role],
                true,
                false
            ),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::UpdatePassword
            ]))
        );
    }

    #[test]
    fn realm_level_mfa_without_a_credential_demands_enrolment() {
        let settings = make_realm_setting(true);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(&[], Some(&settings), &[role], false, false),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::ConfigureOtp
            ]))
        );
    }

    #[test]
    fn role_level_mfa_without_a_credential_demands_enrolment() {
        let settings = make_realm_setting(false);
        let role = make_role(true);
        assert_eq!(
            pending_auth_step(&[], Some(&settings), &[role], false, false),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::ConfigureOtp
            ]))
        );
    }

    #[test]
    fn a_temporary_password_is_settled_before_mfa_enrolment() {
        let settings = make_realm_setting(true);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(
                &[RequiredAction::UpdatePassword],
                Some(&settings),
                &[role],
                false,
                true
            ),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::UpdatePassword
            ]))
        );
    }

    #[test]
    fn enrolment_is_not_demanded_twice() {
        let settings = make_realm_setting(true);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(
                &[RequiredAction::ConfigureOtp],
                Some(&settings),
                &[role],
                false,
                false
            ),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::ConfigureOtp
            ]))
        );
    }

    #[test]
    fn a_realm_without_settings_still_honours_role_level_mfa() {
        let role = make_role(true);
        assert_eq!(
            pending_auth_step(&[], None, &[role], false, false),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::ConfigureOtp
            ]))
        );
    }

    #[test]
    fn a_temporary_password_is_owed_even_when_nothing_was_persisted() {
        let settings = make_realm_setting(false);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(&[], Some(&settings), &[role], false, true),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::UpdatePassword
            ]))
        );
    }

    #[test]
    fn a_temporary_password_is_not_owed_twice() {
        let settings = make_realm_setting(false);
        let role = make_role(false);
        assert_eq!(
            pending_auth_step(
                &[RequiredAction::UpdatePassword],
                Some(&settings),
                &[role],
                false,
                true
            ),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::UpdatePassword
            ]))
        );
    }
}
