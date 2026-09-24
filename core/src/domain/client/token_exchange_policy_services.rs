use std::collections::HashSet;
use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::{
        entities::{
            Client, CreateTokenExchangePolicyInput, DeleteTokenExchangePolicyInput,
            GetTokenExchangePoliciesInput,
            token_exchange_policy::{TokenExchangePolicy, TokenExchangePolicyDefinition},
        },
        ports::{
            ClientPolicy, ClientRepository, TokenExchangePolicyRepository,
            TokenExchangePolicyService,
        },
        value_objects::CreateTokenExchangePolicyRequest,
    },
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::{entities::RealmScope, ports::RealmRepository},
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct TokenExchangePolicyServiceImpl<R, U, C, UR, TP>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    TP: TokenExchangePolicyRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) token_exchange_policy_repository: Arc<TP>,

    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, TP> TokenExchangePolicyServiceImpl<R, U, C, UR, TP>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    TP: TokenExchangePolicyRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        token_exchange_policy_repository: Arc<TP>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            token_exchange_policy_repository,
            policy,
        }
    }

    async fn load_client(
        &self,
        scope: &RealmScope,
        client_id: uuid::Uuid,
    ) -> Result<Client, CoreError> {
        Ok(self
            .client_repository
            .get_by_id(scope.id(), client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(scope)?
            .into_inner())
    }

    async fn normalize(
        &self,
        scope: &RealmScope,
        client: &Client,
        payload: CreateTokenExchangePolicyRequest,
    ) -> Result<TokenExchangePolicyDefinition, CoreError> {
        let target_audience = payload.target_audience.trim().to_string();

        if target_audience.is_empty() {
            return Err(CoreError::InvalidTokenExchangePolicy(
                "target_audience is required".to_string(),
            ));
        }

        if target_audience == client.client_id {
            return Err(CoreError::InvalidTokenExchangePolicy(
                "target_audience cannot be the requesting client itself".to_string(),
            ));
        }

        match self
            .client_repository
            .get_by_client_id(target_audience.clone(), scope.id())
            .await
        {
            Ok(target) => {
                target
                    .in_realm(scope)
                    .map_err(|_| unknown_audience(&target_audience))?;
            }
            Err(CoreError::NotFound) => return Err(unknown_audience(&target_audience)),
            Err(error) => return Err(error),
        }

        let allowed_scopes = payload.allowed_scopes.map(normalize_scopes).transpose()?;

        Ok(TokenExchangePolicyDefinition {
            target_audience,
            allowed_scopes,
            allow_impersonation: payload.allow_impersonation,
            allow_delegation: payload.allow_delegation,
        })
    }
}

fn unknown_audience(target_audience: &str) -> CoreError {
    CoreError::InvalidTokenExchangePolicy(format!(
        "target_audience {target_audience} is not a client of this realm"
    ))
}

fn normalize_scopes(scopes: Vec<String>) -> Result<Vec<String>, CoreError> {
    if scopes.is_empty() {
        return Err(CoreError::InvalidTokenExchangePolicy(
            "allowed_scopes cannot be empty, send null for no ceiling".to_string(),
        ));
    }

    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(scopes.len());

    for scope in scopes {
        let scope = scope.trim();

        if scope.is_empty() {
            return Err(CoreError::InvalidTokenExchangePolicy(
                "allowed_scopes cannot contain an empty scope".to_string(),
            ));
        }

        if scope.chars().any(char::is_whitespace) {
            return Err(CoreError::InvalidTokenExchangePolicy(format!(
                "allowed_scopes entry {scope:?} contains whitespace"
            )));
        }

        if seen.insert(scope.to_string()) {
            normalized.push(scope.to_string());
        }
    }

    Ok(normalized)
}

impl<R, U, C, UR, TP> TokenExchangePolicyService for TokenExchangePolicyServiceImpl<R, U, C, UR, TP>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    TP: TokenExchangePolicyRepository,
{
    async fn create_token_exchange_policy(
        &self,
        identity: Identity,
        input: CreateTokenExchangePolicyInput,
    ) -> Result<TokenExchangePolicy, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        let client = self.load_client(&scope, input.client_id).await?;

        let definition = self.normalize(&scope, &client, input.payload).await?;

