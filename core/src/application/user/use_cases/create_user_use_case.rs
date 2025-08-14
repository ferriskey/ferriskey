use crate::{
    application::{
        common::services::{DefaultClientService, DefaultRealmService, DefaultUserService},
        user::policies::user_policy::UserPolicy,
    },
    domain::{
        authentication::value_objects::Identity,
        realm::ports::RealmService,
        user::{
            entities::{User, UserError},
            ports::UserService,
            value_objects::CreateUserRequest,
        },
    },
};

#[derive(Debug, Clone)]
pub struct CreateUserUseCaseParams {
    pub realm_name: String,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: Option<bool>,
}

#[derive(Clone)]
pub struct CreateUserUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
}

impl CreateUserUseCase {
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
        params: CreateUserUseCaseParams,
    ) -> Result<User, UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let realm_id = realm.id;
        Self::ensure_permissions(
            UserPolicy::store(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to create a user",
        )?;

        let mut user = self
            .user_service
            .create_user(CreateUserRequest {
                client_id: None,
                realm_id,
                username: params.username,
                firstname: params.firstname,
                lastname: params.lastname,
                email: params.email,
                email_verified: params.email_verified.unwrap_or(false),
                enabled: true,
            })
            .await?;

        user.realm = Some(realm);

        Ok(user)
    }

    #[inline]
    fn ensure_permissions(
        result_has_permission: Result<bool, UserError>,
        error_message: &str,
    ) -> Result<(), UserError> {
        match result_has_permission {
            Ok(true) => Ok(()),
            _ => Err(UserError::Forbidden(error_message.to_string())),
        }
    }
}
