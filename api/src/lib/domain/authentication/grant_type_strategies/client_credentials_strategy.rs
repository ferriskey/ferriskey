use std::sync::Arc;

use tracing::info;

use crate::domain::{
    authentication::{
        entities::{error::AuthenticationError, jwt_token::JwtToken},
        ports::grant_type_strategy::{GrantTypeParams, GrantTypeStrategy},
    },
    client::{
        ports::client_service::ClientService, services::client_service::DefaultClientService,
    },
    jwt::{
        entities::jwt_claim::JwtClaim, ports::jwt_service::JwtService,
        services::jwt_service::DefaultJwtService,
    },
    user::{ports::user_service::UserService, services::user_service::DefaultUserService},
};

#[derive(Clone)]
pub struct ClientCredentialsStrategy {
    pub client_service: Arc<DefaultClientService>,
    pub user_service: Arc<DefaultUserService>,
    pub jwt_service: Arc<DefaultJwtService>,
}

impl ClientCredentialsStrategy {
    pub fn new(
        client_service: Arc<DefaultClientService>,
        user_service: Arc<DefaultUserService>,
        jwt_service: Arc<DefaultJwtService>,
    ) -> Self {
        Self {
            client_service,
            user_service,
            jwt_service,
        }
    }
}

impl GrantTypeStrategy for ClientCredentialsStrategy {
    fn execute(
        &self,
        params: GrantTypeParams,
    ) -> impl Future<Output = Result<JwtToken, AuthenticationError>> + Send {
        async move {
            let client = self
                .client_service
                .get_by_client_id(params.client_id.clone(), params.realm_id)
                .await
                .map_err(|_| AuthenticationError::Invalid);

            match client {
                Ok(client) => {
                    info!("success to login with client: {:?}", client.name);

                    let user = self
                        .user_service
                        .get_by_client_id(client.id, params.realm_id)
                        .await
                        .map_err(|_| AuthenticationError::ServiceAccountNotFound)?;

                    let claims = JwtClaim::new(
                        user.id,
                        user.username,
                        "http://localhost:3333/realms/master".to_string(),
                        vec!["master-realm".to_string(), "account".to_string()],
                        "Bearer".to_string(),
                        params.client_id,
                    );

                    let jwt = self
                        .jwt_service
                        .generate_token(claims)
                        .await
                        .map_err(|_| AuthenticationError::InternalServerError)?;

                    Ok(JwtToken::new(
                        jwt.token,
                        "Bearer".to_string(),
                        "8xLOxBtZp8".to_string(),
                        3600,
                        "id_token".to_string(),
                    ))
                }
                Err(error) => Err(error),
            }
        }
    }
}
