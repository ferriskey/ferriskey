use crate::domain::{
    client::{ports::ClientRepository, value_objects::CreateClientRequest},
    common::generate_random_string,
    realm::{
        entities::{Realm, RealmError, RealmSetting},
        ports::{RealmRepository, RealmService},
    },
    role::{
        entities::permission::Permissions, ports::RoleRepository, value_objects::CreateRoleRequest,
    },
    user::{
        entities::User,
        ports::{UserRepository, UserRoleRepository},
    },
};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
pub struct RealmServiceImpl<R, C, RO, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    pub realm_repository: R,
    pub client_repository: C,
    pub role_repository: RO,
    pub user_repository: U,
    pub user_role_repository: UR,
}

impl<R, C, RO, U, UR> RealmServiceImpl<R, C, RO, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    pub fn new(
        realm_repository: R,
        client_repository: C,
        role_repository: RO,
        user_repository: U,
        user_role_repository: UR,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            role_repository,
            user_repository,
            user_role_repository,
        }
    }
}

impl<R, C, RO, U, UR> RealmService for RealmServiceImpl<R, C, RO, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    async fn fetch_realm(&self) -> Result<Vec<Realm>, RealmError> {
        self.realm_repository.fetch_realm().await
    }

    async fn create_realm_with_user(&self, name: String, user: &User) -> Result<Realm, RealmError> {
        let realm = self.realm_repository.create_realm(name.clone()).await?;
        self.realm_repository
            .create_realm_settings(realm.id, "RS256".to_string())
            .await?;

        let realm_master = self.realm_repository.get_by_name("master".into()).await?;

        // If the master realm does not exist, delete the created realm
        if realm_master.is_none() {
            error!("master realm not found, deleting created realm");
            self.realm_repository.delete_by_name(name).await?;
            return Err(RealmError::InternalServerError);
        }

        let realm_master = realm_master.ok_or(RealmError::InternalServerError)?;

        // Create client for realm master
        let client_id = format!("{name}-realm");

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
                client_type: "public".into(),
            })
            .await
            .map_err(|_| RealmError::InternalServerError)?;

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
                name: format!("{name}-realm-admin"),
                permissions,
                realm_id: realm_master.id,
                description: Some(format!("role for manage realm {name}")),
            })
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        Ok(realm)
    }

    async fn create_realm(&self, name: String) -> Result<Realm, RealmError> {
        let realm = self.realm_repository.create_realm(name.clone()).await?;
        info!("Created realm: {:?}", realm);
        self.realm_repository
            .create_realm_settings(realm.id, "RS256".to_string())
            .await?;

        Ok(realm)
    }

    async fn update_realm(&self, realm_name: String, name: String) -> Result<Realm, RealmError> {
        self.realm_repository.update_realm(realm_name, name).await
    }

    async fn delete_by_name(&self, name: String) -> Result<(), RealmError> {
        let realm = self.get_by_name(name.clone()).await.map_err(|_| {
            error!("realm {} not found", name);
            RealmError::Forbidden
        })?;

        if !realm.can_delete() {
            error!("try to delete master realm");
            return Err(RealmError::CannotDeleteMaster);
        }
        self.realm_repository.delete_by_name(name).await
    }

    async fn get_by_name(&self, name: String) -> Result<Realm, RealmError> {
        self.realm_repository
            .get_by_name(name)
            .await?
            .ok_or(RealmError::NotFound)
    }

    async fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, RealmError> {
        self.realm_repository
            .update_realm_setting(realm_id, algorithm)
            .await
    }
}
