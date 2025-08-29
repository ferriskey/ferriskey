use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService, DefaultWebhookNotifierService,
};
use crate::application::realm::policies::RealmPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::{Realm, RealmError};
use crate::domain::realm::ports::RealmService;
use crate::domain::user::ports::UserService;
use crate::domain::webhook::entities::webhook_payload::WebhookPayload;
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::ports::WebhookNotifierService;
use tracing::error;

#[derive(Clone)]
pub struct CreateRealmUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}

pub struct CreateRealmUseCaseParams {
    pub realm_name: String,
}

impl CreateRealmUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        webhook_notifier_service: DefaultWebhookNotifierService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            webhook_notifier_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: CreateRealmUseCaseParams,
    ) -> Result<Realm, RealmError> {
        let realm_master = self.realm_service.get_by_name("master".to_string()).await?;

        ensure_permissions(
            RealmPolicy::create(
                identity.clone(),
                realm_master,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to create a realm",
        )
        .map_err(|_| RealmError::Forbidden)?;

        let user = match identity {
            Identity::User(user) => user,
            Identity::Client(client) => self
                .user_service
                .get_by_client_id(client.id)
                .await
                .map_err(|_| RealmError::InternalServerError)?,
        };

        let realm = self
            .realm_service
            .create_realm_with_user(params.realm_name, &user)
            .await?;

        self.webhook_notifier_service
            .notify(
                realm.id,
                WebhookPayload::new(WebhookTrigger::RealmCreated, realm.id, Some(realm.clone())),
            )
            .await
            .map_err(|e| {
                error!("Failed to notify webhook: {}", e);
                RealmError::InternalServerError
            })?;

        Ok(realm)
    }
}
