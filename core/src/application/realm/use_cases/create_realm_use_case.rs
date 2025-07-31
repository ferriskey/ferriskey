use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService,
};
use crate::application::realm::policies::RealmPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::{Realm, RealmError};
use crate::domain::realm::ports::RealmService;
use crate::domain::user::ports::UserService;

#[derive(Clone)]
pub struct CreateRealmUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
}

pub struct CreateRealmUseCaseParams {
    pub realm_name: String,
}

impl CreateRealmUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: CreateRealmUseCaseParams,
    ) -> Result<Realm, RealmError> {
        let realm_master = self.realm_service.get_by_name("master".to_string()).await?;

        ensure_permissions(
            RealmPolicy::create(
                identity.clone(),
                realm_master,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(|e| anyhow::Error::new(e)),
            "Insufficient permissions to create a realm",
        )
        .map_err(|_| RealmError::Forbidden)?;

        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self
                .user_service
                .get_by_client_id(client.id)
                .await
                .map_err(|_| RealmError::InternalServerError)?,
        };

        self.realm_service
            .create_realm_with_user(params.realm_name, &user)
            .await
    }
}
