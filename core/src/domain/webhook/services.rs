use uuid::Uuid;

use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::{
        entities::{
            webhook::Webhook, webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger,
        },
        ports::{
            CreateWebhookInput, DeleteWebhookInput, GetWebhookInput, GetWebhookSubscribersInput,
            GetWebhooksInput, UpdateWebhookInput, WebhookNotifierRepository, WebhookPolicy,
            WebhookRepository, WebhookService,
        },
    },
};

pub mod webhook_notifier_service;

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC> WebhookService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    WN: WebhookNotifierRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
{
    async fn get_webhooks_by_realm(
        &self,
        identity: Identity,
        input: GetWebhooksInput,
    ) -> Result<Vec<Webhook>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_webhook(identity, realm).await,
            "insufficient permissions",
        )?;

        let webhooks = self
            .webhook_repository
            .fetch_webhooks_by_realm(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(webhooks)
    }

    async fn get_webhooks_by_subscribers(
        &self,
        identity: Identity,
        input: GetWebhookSubscribersInput,
    ) -> Result<Vec<Webhook>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_webhook(identity, realm).await,
            "insufficient permissions",
        )?;

        let webhooks = self
            .webhook_repository
            .fetch_webhooks_by_subscriber(realm_id, input.subscriber)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(webhooks)
    }

    async fn get_webhook(
        &self,
        identity: Identity,
        input: GetWebhookInput,
    ) -> Result<Option<Webhook>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_webhook(identity, realm).await,
            "insufficient permissions",
        )?;

        let webhook = self
            .webhook_repository
            .get_webhook_by_id(input.webhook_id, realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(webhook)
    }

    async fn create_webhook(
        &self,
        identity: Identity,
        input: CreateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_create_webhook(identity, realm).await,
            "insufficient permissions",
        )?;

        let webhook = self
            .webhook_repository
            .create_webhook(
                realm_id,
                input.name,
                input.description,
                input.endpoint,
                input.subscribers,
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let webhooks = self
            .webhook_repository
            .fetch_webhooks_by_subscriber(realm_id, WebhookTrigger::WebhookCreated)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.webhook_notifier_repository
            .notify(
                webhooks,
                WebhookPayload::new(
                    WebhookTrigger::WebhookCreated,
                    realm_id,
                    Some(webhook.clone()),
                ),
            )
            .await?;

        Ok(webhook)
    }

    async fn update_webhook(
        &self,
        identity: Identity,
        input: UpdateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_webhook(identity, realm).await,
            "insufficient permissions",
        )?;

        let webhook = self
            .webhook_repository
            .update_webhook(
                input.webhook_id,
                input.name,
                input.description,
                input.endpoint,
                input.subscribers,
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let webhooks = self
            .webhook_repository
            .fetch_webhooks_by_subscriber(realm_id, WebhookTrigger::WebhookCreated)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.webhook_notifier_repository
            .notify(
                webhooks,
                WebhookPayload::new(
                    WebhookTrigger::WebhookUpdated,
                    realm_id,
                    Some(webhook.clone()),
                ),
            )
            .await?;

        Ok(webhook)
    }

    async fn delete_webhook(
        &self,
        identity: Identity,
        input: DeleteWebhookInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_webhook(identity, realm).await,
            "insufficient permissions",
        )?;

        self.webhook_repository
            .delete_webhook(input.webhook_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let webhooks = self
            .webhook_repository
            .fetch_webhooks_by_subscriber(realm_id, WebhookTrigger::WebhookCreated)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.webhook_notifier_repository
            .notify(
                webhooks,
                WebhookPayload::<Uuid>::new(WebhookTrigger::WebhookDeleted, realm_id, None),
            )
            .await?;

        Ok(())
    }
}
