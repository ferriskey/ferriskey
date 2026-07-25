use std::sync::Arc;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, ensure_policy};
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::entities::{
    webhook::Webhook, webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger,
};
use crate::ports::{
    CreateWebhookInput, DeleteWebhookInput, GetWebhookInput, GetWebhookSubscribersInput,
    GetWebhooksInput, UpdateWebhookInput, WebhookPolicy, WebhookRepository, WebhookService,
};

#[derive(Clone, Debug)]
pub struct WebhookServiceImpl<R, U, C, UR, W>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) webhook_repository: Arc<W>,

    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, W> WebhookServiceImpl<R, U, C, UR, W>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        webhook_repository: Arc<W>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            webhook_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, W> WebhookService for WebhookServiceImpl<R, U, C, UR, W>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
{
    async fn get_webhooks_by_realm(
        &self,
        identity: Identity,
        input: GetWebhooksInput,
    ) -> Result<Vec<Webhook>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_webhook(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let webhooks = self
            .webhook_repository
            .fetch_webhooks_by_realm(realm_id)
            .await?;

        Ok(webhooks)
    }

    async fn get_webhooks_by_subscribers(
        &self,
        identity: Identity,
        input: GetWebhookSubscribersInput,
    ) -> Result<Vec<Webhook>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_webhook(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let webhooks = self
            .webhook_repository
            .fetch_webhooks_by_subscriber(realm_id, input.subscriber)
            .await?;

        Ok(webhooks)
    }

    async fn get_webhook(
        &self,
        identity: Identity,
        input: GetWebhookInput,
    ) -> Result<Option<Webhook>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_webhook(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let webhook = self
            .webhook_repository
            .get_webhook_by_id(input.webhook_id, realm_id)
            .await?;

        Ok(webhook)
    }

    async fn create_webhook(
        &self,
        identity: Identity,
        input: CreateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_create_webhook(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let webhook = self
            .webhook_repository
            .create_webhook(
                realm_id,
                input.name,
                input.description,
                input.endpoint,
                input.headers,
                input.subscribers,
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::WebhookCreated,
                    realm_id.into(),
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
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_webhook(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let webhook = self
            .webhook_repository
            .update_webhook(
                input.webhook_id,
                input.name,
                input.description,
                input.endpoint,
                input.headers,
                input.subscribers,
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::WebhookUpdated,
                    realm_id.into(),
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
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_webhook(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let webhook = self
            .webhook_repository
            .get_webhook_by_id(input.webhook_id, realm_id)
            .await?;

        self.webhook_repository
            .delete_webhook(input.webhook_id)
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::WebhookDeleted,
                    realm_id.into(),
                    Some(webhook),
                ),
            )
            .await?;

        Ok(())
    }
}
