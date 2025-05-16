use crate::{
    application::{
        auth::Identity,
        http::server::{api_entities::api_error::ApiError, app_state::AppState},
    },
    domain::user::ports::user_service::UserService,
};

pub struct RolePolicy {}

impl RolePolicy {
    pub async fn create(identity: Identity, state: AppState) -> Result<bool, ApiError> {
        // Implement your logic to check if the user has permission to create a role
        // For example, check if the user has a specific role or permission
        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => {
                let service_account = state
                    .user_service
                    .get_by_client_id(client.id)
                    .await
                    .map_err(|_| ApiError::Forbidden("Client not found".to_string()))?;

                service_account
            }
        };

        Ok(true)
    }
}
