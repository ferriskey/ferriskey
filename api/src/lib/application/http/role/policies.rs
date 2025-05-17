use std::collections::HashSet;

use crate::{
    application::{
        auth::Identity,
        http::server::{api_entities::api_error::ApiError, app_state::AppState},
    },
    domain::{role::entities::permission::Permissions, user::ports::user_service::UserService},
};

pub struct RolePolicy {}

impl RolePolicy {
    pub async fn create(identity: Identity, state: AppState) -> Result<bool, ApiError> {
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

        let roles = state
            .user_service
            .get_user_roles(user.id)
            .await
            .map_err(|_| ApiError::Forbidden("User not found".to_string()))?;

        //let mut permissions: Vec<Permissions> = Vec::new();
        // create a vector of permissions unique
        let mut permissions: HashSet<Permissions> = HashSet::new();

        for role in roles {
            let t = role.permissions as u64;

            let a = Permissions::from_bitfield(t);
            for p in a {
                permissions.insert(p);
            }
        }

        let c = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(c)
    }
}
