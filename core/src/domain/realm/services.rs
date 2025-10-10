use std::collections::HashSet;

use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::ports::KeyStoreRepository,
    realm::{
        entities::Realm,
        ports::{CreateRealmInput, RealmPolicy, RealmRepository, RealmService},
    },
    role::{entities::permission::Permissions, ports::RoleRepository},
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::{WebhookNotifierRepository, WebhookRepository},
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN> RealmService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    WN: WebhookNotifierRepository,
{
    async fn create_realm(
        &self,
        identity: Identity,
        input: CreateRealmInput,
    ) -> Result<Realm, CoreError> {
        let realm_master = self
            .realm_repository
            .get_by_name("master".to_string())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_master_id = realm_master.id;
        todo!()
    }

    async fn create_realm_with_user(
        &self,
        identity: Identity,
        input: super::ports::CreateRealmWithUserInput,
    ) -> Result<Realm, CoreError> {
        todo!()
    }

    async fn delete_realm(
        &self,
        identity: Identity,
        input: super::ports::DeleteRealmInput,
    ) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_realm_by_name(
        &self,
        identity: Identity,
        input: super::ports::GetRealmInput,
    ) -> Result<Realm, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_realm(identity, realm.clone()).await,
            "insufficient permissions",
        )?;

        Ok(realm)
    }

    async fn get_realm_setting_by_name(
        &self,
        identity: Identity,
        input: super::ports::GetRealmSettingInput,
    ) -> Result<super::entities::RealmSetting, CoreError> {
        todo!()
    }

    async fn get_realms_by_user(&self, identity: Identity) -> Result<Vec<Realm>, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self.user_repository.get_by_client_id(client.id).await?,
        };

        let realm = user.realm.clone().ok_or(CoreError::InternalServerError)?;
        self.realm_repository
            .get_by_name(realm.name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let user_roles = self.user_role_repository.get_user_roles(user.id).await?;

        let realms = self.realm_repository.fetch_realm().await?;

        let mut user_realms: Vec<Realm> = Vec::new();

        for realm in realms {
            let client_name = format!("{}-realm", realm.name);

            let client_roles = user_roles
                .iter()
                .filter(|role| role.client.is_some())
                .filter(|role| role.client.as_ref().unwrap().name == client_name)
                .collect::<Vec<_>>();

            let mut permissions = HashSet::new();

            for role in client_roles {
                let role_permissions = role
                    .permissions
                    .iter()
                    .filter_map(|perm_str| Permissions::from_name(perm_str))
                    .collect::<HashSet<Permissions>>();

                permissions.extend(role_permissions);
            }

            let has_access = Permissions::has_one_of_permissions(
                &permissions.iter().cloned().collect::<Vec<Permissions>>(),
                &[
                    Permissions::QueryRealms,
                    Permissions::ManageRealm,
                    Permissions::ViewRealm,
                ],
            );

            if has_access {
                user_realms.push(realm.clone());
            }
        }

        Ok(user_realms)
    }

    async fn update_realm(
        &self,
        identity: Identity,
        input: super::ports::UpdateRealmInput,
    ) -> Result<Realm, CoreError> {
        todo!()
    }

    async fn update_realm_setting(
        &self,
        identity: Identity,
        input: super::ports::UpdateRealmSettingInput,
    ) -> Result<super::entities::RealmSetting, CoreError> {
        todo!()
    }
}
