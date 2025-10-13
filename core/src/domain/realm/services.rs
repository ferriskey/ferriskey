use std::collections::HashSet;

use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::{
        ports::{ClientRepository, RedirectUriRepository},
        value_objects::CreateClientRequest,
    },
    common::{
        entities::app_errors::CoreError, generate_random_string, policies::ensure_policy,
        services::Service,
    },
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    realm::{
        entities::{Realm, RealmSetting},
        ports::{
            CreateRealmInput, CreateRealmWithUserInput, DeleteRealmInput, GetRealmInput,
            GetRealmSettingInput, RealmPolicy, RealmRepository, RealmService, UpdateRealmInput,
            UpdateRealmSettingInput,
        },
    },
    role::{
        entities::permission::Permissions, ports::RoleRepository, value_objects::CreateRoleRequest,
    },
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::{WebhookNotifierRepository, WebhookRepository},
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC> RealmService
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
        ensure_policy(
            self.policy
                .can_create_realm(identity.clone(), realm_master)
                .await,
            "insufficient permissions",
        )?;

        let realm = self.realm_repository.create_realm(input.realm_name).await?;
        self.realm_repository
            .create_realm_settings(realm.id, "RS256".to_string())
            .await?;

        let name = format!("{}-realm", realm.name);

        let client = self
            .client_repository
            .create_client(CreateClientRequest::create_realm_system_client(
                realm_master_id,
                name.clone(),
            ))
            .await?;

        let role = self
            .role_repository
            .create(CreateRoleRequest {
                client_id: Some(client.id),
                description: None,
                name,
                permissions: vec![Permissions::ManageRealm.name()],
                realm_id: realm_master_id,
            })
            .await?;

        let user = match identity {
            Identity::User(u) => u,
            Identity::Client(c) => self.user_repository.get_by_client_id(c.id).await?,
        };

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await?;

        // Clients in the new realm
        self.client_repository
            .create_client(CreateClientRequest {
                client_id: "admin-cli".to_string(),
                client_type: "".to_string(),
                direct_access_grants_enabled: true,
                enabled: true,
                name: "admin-cli".to_string(),
                protocol: "openid-connect".to_string(),
                public_client: true,
                realm_id: realm.id,
                secret: None,
                service_account_enabled: false,
            })
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
                identity.clone(),
                CreateRealmInput {
                    realm_name: input.realm_name.clone(),
                },
            )
            .await?;

        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self.user_repository.get_by_client_id(client.id).await?,
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
            .await?;

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
            .await?;

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await?;

        Ok(realm)
    }

    async fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_delete_realm(identity, realm).await,
            "insufficient permissions",
        )?;

        self.realm_repository
            .delete_by_name(input.realm_name)
            .await?;

        Ok(())
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
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_view_realm(identity, realm.clone()).await,
            "insufficient permissions",
        )?;

        let realm_setting = self.realm_repository.get_realm_settings(realm_id).await?;

        Ok(realm_setting)
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
        input: UpdateRealmInput,
    ) -> Result<Realm, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_realm(identity, realm).await,
            "insufficient permissions",
        )?;

        let realm = self
            .realm_repository
            .update_realm(input.realm_name, input.name)
            .await?;

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
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_realm(identity, realm).await,
            "insufficient permissions",
        )?;

        let realm_setting = self
            .realm_repository
            .update_realm_setting(realm_id, input.algorithm)
            .await?;

        Ok(realm_setting)
    }
}