        self.token_exchange_policy_repository
            .create(scope.id(), client.id, definition)
            .await
            .map_err(|error| match error {
                CoreError::AlreadyExists => CoreError::TokenExchangePolicyAlreadyExists,
                other => other,
            })
    }

    async fn get_token_exchange_policies(
        &self,
        identity: Identity,
        input: GetTokenExchangePoliciesInput,
    ) -> Result<Vec<TokenExchangePolicy>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        let client = self.load_client(&scope, input.client_id).await?;

        self.token_exchange_policy_repository
            .list_by_client(client.id)
            .await
    }

    async fn delete_token_exchange_policy(
        &self,
        identity: Identity,
        input: DeleteTokenExchangePolicyInput,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        let client = self.load_client(&scope, input.client_id).await?;

        self.token_exchange_policy_repository
            .delete(client.id, input.policy_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        client::ports::{MockClientRepository, MockTokenExchangePolicyRepository},
        common::services::tests::{
            create_test_realm_with_name, create_test_user_identity_with_realm,
        },
        realm::{
            entities::{Realm, Unscoped},
            ports::MockRealmRepository,
        },
        role::entities::{Role, permission::Permissions},
        user::ports::{MockUserRepository, MockUserRoleRepository},
    };
    use uuid::Uuid;

    type TestService = TokenExchangePolicyServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockTokenExchangePolicyRepository,
    >;

    struct Mocks {
        realm: MockRealmRepository,
        client: MockClientRepository,
        user_role: MockUserRoleRepository,
        policies: MockTokenExchangePolicyRepository,
    }

    impl Mocks {
        fn new(realm: &Realm, identity: &Identity, permissions: Vec<String>) -> Self {
            let mut realm_repo = MockRealmRepository::new();
            let found = realm.clone();
            realm_repo.expect_get_by_name().returning(move |_| {
                let found = found.clone();
                Box::pin(async move { Ok(Some(found)) })
            });

            let role = Role {
                id: Uuid::new_v4(),
                name: "client-admin".to_string(),
                description: None,
                permissions,
                realm_id: realm.id,
                client_id: None,
                client: None,
                require_mfa: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let user_id = identity.id();
            let mut user_role = MockUserRoleRepository::new();
            user_role
                .expect_get_user_roles()
                .withf(move |id| *id == user_id)
                .returning(move |_| {
                    let roles = vec![role.clone()];
                    Box::pin(async move { Ok(roles) })
                });

            Self {
                realm: realm_repo,
                client: MockClientRepository::new(),
                user_role,
                policies: MockTokenExchangePolicyRepository::new(),
            }
        }

        fn for_admin_of(realm: &Realm, identity: &Identity) -> Self {
            Self::new(
                realm,
                identity,
                vec![
                    Permissions::ManageClients.name(),
                    Permissions::ViewClients.name(),
                ],
            )
        }

        fn with_client(mut self, client: &Client) -> Self {
            let stored = client.clone();
            self.client.expect_get_by_id().returning(move |_, _| {
                let stored = stored.clone();
                Box::pin(async move { Ok(Unscoped::new(stored)) })
            });
            self
        }

        fn with_audiences(mut self, realm: &Realm, audiences: &[&str]) -> Self {
            let realm_id = realm.id;
            let known: Vec<String> = audiences.iter().map(|a| a.to_string()).collect();
            self.client
                .expect_get_by_client_id()
                .returning(move |client_id, _| {
                    let found = known
                        .contains(&client_id)
                        .then(|| Client::from_realm_and_client_id(realm_id, client_id));
                    Box::pin(async move { found.map(Unscoped::new).ok_or(CoreError::NotFound) })
                });
            self
        }

        fn build(self) -> TestService {
            let user = Arc::new(MockUserRepository::new());
            let client = Arc::new(self.client);
            let policy = Arc::new(FerriskeyPolicy::new(
                user,
                client.clone(),
                Arc::new(self.user_role),
            ));

            TokenExchangePolicyServiceImpl::new(
                Arc::new(self.realm),
                client,
                Arc::new(self.policies),
                policy,
            )
        }
    }

    fn requesting_client(realm: &Realm) -> Client {
        Client::from_realm_and_client_id(realm.id, "frontend".to_string())
    }

    fn create_input(
        client: &Client,
        payload: CreateTokenExchangePolicyRequest,
    ) -> CreateTokenExchangePolicyInput {
        CreateTokenExchangePolicyInput {
            realm_name: "acme".to_string(),
            client_id: client.id,
            payload,
        }
    }

    fn request(
        target_audience: &str,
        allowed_scopes: Option<Vec<&str>>,
    ) -> CreateTokenExchangePolicyRequest {
        CreateTokenExchangePolicyRequest {
            target_audience: target_audience.to_string(),
            allowed_scopes: allowed_scopes
                .map(|scopes| scopes.into_iter().map(str::to_string).collect()),
            allow_impersonation: true,
            allow_delegation: false,
        }
    }

    fn assert_invalid(result: Result<TokenExchangePolicy, CoreError>) {
        assert!(
            matches!(result, Err(CoreError::InvalidTokenExchangePolicy(_))),
            "expected an invalid policy error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn an_admin_creates_a_policy_with_a_normalized_definition() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity)
            .with_client(&client)
            .with_audiences(&realm, &["orders-api"]);

        let expected_client = client.id;
        mocks
            .policies
            .expect_create()
            .withf(move |_, client_id, definition| {
                *client_id == expected_client
                    && *definition
                        == TokenExchangePolicyDefinition {
                            target_audience: "orders-api".to_string(),
                            allowed_scopes: Some(vec![
                                "orders:read".to_string(),
                                "orders:write".to_string(),
                            ]),
                            allow_impersonation: true,
                            allow_delegation: false,
                        }
            })
            .times(1)
            .returning(|realm_id, client_id, definition| {
                let policy = TokenExchangePolicy::new(realm_id, client_id, definition);
                Box::pin(async move { Ok(policy) })
            });

        let policy = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(
                    &client,
                    request(
                        "  orders-api ",
                        Some(vec![" orders:read", "orders:write", "orders:read "]),
                    ),
                ),
            )
            .await
            .expect("policy is created");

        assert_eq!(policy.target_audience, "orders-api");
        assert_eq!(policy.client_id, client.id);
        assert_eq!(policy.realm_id, realm.id);
    }

    #[tokio::test]
    async fn an_unknown_target_audience_is_rejected() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mocks = Mocks::for_admin_of(&realm, &identity)
            .with_client(&client)
            .with_audiences(&realm, &["orders-api"]);

        let result = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(&client, request("billing-api", None)),
            )
            .await;

        assert_invalid(result);
    }

    #[tokio::test]
    async fn a_client_cannot_target_itself() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mocks = Mocks::for_admin_of(&realm, &identity)
            .with_client(&client)
            .with_audiences(&realm, &["frontend"]);

        let result = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(&client, request("frontend", None)),
            )
            .await;

        assert_invalid(result);
    }

    #[tokio::test]
    async fn a_scope_with_inner_whitespace_is_rejected() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mocks = Mocks::for_admin_of(&realm, &identity)
            .with_client(&client)
            .with_audiences(&realm, &["orders-api"]);

        let result = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(&client, request("orders-api", Some(vec!["a b"]))),
            )
            .await;

        assert_invalid(result);
    }

    #[tokio::test]
    async fn an_empty_scope_is_rejected() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mocks = Mocks::for_admin_of(&realm, &identity)
            .with_client(&client)
            .with_audiences(&realm, &["orders-api"]);

        let result = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(&client, request("orders-api", Some(vec!["openid", "  "]))),
            )
            .await;

        assert_invalid(result);
    }

    #[tokio::test]
    async fn an_empty_scope_list_is_rejected() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mocks = Mocks::for_admin_of(&realm, &identity)
            .with_client(&client)
            .with_audiences(&realm, &["orders-api"]);

        let result = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(&client, request("orders-api", Some(vec![]))),
            )
            .await;

        assert_invalid(result);
    }

    #[tokio::test]
    async fn a_duplicate_audience_is_reported_as_a_conflict() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity)
            .with_client(&client)
            .with_audiences(&realm, &["orders-api"]);
        mocks
            .policies
            .expect_create()
            .returning(|_, _, _| Box::pin(async { Err(CoreError::AlreadyExists) }));

        let result = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(&client, request("orders-api", None)),
            )
            .await;

        assert!(matches!(
            result,
            Err(CoreError::TokenExchangePolicyAlreadyExists)
        ));
    }

    #[tokio::test]
    async fn an_identity_without_client_permissions_is_forbidden() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mocks = Mocks::new(&realm, &identity, vec![Permissions::ViewClients.name()]);

        let result = mocks
            .build()
            .create_token_exchange_policy(
                identity,
                create_input(&client, request("orders-api", None)),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn deleting_an_unknown_policy_is_not_found() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity).with_client(&client);
        let policy_id = Uuid::new_v4();
        let expected_client = client.id;
        mocks
            .policies
            .expect_delete()
            .withf(move |client_id, id| *client_id == expected_client && *id == policy_id)
            .times(1)
            .returning(|_, _| Box::pin(async { Err(CoreError::NotFound) }));

        let result = mocks
            .build()
            .delete_token_exchange_policy(
                identity,
                DeleteTokenExchangePolicyInput {
                    realm_name: "acme".to_string(),
                    client_id: client.id,
                    policy_id,
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn listing_returns_the_repository_rows() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let client = requesting_client(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity).with_client(&client);
        let stored = vec![
            TokenExchangePolicy::new(
                realm.id,
                client.id,
                TokenExchangePolicyDefinition {
                    target_audience: "orders-api".to_string(),
                    allowed_scopes: None,
                    allow_impersonation: true,
                    allow_delegation: false,
                },
            ),
            TokenExchangePolicy::new(
                realm.id,
                client.id,
                TokenExchangePolicyDefinition {
                    target_audience: "billing-api".to_string(),
                    allowed_scopes: Some(vec!["billing:read".to_string()]),
                    allow_impersonation: false,
                    allow_delegation: true,
                },
            ),
        ];
        let rows = stored.clone();
        let expected_client = client.id;
        mocks
            .policies
            .expect_list_by_client()
            .withf(move |client_id| *client_id == expected_client)
            .returning(move |_| {
                let rows = rows.clone();
                Box::pin(async move { Ok(rows) })
            });

        let listed = mocks
            .build()
            .get_token_exchange_policies(
                identity,
                GetTokenExchangePoliciesInput {
                    realm_name: "acme".to_string(),
                    client_id: client.id,
                },
            )
            .await
            .expect("policies are listed");

        assert_eq!(listed, stored);
    }
}
