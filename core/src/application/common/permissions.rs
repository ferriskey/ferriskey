use std::{collections::HashSet, hash::Hash};

use crate::{
    domain::{
        authentication::value_objects::Identity,
        client::{entities::Client, ports::ClientRepository},
        common::{entities::app_errors::CoreError, policies::Policy},
        realm::entities::Realm,
        role::entities::{Role, permission::Permissions},
        user::{
            entities::User,
            ports::{UserRepository, UserRoleRepository},
        },
    },
    infrastructure::{
        client::repositories::ClientRepoAny,
        user::{UserRepoAny, repositories::user_role_repository::UserRoleRepoAny},
    },
};

#[derive(Clone)]
pub struct FerriskeyPolicy {
    user_repository: UserRepoAny,
    client_repository: ClientRepoAny,
    user_role_repository: UserRoleRepoAny,
}

impl Policy for FerriskeyPolicy {
    async fn get_user_from_identity(&self, identity: &Identity) -> Result<User, CoreError> {
        match identity {
            Identity::User(user) => Ok(user.clone()),
            Identity::Client(client) => {
                let service_account = self
                    .user_repository
                    .get_by_client_id(client.id)
                    .await
                    .map_err(|e| CoreError::Forbidden(e.to_string()))?;

                Ok(service_account)
            }
        }
    }

    async fn get_client_specific_permissions(
        &self,
        user: &User,
        client: &Client,
    ) -> Result<HashSet<Permissions>, CoreError> {
        let roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .map_err(|_| CoreError::Forbidden("user not found".to_string()))?;

        let client_roles = roles
            .into_iter()
            .filter(|role| role.client_id == Some(client.id))
            .collect::<Vec<Role>>();

        let mut permissions: HashSet<Permissions> = HashSet::new();

        for role in client_roles {
            let role_permissions: HashSet<Permissions> = role
                .permissions
                .iter()
                .filter_map(|p| Permissions::from_name(p))
                .collect();

            let permissions_as_vec: Vec<Permissions> = role_permissions.into_iter().collect();
            let permissions_bits = Permissions::to_bitfield(&permissions_as_vec);
            let validated_permissions = Permissions::from_bitfield(permissions_bits);

            permissions.extend(validated_permissions);
        }

        Ok(permissions)
    }

    async fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> Result<HashSet<Permissions>, CoreError> {
        let user_realm = user
            .realm
            .as_ref()
            .ok_or(CoreError::Forbidden("user has no realm".to_string()))?;

        let mut permissions: HashSet<Permissions> = HashSet::new();

        if !self.can_access_realm(user_realm, target_realm) {
            return Ok(permissions);
        }

        if self.is_cross_realm_access(user_realm, target_realm) {
            let client_id = format!("{}-realm", target_realm.name);

            let client = self
                .client_repository
                .get_by_client_id(client_id, user_realm.id)
                .await
                .map_err(|_| {
                    CoreError::Forbidden("client not found for target realm".to_string())
                })?;

            let client_permissions = self.get_client_specific_permissions(user, &client).await?;

            permissions.extend(client_permissions);
        } else {
            let user_permissions = self.get_user_permissions(user).await?;
            permissions.extend(user_permissions);
        }

        Ok(permissions)
    }

    async fn get_user_permissions(&self, user: &User) -> Result<HashSet<Permissions>, CoreError> {
        let roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .map_err(|_| CoreError::Forbidden("user not found".to_string()))?;

        let mut permissions: HashSet<Permissions> = HashSet::new();

        for role in roles {
            let role_permissions: HashSet<Permissions> = role
                .permissions
                .iter()
                .filter_map(|p| Permissions::from_name(p))
                .collect();

            let permissions_as_vec: Vec<Permissions> = role_permissions.into_iter().collect();
            let permissions_bits = Permissions::to_bitfield(&permissions_as_vec);
            let validated_permissions = Permissions::from_bitfield(permissions_bits);

            permissions.extend(validated_permissions);
        }

        Ok(permissions)
    }

    fn can_access_realm(&self, user_realm: &Realm, target_realm: &Realm) -> bool {
        user_realm.name == target_realm.name || user_realm.name == "master"
    }

    fn is_cross_realm_access(&self, user_realm: &Realm, target_realm: &Realm) -> bool {
        user_realm.name == "master" && user_realm.name != target_realm.name
    }
}
