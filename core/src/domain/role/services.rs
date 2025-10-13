use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    realm::ports::RealmRepository,
    role::{
        entities::{GetUserRolesInput, Role, UpdateRoleInput},
        ports::{RolePolicy, RoleRepository, RoleService},
        value_objects::{UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::{WebhookNotifierRepository, WebhookRepository},
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC> RoleService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
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
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
{
    async fn delete_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_delete_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .delete_by_id(role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn get_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .get_by_id(role_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .ok_or(CoreError::NotFound)
    }

    async fn get_roles(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<Role>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .find_by_realm_id(realm_id)
            .await
            .map_err(|_| CoreError::NotFound)
    }

    async fn update_role(
        &self,
        identity: Identity,
        input: UpdateRoleInput,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_update_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .update_by_id(
                input.role_id,
                UpdateRoleRequest {
                    description: input.description,
                    name: input.name,
                },
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(role)
    }

    async fn update_role_permissions(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
        permissions: Vec<String>,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_update_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .update_permissions_by_id(role_id, UpdateRolePermissionsRequest { permissions })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(role)
    }

    async fn get_user_roles(
        &self,
        identity: Identity,
        input: GetUserRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.user_role_repository
            .get_user_roles(input.user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }
}
