use anyhow::anyhow;
use std::collections::HashSet;
use thiserror::Error;

use crate::domain::{
    authentication::value_objects::Identity,
    client::{entities::Client, ports::OldClientService},
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    role::entities::{Role, permission::Permissions},
    user::{entities::User, ports::UserService},
};

#[derive(Debug, Clone, Error)]
pub enum PolicyError {
    #[error("User not found")]
    UserNotFound,

    #[error("{0}")]
    Forbidden(String),
}

pub struct PolicyEnforcer<U, C>
where
    U: UserService,
    C: OldClientService,
{
    user_service: U,
    client_service: C,
}

impl<U, C> PolicyEnforcer<U, C>
where
    U: UserService,
    C: OldClientService,
{
    pub fn new(user_service: U, client_service: C) -> Self {
        Self {
            user_service,
            client_service,
        }
    }

    pub async fn get_user_from_identity(&self, identity: &Identity) -> Result<User, PolicyError> {
        match identity {
            Identity::User(user) => Ok(user.clone()),
            Identity::Client(client) => {
                let service_account = self
                    .user_service
                    .get_by_client_id(client.id)
                    .await
                    .map_err(|e| PolicyError::Forbidden(e.to_string()))?;

                Ok(service_account)
            }
        }
    }

    pub async fn get_user_permissions(
        &self,
        user: &User,
    ) -> Result<HashSet<Permissions>, PolicyError> {
        let roles = self
            .user_service
            .get_user_roles(user.id)
            .await
            .map_err(|_| PolicyError::Forbidden("User not found".to_string()))?;

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
    ) -> Result<HashSet<Permissions>, PolicyError> {
        let roles = self
            .user_service
            .get_user_roles(user.id)
            .await
            .map_err(|_| PolicyError::Forbidden("User not found".to_string()))?;

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
    ) -> Result<HashSet<Permissions>, PolicyError> {
        let user_realm = user
            .realm
            .as_ref()
            .ok_or(PolicyError::Forbidden("User has no realm".into()))?;

        let mut permissions: HashSet<Permissions> = HashSet::new();

        if !self.can_access_realm(user_realm, target_realm) {
            return Ok(permissions);
        }

        if self.is_cross_realm_access(user_realm, target_realm) {
            let client_id = format!("{}-realm", target_realm.name);

            let client = self
                .client_service
                .get_by_client_id(client_id, user_realm.id)
                .await
                .map_err(|_| {
                    PolicyError::Forbidden("Cleint not found for target realm".to_string())
                })?;

            let client_permissions = self.get_client_specific_permissions(user, &client).await?;

            permissions.extend(client_permissions);
        } else {
            let user_permissions = self.get_user_permissions(user).await?;
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

pub fn ensure_policy(
    result_has_permission: Result<bool, CoreError>,
    error_message: &str,
) -> Result<(), CoreError> {
    match result_has_permission {
        Ok(true) => Ok(()),
        Ok(false) => Err(CoreError::Forbidden(error_message.to_string())),
        Err(_) => Err(CoreError::Forbidden(error_message.to_string())),
    }
}

pub fn ensure_permissions(
    result_has_permission: Result<bool, anyhow::Error>,
    error_message: &str,
) -> Result<(), anyhow::Error> {
    match result_has_permission {
        Ok(true) => Ok(()),
        Ok(false) => Err(anyhow!("{}", error_message)),
        Err(e) => Err(anyhow!("{}: {}", error_message, e)),
    }
}
