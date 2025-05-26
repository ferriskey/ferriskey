use std::collections::HashSet;

use crate::{
    application::auth::Identity,
    domain::{
        client::entities::model::Client,
        role::entities::{
            models::Role,
            permission::{Permissions},
        },
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
            let permissions_bits = role.permissions as u64;

            let perm_values = Permissions::from_bitfield(permissions_bits);

            for p in perm_values {
                permissions.insert(p);
            }
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
            let permissions_bits = role.permissions as u64;

            let perm_values = Permissions::from_bitfield(permissions_bits);

            for p in perm_values {
                permissions.insert(p);
            }
        }

        Ok(permissions)
    }
}
