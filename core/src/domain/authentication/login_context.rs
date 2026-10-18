use crate::domain::client::entities::Client;
use crate::domain::credential::entities::{Credential, CredentialType};
use crate::domain::realm::entities::RealmSetting;
use crate::domain::role::entities::Role;
use crate::domain::trident::mfa_policy::{self, PendingAuthStep};
use crate::domain::user::entities::User;
use uuid::Uuid;

pub const FEDERATED_CREDENTIAL: &str = "federated";

#[derive(Debug)]
pub struct LoginContext<'c> {
    pub client: &'c Client,
    pub user: User,
    pub federated_by: Option<Uuid>,
    pub credentials: Vec<Credential>,
}

impl LoginContext<'_> {
    pub fn is_federated(&self) -> bool {
        self.federated_by.is_some()
    }

    pub fn password_credential(&self) -> Option<&Credential> {
        self.credentials
            .iter()
            .find(|credential| credential.credential_type == CredentialType::Password)
    }

    pub fn has_otp_credential(&self) -> bool {
        self.credentials
            .iter()
            .any(|credential| credential.credential_type == CredentialType::Otp)
    }

    pub fn has_temporary_password(&self) -> bool {
        self.credentials
            .iter()
            .any(|credential| credential.temporary)
    }

    pub fn credential_names(&self) -> Vec<String> {
        let local = self
            .credentials
            .iter()
            .map(|credential| credential.credential_type.to_string());

        if self.is_federated() {
            return std::iter::once(FEDERATED_CREDENTIAL.to_string())
                .chain(local)
                .collect();
        }

        local.collect()
    }

    pub fn pending_step(
        &self,
        settings: Option<&RealmSetting>,
        roles: &[Role],
    ) -> Option<PendingAuthStep> {
        mfa_policy::pending_auth_step(
            &self.user.required_actions,
            settings,
            roles,
            self.has_otp_credential(),
            self.has_temporary_password(),
        )
    }
}

#[derive(Debug)]
pub struct VerifiedLogin<'c> {
    pub context: LoginContext<'c>,
    pub roles: Vec<Role>,
    pub pending_step: Option<PendingAuthStep>,
}

