use crate::{
    application::user::policies::user_policy::UserPolicy,
    domain::{
        authentication::value_objects::Identity,
        client::ports::ClientService,
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
pub struct CreateUserUseCase<R, U, C>
where
    R: RealmService,
    U: UserService,
    C: ClientService,
{
    pub realm_service: R,
    pub user_service: U,
    pub client_service: C,
}

impl<R, U, C> CreateUserUseCase<R, U, C>
where
    R: RealmService,
    U: UserService,
    C: ClientService,
{
    pub fn new(realm_service: R, user_service: U, client_service: C) -> Self {
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
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to create a user",
        )?;

        self.user_service
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
            .await
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
