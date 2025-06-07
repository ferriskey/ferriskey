use crate::application::auth::Identity;
use crate::application::http::policies::PolicyEnforcer;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::app_state::AppState;
use crate::domain::realm::entities::realm::Realm;
use crate::domain::role::entities::permission::Permissions;

pub struct ClientPolicy {}

impl ClientPolicy {
    pub async fn delete(
        identity: Identity,
        state: AppState,
        target_realm: Realm
    ) -> Result<bool, ApiError> {
        let policy = PolicyEnforcer::new(state);

        let user = policy.get_user_from_identity(&identity).await?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageClients]
        );

        Ok(has_permission)
    }
}