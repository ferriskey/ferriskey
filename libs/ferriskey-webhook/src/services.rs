use std::sync::Arc;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::{FerriskeyPolicy, ensure_policy};
use ferriskey_domain::realm::ports::RealmRepository;
use ferriskey_domain::user::ports::{UserRepository, UserRoleRepository};

use crate::endpoint::{reject_reserved_headers, validate_endpoint};
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

        let endpoint = validate_endpoint(&input.endpoint)?;
        reject_reserved_headers(&input.headers)?;

        let webhook = self
            .webhook_repository
            .create_webhook(
                realm_id,
                input.name,
                input.description,
                endpoint.to_string(),
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

        self.webhook_repository
            .get_webhook_by_id(input.webhook_id, realm_id)
            .await?
            .ok_or(CoreError::WebhookNotFound)?;

        let endpoint = validate_endpoint(&input.endpoint)?;
        reject_reserved_headers(&input.headers)?;

        let webhook = self
            .webhook_repository
            .update_webhook(
                realm_id,
                input.webhook_id,
                input.name,
                input.description,
                endpoint.to_string(),
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
            .await?
            .ok_or(CoreError::WebhookNotFound)?;

        self.webhook_repository
            .delete_webhook(realm_id, input.webhook_id)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::realm::{Realm, ports::MockRealmRepository};
    use ferriskey_domain::role::entities::Role;
    use ferriskey_domain::user::{
        entities::User,
        ports::{MockUserRepository, MockUserRoleRepository},
    };
    use uuid::Uuid;

    use super::*;
    use crate::ports::MockWebhookRepository;

    fn test_realm() -> Realm {
        Realm {
            id: Uuid::new_v4().into(),
            name: "test-realm".to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_user(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            username: "admin".to_string(),
            firstname: Some("Admin".to_string()),
            lastname: Some("User".to_string()),
            email: Some("admin@test.com".to_string()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            client_id: None,
            required_actions: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
        }
    }

    fn admin_role(realm: &Realm) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "admin".to_string(),
            description: None,
            permissions: vec!["manage_webhooks".to_string()],
            realm_id: realm.id,
            client_id: None,
            client: None,
            require_mfa: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn owned_webhook(id: Uuid) -> Webhook {
        Webhook {
            id,
            endpoint: "https://93.184.216.34/hook".to_string(),
            headers: HashMap::new(),
            secret: "existing-secret".to_string(),
            name: Some("existing".to_string()),
            description: None,
            subscribers: Vec::new(),
            triggered_at: None,
            updated_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    fn update_input(realm_name: &str, webhook_id: Uuid) -> UpdateWebhookInput {
        UpdateWebhookInput {
            realm_name: realm_name.to_string(),
            webhook_id,
            name: Some("renamed".to_string()),
            description: None,
            endpoint: "https://93.184.216.34/hook".to_string(),
            headers: HashMap::new(),
            subscribers: Vec::new(),
        }
    }

    fn delete_input(realm_name: &str, webhook_id: Uuid) -> DeleteWebhookInput {
        DeleteWebhookInput {
            realm_name: realm_name.to_string(),
            webhook_id,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_service(
        realm_repo: MockRealmRepository,
        user_repo: MockUserRepository,
        user_role_repo: MockUserRoleRepository,
        webhook_repo: MockWebhookRepository,
    ) -> WebhookServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockWebhookRepository,
    > {
        let client_repo = MockClientRepository::new();
        let policy = Arc::new(FerriskeyPolicy::new(
            Arc::new(user_repo),
            Arc::new(client_repo),
            Arc::new(user_role_repo),
        ));
        WebhookServiceImpl::new(Arc::new(realm_repo), Arc::new(webhook_repo), policy)
    }

    fn allowing_realm_and_role_mocks(
        realm: &Realm,
    ) -> (MockRealmRepository, MockUserRoleRepository) {
        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo.expect_get_by_name().returning(move |_| {
            let r = realm_clone.clone();
            Box::pin(async move { Ok(Some(r)) })
        });

        let mut user_role_repo = MockUserRoleRepository::new();
        let realm_for_roles = realm.clone();
        user_role_repo.expect_get_user_roles().returning(move |_| {
            let r = realm_for_roles.clone();
            Box::pin(async move { Ok(vec![admin_role(&r)]) })
        });

        (realm_repo, user_role_repo)
    }

    /// The repository's `get_webhook_by_id` is scoped by realm, so it returns `None` both when
    /// the id belongs to a different realm and when it never existed at all — the service must
    /// not be able to act on either case, nor tell a caller which one it hit.
    #[tokio::test]
    async fn update_webhook_not_found_is_indistinguishable_for_cross_realm_and_missing_ids() {
        let realm = test_realm();
        let user = test_user(&realm);
        let (realm_repo, user_role_repo) = allowing_realm_and_role_mocks(&realm);

        let mut webhook_repo = MockWebhookRepository::new();
        webhook_repo
            .expect_get_webhook_by_id()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        webhook_repo.expect_update_webhook().never();

        let service = build_service(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            webhook_repo,
        );

        let cross_realm_webhook_id = Uuid::new_v4();
        let cross_realm_result = service
            .update_webhook(
                Identity::User(user.clone()),
                update_input(&realm.name, cross_realm_webhook_id),
            )
            .await;

        let missing_webhook_id = Uuid::new_v4();
        let missing_result = service
            .update_webhook(
                Identity::User(user),
                update_input(&realm.name, missing_webhook_id),
            )
            .await;

        assert!(matches!(
            cross_realm_result,
            Err(CoreError::WebhookNotFound)
        ));
        assert!(matches!(missing_result, Err(CoreError::WebhookNotFound)));
        assert_eq!(
            cross_realm_result.unwrap_err().to_string(),
            missing_result.unwrap_err().to_string()
        );
    }

    #[tokio::test]
    async fn update_webhook_writes_and_notifies_when_webhook_exists_and_policy_allows() {
        let realm = test_realm();
        let user = test_user(&realm);
        let (realm_repo, user_role_repo) = allowing_realm_and_role_mocks(&realm);

        let realm_id = realm.id;
        let existing_id = Uuid::new_v4();
        let existing = owned_webhook(existing_id);

        let mut webhook_repo = MockWebhookRepository::new();
        let existing_for_lookup = existing.clone();
        webhook_repo
            .expect_get_webhook_by_id()
            .returning(move |id, rid| {
                let result =
                    (id == existing_id && rid == realm_id).then(|| existing_for_lookup.clone());
                Box::pin(async move { Ok(result) })
            });

        let updated = existing.clone();
        webhook_repo
            .expect_update_webhook()
            .times(1)
            .returning(move |_, _, _, _, _, _, _| {
                let webhook = updated.clone();
                Box::pin(async move { Ok(webhook) })
            });

        webhook_repo
            .expect_notify::<Webhook>()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = build_service(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            webhook_repo,
        );

        let result = service
            .update_webhook(Identity::User(user), update_input(&realm.name, existing_id))
            .await;

        assert!(result.is_ok());
    }

    /// Mirrors `update_webhook_not_found_is_indistinguishable_for_cross_realm_and_missing_ids`:
    /// `delete_webhook` must fail closed on the same realm-scoped precondition, not merely fetch
    /// it and delete anyway — that gap is exactly what shipped in the finding.
    #[tokio::test]
    async fn delete_webhook_not_found_is_indistinguishable_for_cross_realm_and_missing_ids() {
        let realm = test_realm();
        let user = test_user(&realm);
        let (realm_repo, user_role_repo) = allowing_realm_and_role_mocks(&realm);

        let mut webhook_repo = MockWebhookRepository::new();
        webhook_repo
            .expect_get_webhook_by_id()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        webhook_repo.expect_delete_webhook().never();

        let service = build_service(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            webhook_repo,
        );

        let cross_realm_webhook_id = Uuid::new_v4();
        let cross_realm_result = service
            .delete_webhook(
                Identity::User(user.clone()),
                delete_input(&realm.name, cross_realm_webhook_id),
            )
            .await;

        let missing_webhook_id = Uuid::new_v4();
        let missing_result = service
            .delete_webhook(
                Identity::User(user),
                delete_input(&realm.name, missing_webhook_id),
            )
            .await;

        assert!(matches!(
            cross_realm_result,
            Err(CoreError::WebhookNotFound)
        ));
        assert!(matches!(missing_result, Err(CoreError::WebhookNotFound)));
        assert_eq!(
            cross_realm_result.unwrap_err().to_string(),
            missing_result.unwrap_err().to_string()
        );
    }

    #[tokio::test]
    async fn delete_webhook_deletes_and_notifies_when_webhook_exists_and_policy_allows() {
        let realm = test_realm();
        let user = test_user(&realm);
        let (realm_repo, user_role_repo) = allowing_realm_and_role_mocks(&realm);

        let realm_id = realm.id;
        let existing_id = Uuid::new_v4();
        let existing = owned_webhook(existing_id);

        let mut webhook_repo = MockWebhookRepository::new();
        let existing_for_lookup = existing.clone();
        webhook_repo
            .expect_get_webhook_by_id()
            .returning(move |id, rid| {
                let result =
                    (id == existing_id && rid == realm_id).then(|| existing_for_lookup.clone());
                Box::pin(async move { Ok(result) })
            });

        webhook_repo
            .expect_delete_webhook()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        webhook_repo
            .expect_notify::<Webhook>()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let service = build_service(
            realm_repo,
            MockUserRepository::new(),
            user_role_repo,
            webhook_repo,
        );

        let result = service
            .delete_webhook(Identity::User(user), delete_input(&realm.name, existing_id))
            .await;

        assert!(result.is_ok());
    }
}
