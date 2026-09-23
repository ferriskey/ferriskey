use ferriskey_domain::{realm::RealmSetting, role::entities::Role};

use crate::mfa_policy::user_requires_mfa;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountFactors {
    pub has_password: bool,
    pub passkey_count: usize,
    pub has_otp: bool,
    pub federated_identity_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactorRemoval {
    Otp,
    Passkey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalRefusal {
    LastSignInMeans,
    MfaRequired,
}

impl AccountFactors {
    fn after(self, removal: FactorRemoval) -> Self {
        match removal {
            FactorRemoval::Otp => Self {
                has_otp: false,
                ..self
            },
            FactorRemoval::Passkey => Self {
                passkey_count: self.passkey_count.saturating_sub(1),
                ..self
            },
        }
    }

    fn keeps_a_sign_in_means(&self) -> bool {
        self.has_password || self.passkey_count > 0 || self.federated_identity_count > 0
    }

    fn keeps_a_second_factor(&self) -> bool {
        self.has_otp || self.passkey_count > 0
    }
}

pub fn refusal_for_removal(
    factors: AccountFactors,
    removal: FactorRemoval,
    settings: Option<&RealmSetting>,
    roles: &[Role],
) -> Option<RemovalRefusal> {
    let remaining = factors.after(removal);

    if !remaining.keeps_a_sign_in_means() {
        return Some(RemovalRefusal::LastSignInMeans);
    }

    if user_requires_mfa(settings, roles) && !remaining.keeps_a_second_factor() {
        return Some(RemovalRefusal::MfaRequired);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ferriskey_domain::realm::RealmId;
    use uuid::Uuid;

    fn settings(require_mfa: bool) -> RealmSetting {
        let mut settings = RealmSetting::new(RealmId::new(Uuid::new_v4()), None);
        settings.require_mfa = require_mfa;
        settings
    }

    fn role(require_mfa: bool) -> Role {
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

    fn factors() -> AccountFactors {
        AccountFactors {
            has_password: false,
            passkey_count: 0,
            has_otp: false,
            federated_identity_count: 0,
        }
    }

    #[test]
    fn removing_the_only_passkey_of_a_passwordless_account_is_refused() {
        let account = AccountFactors {
            passkey_count: 1,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Passkey, None, &[]),
            Some(RemovalRefusal::LastSignInMeans)
        );
    }

    #[test]
    fn removing_a_passkey_is_allowed_while_a_password_remains() {
        let account = AccountFactors {
            has_password: true,
            passkey_count: 1,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Passkey, None, &[]),
            None
        );
    }

    #[test]
    fn removing_a_passkey_is_allowed_while_another_passkey_remains() {
        let account = AccountFactors {
            passkey_count: 2,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Passkey, None, &[]),
            None
        );
    }

    #[test]
    fn a_linked_identity_provider_counts_as_a_sign_in_means() {
        let account = AccountFactors {
            passkey_count: 1,
            federated_identity_count: 1,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Passkey, None, &[]),
            None
        );
    }

    #[test]
    fn removing_the_only_otp_of_an_account_with_nothing_else_is_refused_as_lockout() {
        let account = AccountFactors {
            has_otp: true,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Otp, None, &[]),
            Some(RemovalRefusal::LastSignInMeans)
        );
    }

    #[test]
    fn removing_otp_is_allowed_when_the_realm_does_not_require_mfa() {
        let account = AccountFactors {
            has_password: true,
            has_otp: true,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Otp, Some(&settings(false)), &[]),
            None
        );
    }

    #[test]
    fn removing_the_last_second_factor_is_refused_when_the_realm_requires_mfa() {
        let account = AccountFactors {
            has_password: true,
            has_otp: true,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Otp, Some(&settings(true)), &[]),
            Some(RemovalRefusal::MfaRequired)
        );
    }

    #[test]
    fn removing_the_last_second_factor_is_refused_when_a_role_requires_mfa() {
        let account = AccountFactors {
            has_password: true,
            has_otp: true,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(
                account,
                FactorRemoval::Otp,
                Some(&settings(false)),
                &[role(true)]
            ),
            Some(RemovalRefusal::MfaRequired)
        );
    }

    #[test]
    fn removing_otp_is_allowed_under_mfa_enforcement_while_a_passkey_remains() {
        let account = AccountFactors {
            has_password: true,
            passkey_count: 1,
            has_otp: true,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Otp, Some(&settings(true)), &[]),
            None
        );
    }

    #[test]
    fn removing_the_last_passkey_under_mfa_enforcement_is_refused_for_mfa_not_for_lockout() {
        let account = AccountFactors {
            has_password: true,
            passkey_count: 1,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Passkey, Some(&settings(true)), &[]),
            Some(RemovalRefusal::MfaRequired)
        );
    }

    #[test]
    fn lockout_outranks_mfa_when_both_would_refuse() {
        let account = AccountFactors {
            passkey_count: 1,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Passkey, Some(&settings(true)), &[]),
            Some(RemovalRefusal::LastSignInMeans)
        );
    }

    #[test]
    fn removing_an_absent_passkey_cannot_underflow_the_count() {
        let account = AccountFactors {
            has_password: true,
            ..factors()
        };

        assert_eq!(
            refusal_for_removal(account, FactorRemoval::Passkey, None, &[]),
            None
        );
    }
}
