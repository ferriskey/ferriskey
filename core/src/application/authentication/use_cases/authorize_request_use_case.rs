use uuid::Uuid;

use crate::{
    application::common::services::{DefaultClientService, DefaultJwtService, DefaultUserService},
    domain::{
        authentication::{entities::AuthenticationError, value_objects::Identity},
        client::ports::ClientService,
        jwt::{
            entities::{ClaimsTyp, JwtClaim},
            ports::JwtService,
        },
        user::ports::UserService,
    },
};

#[derive(Clone)]
pub struct AuthorizeRequestUseCase {
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    jwt_service: DefaultJwtService,
}

pub struct AuthorizeRequestUseCaseInput {
    pub claims: JwtClaim,
    pub token: String,
}

pub struct AuthorizeRequestUseCaseOutput {
    pub identity: Identity,
}

impl AuthorizeRequestUseCase {
    pub fn new(
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        jwt_service: DefaultJwtService,
    ) -> Self {
        Self {
            user_service,
            client_service,
            jwt_service,
        }
    }

    pub async fn execute(
        &self,
        input: AuthorizeRequestUseCaseInput,
    ) -> Result<AuthorizeRequestUseCaseOutput, AuthenticationError> {
        let AuthorizeRequestUseCaseInput { claims, token } = input;

        // Vérifier que c'est un token Bearer
        if claims.typ != ClaimsTyp::Bearer {
            return Err(AuthenticationError::InternalServerError);
        }

        let user = self.user_service.get_by_id(claims.sub).await.map_err(|e| {
            tracing::error!("failed to get user by id {}: {:?}", claims.sub, e);
            AuthenticationError::InvalidUser
        })?;

        self.jwt_service
            .verify_refresh_token(token, user.realm_id)
            .await
            .map_err(|e| {
                tracing::error!("JWT verification failed: {:?}", e);
                AuthenticationError::Invalid
            })?;

        let identity: Identity = match claims.is_service_account() {
            true => {
                let client_id = claims.client_id.ok_or(AuthenticationError::InvalidClient)?;
                let client_id = Uuid::parse_str(&client_id).map_err(|e| {
                    tracing::error!("failed to parse client id: {:?}", e);
                    AuthenticationError::InvalidClient
                })?;

                let client = self
                    .client_service
                    .get_by_id(client_id)
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to get client by id {}: {:?}", client_id, e);
                        AuthenticationError::InvalidClient
                    })?;

                Identity::Client(client)
            }
            false => Identity::User(user),
        };

        Ok(AuthorizeRequestUseCaseOutput { identity })
    }
}
