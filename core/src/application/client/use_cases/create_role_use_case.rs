use uuid::Uuid;

use crate::{
    application::{
        common::services::{
            DefaultClientService, DefaultRealmService, DefaultRoleService, DefaultUserService,
        },
        role::policies::RolePolicy,
    },
    domain::{
        authentication::value_objects::Identity,
        client::entities::ClientError,
        realm::ports::RealmService,
        role::{entities::Role, ports::RoleService, value_objects::CreateRoleRequest},
    },
};

#[derive(Clone)]
pub struct CreateRoleUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    role_service: DefaultRoleService,
}

pub struct CreateRoleUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
    pub description: Option<String>,
    pub name: String,
    pub permissions: Vec<String>,
}

impl CreateRoleUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        role_service: DefaultRoleService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            role_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: CreateRoleUseCaseParams,
    ) -> Result<Role, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let realm_id = realm.id;
        Self::ensure_permissions(
            RolePolicy::create(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(|e| ClientError::Forbidden(e.to_string())),
            "Insufficient permissions to create a role",
        )?;

        let role = self
            .role_service
            .create(CreateRoleRequest {
                client_id: Some(params.client_id),
                description: params.description,
                name: params.name,
                permissions: params.permissions,
                realm_id,
            })
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        Ok(role)
    }

    #[inline]
    fn ensure_permissions(
        result_has_permission: Result<bool, ClientError>,
        error_message: &str,
    ) -> Result<(), ClientError> {
        result_has_permission
            .map_err(|_| ClientError::Forbidden(error_message.to_string()))?
            .then_some(())
            .ok_or_else(|| ClientError::Forbidden(error_message.to_string()))
    }
}
