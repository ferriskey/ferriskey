use crate::{
    application::{
        auth::Identity,
        http::{
            policies::PolicyEnforcer,
            server::{api_entities::api_error::ApiError, app_state::AppState},
        },
    },
    domain::{realm::ports::realm_service::RealmService, role::entities::permission::Permissions},
};

pub struct RealmPolicy {}

impl RealmPolicy {
    pub async fn create(identity: Identity, state: AppState) -> Result<bool, ApiError> {
        let policy = PolicyEnforcer::new(state.clone());

        let user = policy.get_user_from_identity(&identity).await?;

        let realm = state
            .realm_service
            .get_by_name("master".to_string())
            .await
            .map_err(|_| ApiError::Forbidden("unauthorized".to_string()))?;

        if realm.id != user.realm_id {
            return Err(ApiError::Forbidden("unauthorized".to_string()));
        }

        let permissions = policy.get_user_permissions(&user).await?;

        let c = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm],
        );

        Ok(c)
    }
}
