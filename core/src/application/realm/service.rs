use std::collections::HashSet;

use anyhow::Ok;

use crate::{
    application::common::{FerriskeyService, policies::ensure_policy},
    domain::{
        authentication::value_objects::Identity,
        client::{ports::ClientRepository, value_objects::CreateClientRequest},
        common::{entities::app_errors::CoreError, generate_random_string},
        realm::{
            entities::{Realm, RealmSetting},
            ports::{
                CreateRealmInput, CreateRealmWithUserInput, DeleteRealmInput, GetRealmInput,
                GetRealmSettingInput, RealmPolicy, RealmRepository, RealmService, UpdateRealmInput,
                UpdateRealmSettingInput,
            },
        },
        role::{
            entities::permission::Permissions, ports::RoleRepository,
            value_objects::CreateRoleRequest,
        },
        user::ports::{UserRepository, UserRoleRepository, UserService},
    },
};

impl RealmService for FerriskeyService {
    async fn get_realms_by_user(&self, identity: Identity) -> Result<Vec<Realm>, CoreError> {
        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self
                .get_by_client_id(client.id)
                .await
                .map_err(|_| CoreError::Forbidden)?,
        };

        let realm = user.realm.clone().ok_or(CoreError::Forbidden)?;
        let realm = self
            .realm_repository
            .get_by_name(realm.name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let user_roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let realms = self
            .realm_repository
            .fetch_realm()
            .await
            .map_err(|_| CoreError::InternalServerError)?;

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

    async fn get_realm_by_name(
        &self,
        identity: Identity,
        input: GetRealmInput,
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
        input: GetRealmSettingInput,
    ) -> Result<RealmSetting, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id.clone();
        ensure_policy(
            self.policy.can_view_realm(identity, realm.clone()).await,
            "insufficient permissions",
        )?;

        let realm_setting = self
            .realm_repository
            .get_realm_settings(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(realm_setting)
    }

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

        ensure_policy(
            self.policy.can_create_realm(identity, realm_master).await,
            "insufficient permissions",
        )?;

        let realm = self.realm_repository.create_realm(input.realm_name).await?;
        self.realm_repository
            .create_realm_settings(realm.id.clone(), "RS256".to_string())
            .await?;

        Ok(realm)
    }

    async fn create_realm_with_user(
        &self,
        identity: Identity,
        input: CreateRealmWithUserInput,
    ) -> Result<Realm, CoreError> {
        let realm = self
            .create_realm(
                identity,
                CreateRealmInput {
                    realm_name: input.realm_name,
                },
            )
            .await?;

        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self
                .user_repository
                .get_by_client_id(client.id)
                .await
                .map_err(|_| CoreError::InternalServerError)?,
        };

        let realm_master = self
            .realm_repository
            .get_by_name("master".to_string())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client_id = format!("{}-realm", input.realm_name);
        let client = self
            .client_repository
            .create_client(CreateClientRequest {
                realm_id: realm_master.id,
                name: client_id.clone(),
                client_id,
                secret: Some(generate_random_string()),
                enabled: true,
                protocol: "openid-connect".to_string(),
                public_client: true,
                service_account_enabled: false,
                direct_access_grants_enabled: false,
                client_type: "public".into(),
            })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        // Create role for client
        let permissions = Permissions::to_names(&[
            Permissions::ManageRealm,
            Permissions::ManageClients,
            Permissions::ManageRoles,
            Permissions::ManageUsers,
        ]);

        let role = self
            .role_repository
            .create(CreateRoleRequest {
                client_id: Some(client.id),
                name: format!("{}-realm-admin", input.realm_name),
                permissions,
                realm_id: realm_master.id,
                description: Some(format!("role for manage realm {}", input.realm_name)),
            })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(realm)
    }

    async fn update_realm(
        &self,
        identity: Identity,
        input: UpdateRealmInput,
    ) -> Result<Realm, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_realm(identity, realm).await,
            "insufficient permissions",
        )?;

        let realm = self
            .realm_repository
            .update_realm(input.realm_name, input.name)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(realm)
    }

    async fn update_realm_setting(
        &self,
        identity: Identity,
        input: UpdateRealmSettingInput,
    ) -> Result<RealmSetting, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_realm(identity, realm).await,
            "insufficient permissions",
        )?;

        let realm_setting = self
            .realm_repository
            .update_realm_setting(realm_id, input.algorithm)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(realm_setting)
    }

    async fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_realm(identity, realm).await,
            "insufficient permissions",
        )?;

        self.realm_repository
            .delete_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }
}