impl VerifiedLogin<'_> {
    pub fn otp_challenge_follows(&self) -> bool {
        matches!(self.pending_step, Some(PendingAuthStep::OtpChallenge))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::{Client, ClientConfig, ClientType};
    use crate::domain::credential::entities::{Credential, CredentialData, CredentialType};
    use crate::domain::realm::entities::{RealmId, RealmSetting};
    use crate::domain::trident::mfa_policy::PendingAuthStep;
    use crate::domain::user::entities::{RequiredAction, User, UserConfig};

    use super::{FEDERATED_CREDENTIAL, LoginContext, VerifiedLogin};

    fn client(realm_id: RealmId) -> Client {
        Client::new(ClientConfig {
            realm_id,
            name: "console".to_string(),
            client_id: "console".to_string(),
            secret: None,
            enabled: true,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: true,
            service_account_enabled: false,
            client_type: ClientType::Public,
            direct_access_grants_enabled: None,
            oauth_device_code_grant_enabled: None,
            token_exchange_enabled: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
        })
    }

    fn user(realm_id: RealmId, required_actions: Vec<RequiredAction>) -> User {
        let mut user = User::new(UserConfig {
            id: None,
            realm_id,
            client_id: None,
            username: "alice".to_string(),
            firstname: None,
            lastname: None,
            email: None,
            email_verified: true,
            enabled: true,
        });
        user.required_actions = required_actions;
        user
    }

    fn credential(user_id: Uuid, credential_type: CredentialType, temporary: bool) -> Credential {
        Credential {
            id: Uuid::new_v4(),
            salt: None,
            credential_type,
            user_id,
            user_label: None,
            secret_data: String::new(),
            credential_data: CredentialData::new_hash(1, "argon2".to_string()),
            temporary,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webauthn_credential_id: None,
        }
    }

    fn context(
        client: &Client,
        user: User,
        federated_by: Option<Uuid>,
        credentials: Vec<Credential>,
    ) -> LoginContext<'_> {
        LoginContext {
            client,
            user,
            federated_by,
            credentials,
        }
    }

    #[test]
    fn a_federated_login_without_local_credentials_owes_no_step() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client = client(realm_id);
        let user = user(realm_id, Vec::new());
        let context = context(&client, user, Some(Uuid::new_v4()), Vec::new());

        assert!(context.is_federated());
        assert_eq!(context.credential_names(), vec![FEDERATED_CREDENTIAL]);
        assert!(context.password_credential().is_none());
        assert!(!context.has_otp_credential());
        assert!(context.pending_step(None, &[]).is_none());
    }

    #[test]
    fn a_federated_login_keeps_its_local_credentials() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client = client(realm_id);
        let user = user(realm_id, Vec::new());
        let user_id = user.id;
        let context = context(
            &client,
            user,
            Some(Uuid::new_v4()),
            vec![credential(user_id, CredentialType::Otp, false)],
        );

        assert_eq!(
            context.credential_names(),
            vec![FEDERATED_CREDENTIAL, "otp"]
        );
        assert!(context.has_otp_credential());
        assert!(!context.has_temporary_password());
        assert_eq!(
            context.pending_step(None, &[]),
            Some(PendingAuthStep::OtpChallenge)
        );
    }

    #[test]
    fn a_temporary_local_password_binds_a_federated_login_like_any_other() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client = client(realm_id);
        let user = user(realm_id, Vec::new());
        let user_id = user.id;
        let context = context(
            &client,
            user,
            Some(Uuid::new_v4()),
            vec![
                credential(user_id, CredentialType::Password, true),
                credential(user_id, CredentialType::Otp, false),
            ],
        );

        assert!(context.has_temporary_password());
        assert_eq!(
            context.pending_step(None, &[]),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::UpdatePassword,
            ]))
        );
    }

    #[test]
    fn the_password_credential_is_found_among_the_others() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client = client(realm_id);
        let user = user(realm_id, Vec::new());
        let user_id = user.id;
        let context = context(
            &client,
            user,
            None,
            vec![
                credential(user_id, CredentialType::Otp, false),
                credential(user_id, CredentialType::Password, false),
            ],
        );

        assert_eq!(
            context
                .password_credential()
                .map(|credential| credential.credential_type.clone()),
            Some(CredentialType::Password)
        );
        assert_eq!(context.credential_names(), vec!["otp", "password"]);
        assert!(!context.has_temporary_password());
    }

    #[test]
    fn an_otp_credential_without_pending_actions_leads_to_the_otp_challenge() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client = client(realm_id);
        let user = user(realm_id, Vec::new());
        let user_id = user.id;
        let context = context(
            &client,
            user,
            None,
            vec![
                credential(user_id, CredentialType::Password, false),
                credential(user_id, CredentialType::Otp, false),
            ],
        );

        let verified = VerifiedLogin {
            pending_step: context.pending_step(Some(&RealmSetting::new(realm_id, None)), &[]),
            context,
            roles: Vec::new(),
        };

        assert_eq!(verified.pending_step, Some(PendingAuthStep::OtpChallenge));
        assert!(verified.otp_challenge_follows());
    }

    #[test]
    fn a_temporary_password_adds_update_password_to_the_persisted_actions() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client = client(realm_id);
        let user = user(realm_id, vec![RequiredAction::VerifyEmail]);
        let user_id = user.id;
        let context = context(
            &client,
            user,
            None,
            vec![
                credential(user_id, CredentialType::Password, true),
                credential(user_id, CredentialType::Otp, false),
            ],
        );

        assert!(context.has_temporary_password());
        assert_eq!(
            context.pending_step(None, &[]),
            Some(PendingAuthStep::RequiredActions(vec![
                RequiredAction::VerifyEmail,
                RequiredAction::UpdatePassword,
            ]))
        );
    }

    #[test]
    fn a_plain_password_login_owes_no_step() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client = client(realm_id);
        let user = user(realm_id, Vec::new());
        let user_id = user.id;
        let context = context(
            &client,
            user,
            None,
            vec![credential(user_id, CredentialType::Password, false)],
        );

        let verified = VerifiedLogin {
            pending_step: context.pending_step(Some(&RealmSetting::new(realm_id, None)), &[]),
            context,
            roles: Vec::new(),
        };

        assert!(verified.pending_step.is_none());
        assert!(!verified.otp_challenge_follows());
    }
}
