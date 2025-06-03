use crate::domain::client::entities::dto::CreateClientDto;
use crate::domain::client::ports::client_repository::ClientRepository;
use crate::domain::realm::entities::{
    error::RealmError, realm::Realm, realm_setting::RealmSetting,
};
use crate::domain::realm::ports::{realm_repository::RealmRepository, realm_service::RealmService};
use crate::domain::role::entities::CreateRoleDto;
use crate::domain::role::entities::permission::Permissions;
use crate::domain::role::ports::RoleRepository;
use crate::domain::user::entities::model::User;
use crate::domain::user::ports::user_repository::UserRepository;
use crate::domain::utils::generate_random_string;
use crate::infrastructure::repositories::client_repository::PostgresClientRepository;
use crate::infrastructure::repositories::realm_repository::PostgresRealmRepository;
use crate::infrastructure::repositories::role_repository::PostgresRoleRepository;
use crate::infrastructure::repositories::user_repository::PostgresUserRepository;
use tracing::{error, info};
use uuid::Uuid;

pub type DefaultRealmService = RealmServiceImpl<
    PostgresRealmRepository,
    PostgresClientRepository,
    PostgresRoleRepository,
    PostgresUserRepository,
>;

#[derive(Debug, Clone)]
pub struct RealmServiceImpl<R, C, RO, U>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
{
    pub realm_repository: R,
    pub client_repository: C,
    pub role_repository: RO,
    pub user_repository: U,
}

impl<R, C, RO, U> RealmServiceImpl<R, C, RO, U>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
{
    pub fn new(
        realm_repository: R,
        client_repository: C,
        role_repository: RO,
        user_repository: U,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            role_repository,
            user_repository,
        }
    }
}

impl<R, C, RO, U> RealmService for RealmServiceImpl<R, C, RO, U>
where
    R: RealmRepository,
    C: ClientRepository,
    RO: RoleRepository,
    U: UserRepository,
{
    async fn fetch_realm(&self) -> Result<Vec<Realm>, RealmError> {
        self.realm_repository.fetch_realm().await
    }

    async fn create_realm_with_user(&self, name: String, user: &User) -> Result<Realm, RealmError> {
        let realm = self.realm_repository.create_realm(name.clone()).await?;
        println!("Created realm: {:?}", realm);
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
        let client_id = format!("{}-realm", name);

        let client = self
            .client_repository
            .create_client(CreateClientDto {
                realm_id: realm_master.id,
                name: client_id.clone(),
                client_id,
                secret: generate_random_string(),
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
            .create(CreateRoleDto {
                client_id: Some(client.id),
                name: format!("{}-realm-admin", name),
                permissions,
                realm_id: realm_master.id,
                description: Some(format!("role for manage realm {}", name)),
            })
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        self.user_repository
            .assign_role_to_user(user.id, role.id)
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
