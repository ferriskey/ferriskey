use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::{
            entities::app_errors::CoreError,
            pagination::{Page, PageRequest},
        },
        portal_theme::ports::{GetThemeInput, PortalThemeService},
        realm::{
            entities::{
                AuthorizationServerSettings, Realm, RealmFilter, RealmId, RealmLoginSetting,
                RealmSetting, RealmSortField, ResourceOwner,
            },
            ports::{
                CreateRealmInput, CreateRealmWithUserInput, DeleteRealmInput, GetRealmInput,
                GetRealmSettingInput, ListResourceOwnersInput, RealmService,
                ReplaceResourceOwnersInput, ResourceOwnerService, UpdateRealmInput,
                UpdateRealmSettingInput,
            },
        },
    },
};

impl RealmService for ApplicationService {
    async fn create_realm(
        &self,
        identity: Identity,
        input: CreateRealmInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service.create_realm(identity, input).await
    }

    async fn create_realm_with_user(
        &self,
        identity: Identity,
        input: CreateRealmWithUserInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service
            .create_realm_with_user(identity, input)
            .await
    }

    async fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> Result<(), CoreError> {
        self.realm_service.delete_realm(identity, input).await
    }

    async fn get_login_settings(&self, realm_name: String) -> Result<RealmLoginSetting, CoreError> {
        let mut settings = self
            .realm_service
            .get_login_settings(realm_name.clone())
            .await?;
        settings.theme = self
            .portal_theme_service
            .get_public_theme(GetThemeInput { realm_name })
            .await?;
        Ok(settings)
    }

    async fn get_authorization_server_settings(
        &self,
        realm_name: String,
    ) -> Result<AuthorizationServerSettings, CoreError> {
        self.realm_service
            .get_authorization_server_settings(realm_name)
            .await
    }

    async fn get_realm_by_name(
        &self,
        identity: Identity,
        input: GetRealmInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service.get_realm_by_name(identity, input).await
    }

    async fn get_realm_setting_by_name(
        &self,
        identity: Identity,
        input: GetRealmSettingInput,
    ) -> Result<RealmSetting, CoreError> {
        self.realm_service
            .get_realm_setting_by_name(identity, input)
            .await
    }

    async fn list_user_realms(
        &self,
        identity: Identity,
        request: PageRequest<RealmFilter, RealmSortField>,
    ) -> Result<Page<Realm>, CoreError> {
        self.realm_service.list_user_realms(identity, request).await
    }

    async fn update_realm(
        &self,
        identity: Identity,
        input: UpdateRealmInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service.update_realm(identity, input).await
    }

    async fn update_realm_setting(
        &self,
        identity: Identity,
        input: UpdateRealmSettingInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service
            .update_realm_setting(identity, input)
            .await
    }

    async fn seed_default_scopes(&self, realm_id: RealmId) -> Result<(), CoreError> {
        self.realm_service.seed_default_scopes(realm_id).await
    }
}

impl ResourceOwnerService for ApplicationService {
    async fn list_resource_owners(
        &self,
        identity: Identity,
        input: ListResourceOwnersInput,
    ) -> Result<Vec<ResourceOwner>, CoreError> {
        self.resource_owner_service
            .list_resource_owners(identity, input)
            .await
    }

    async fn replace_resource_owners(
        &self,
        identity: Identity,
        input: ReplaceResourceOwnersInput,
    ) -> Result<Vec<ResourceOwner>, CoreError> {
        self.resource_owner_service
            .replace_resource_owners(identity, input)
            .await
    }
}
