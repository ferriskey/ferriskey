use std::collections::HashSet;

use crate::{
    application::auth::Identity,
    domain::{
        client::{entities::model::Client, ports::client_service::ClientService},
        realm::entities::realm::Realm,
        role::entities::{models::Role, permission::Permissions},
        user::{entities::model::User, ports::user_service::UserService},
    },
};

use super::server::{api_entities::api_error::ApiError, app_state::AppState};

pub trait Policy: Send + Sync {
    // fn check
}

pub struct PolicyEnforcer {
    pub state: AppState,
}

impl PolicyEnforcer {
    pub fn new(state: AppState) -> Self {
        PolicyEnforcer { state }
    }

    pub async fn get_user_from_identity(&self, identity: &Identity) -> Result<User, ApiError> {
        match identity {
            Identity::User(user) => Ok(user.clone()),
            Identity::Client(client) => {
                let service_account = self
                    .state
                    .user_service
                    .get_by_client_id(client.id)
                    .await
                    .map_err(|_| ApiError::Forbidden("Client not found".to_string()))?;

                return Ok(service_account);
            }
        }
    }

    pub async fn get_user_permissions(
        &self,
        user: &User,
    ) -> Result<HashSet<Permissions>, ApiError> {
        let roles = self
            .state
            .user_service
            .get_user_roles(user.id)
            .await
            .map_err(|_| ApiError::Forbidden("User not found".to_string()))?;

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

    pub async fn get_client_specific_permissions(
        &self,
        user: &User,
        client: &Client,
    ) -> Result<HashSet<Permissions>, ApiError> {
        let roles = self
            .state
            .user_service
            .get_user_roles(user.id)
            .await
            .map_err(|_| ApiError::Forbidden("User not found".to_string()))?;

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

    pub async fn get_permission_for_target_realm(
        &self,
        user: &User,
        target_realm: &Realm,
    ) -> Result<HashSet<Permissions>, ApiError> {
        let user_realm = user
            .realm
            .as_ref()
            .ok_or(ApiError::Forbidden("User has no realm".into()))?;

        let mut permissions: HashSet<Permissions> = HashSet::new();

        if !self.can_access_realm(user_realm, target_realm) {
            return Ok(permissions);
        }

        if self.is_cross_realm_access(user_realm, target_realm) {
            let client_id = format!("{}-realm", target_realm.name);

            let client = self
                .state
                .client_service
                .get_by_client_id(client_id, user_realm.id)
                .await
                .map_err(|_| ApiError::Forbidden("Client not found for target realm".into()))?;

            let client_permissions = self.get_client_specific_permissions(&user, &client).await?;
            permissions.extend(client_permissions);
        } else {
            let user_permissions = self.get_user_permissions(&user).await?;
            permissions.extend(user_permissions);
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
