use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::{Realm, RealmSetting},
    user::entities::User,
};

pub trait RealmService: Clone + Send + Sync {
    fn get_realms_by_user(
        &self,
        identity: Identity,
    ) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;

    fn get_realm_by_name(
        &self,
        identity: Identity,
        input: GetRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn get_realm_setting_by_name(
        &self,
        identity: Identity,
        input: GetRealmSettingInput,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn create_realm(
        &self,
        identity: Identity,
        input: CreateRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn create_realm_with_user(
        &self,
        identity: Identity,
        input: CreateRealmWithUserInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm(
        &self,
        identity: Identity,
        input: UpdateRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm_setting(
        &self,
        identity: Identity,
        input: UpdateRealmSettingInput,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait RealmPolicy: Send + Sync + Clone {
    fn can_create_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_delete_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_view_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_update_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub trait RealmRepository: Clone + Send + Sync + 'static {
    fn fetch_realm(&self) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;

    fn get_by_name(
        &self,
        name: String,
    ) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;

    fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm(
        &self,
        realm_name: String,
        name: String,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;
    fn delete_by_name(&self, name: String) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn create_realm_settings(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn get_realm_settings(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
}

pub struct GetRealmInput {
    pub realm_name: String,
}

pub struct GetRealmSettingInput {
    pub realm_name: String,
}

pub struct CreateRealmInput {
    pub realm_name: String,
}

pub struct CreateRealmWithUserInput {
    pub realm_name: String,
    pub user: User,
}

pub struct UpdateRealmInput {
    pub realm_name: String,
    pub name: String,
}

pub struct UpdateRealmSettingInput {
    pub realm_name: String,
    pub algorithm: String,
}

pub struct DeleteRealmInput {
    pub realm_name: String,
}

#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;
    mock! {
        pub RealmService {}
        impl Clone for RealmService {
            fn clone(&self) -> Self;
        }
        impl RealmService for RealmService {
            fn get_realms_by_user(&self, identity: Identity) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;
            fn get_realm_by_name(&self, identity: Identity, input: GetRealmInput) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn get_realm_setting_by_name(&self, identity: Identity, input: GetRealmSettingInput) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
            fn create_realm(&self, identity: Identity, input: CreateRealmInput) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn create_realm_with_user(&self, identity: Identity, input: CreateRealmWithUserInput) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn update_realm(&self, identity: Identity, input: UpdateRealmInput) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn update_realm_setting(&self, identity: Identity, input: UpdateRealmSettingInput) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
            fn delete_realm(&self, identity: Identity, input: DeleteRealmInput) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }
    pub fn get_mock_realm_service_with_clone_expectations() -> MockRealmService {
        let mut mock = MockRealmService::new();
        mock.expect_clone()
            .returning(|| get_mock_realm_service_with_clone_expectations());
        mock
    }
    
    mock! {
        pub RealmRepository {}
        impl Clone for RealmRepository {
            fn clone(&self) -> Self;
        }
        impl RealmRepository for RealmRepository {
            fn fetch_realm(&self) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;
            fn get_by_name(&self, name: String) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;
            fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn update_realm(&self, realm_name: String, name: String) -> impl Future<Output = Result<Realm, CoreError>> + Send;
            fn delete_by_name(&self, name: String) -> impl Future<Output = Result<(), CoreError>> + Send;
            fn create_realm_settings(&self, realm_id: Uuid, algorithm: String) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
            fn update_realm_setting(&self, realm_id: Uuid, algorithm: String) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
            fn get_realm_settings(&self, realm_id: Uuid) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;
        }
    }
    pub fn get_mock_realm_repository_with_clone_expectations() -> MockRealmRepository {
        let mut mock = MockRealmRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_realm_repository_with_clone_expectations());
        mock
    }
    
    mock! {
        pub RealmPolicy {}
        impl Clone for RealmPolicy {
            fn clone(&self) -> Self;
        }
        impl RealmPolicy for RealmPolicy {
            fn can_create_realm(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_delete_realm(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_view_realm(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_update_realm(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
        }
    }
    pub fn get_mock_realm_policy_with_clone_expectations() -> MockRealmPolicy {
        let mut mock = MockRealmPolicy::new();
        mock.expect_clone()
            .returning(|| get_mock_realm_policy_with_clone_expectations());
        mock
    }
}

