use crate::application::common::services::DefaultUserService;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::{Realm, RealmError};
use crate::domain::user::ports::UserService;
use tracing::info;

#[derive(Clone)]
pub struct GetUserRealmsUseCase {
    user_service: DefaultUserService,
}

pub struct GetUserRealmsUseCaseParams {
    pub realm_name: String,
}

impl GetUserRealmsUseCase {
    pub fn new(user_service: DefaultUserService) -> Self {
        Self { user_service }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: GetUserRealmsUseCaseParams,
    ) -> Result<Vec<Realm>, RealmError> {
        info!("Getting user realms for user: {}", params.realm_name);
        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self
                .user_service
                .get_by_client_id(client.id)
                .await
                .map_err(|_| RealmError::Forbidden)?,
        };

        let realm = user.realm.clone().ok_or(RealmError::Forbidden)?;

        let realms = self
            .user_service
            .get_user_realms(user, realm.name)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        Ok(realms)
    }
}
