use crate::{
    application::{
        auth::Identity,
        http::{
            policies::PolicyEnforcer,
            server::{api_entities::api_error::ApiError, app_state::AppState},
        },
    },
    domain::{realm::entities::realm::Realm, role::entities::permission::Permissions},
};

pub struct RolePolicy {}

impl RolePolicy {
    pub async fn create(
        identity: Identity,
        state: AppState,
        target_realm: Realm,
    ) -> Result<bool, ApiError> {
        let policy = PolicyEnforcer::new(state);

        let user = policy.get_user_from_identity(&identity).await?;
        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageClients],
        );

        Ok(has_permission)
    }

    pub async fn view(
        identity: Identity,
        state: AppState,
        target_realm: Realm,
    ) -> Result<bool, ApiError> {
        let policy = PolicyEnforcer::new(state.clone());
        let user = policy.get_user_from_identity(&identity).await?;

        let permissions = policy
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[
                Permissions::ViewRoles,
                Permissions::ManageRealm,
                Permissions::ManageRoles,
            ],
        );

        Ok(has_permission)
    }
}
