use uuid::Uuid;

use crate::{
    application::common::FerriskeyService,
    domain::{
        authentication::{
            entities::{
                AuthInput, AuthOutput, AuthSession, AuthSessionParams, AuthenticateInput,
                AuthenticateOutput, AuthenticationMethod, AuthorizeRequestInput,
                AuthorizeRequestOutput, CredentialsAuthParams,
            },
            ports::{AuthService, AuthSessionRepository, AuthenticatePort, GrantTypeService},
            value_objects::{GrantTypeParams, Identity},
        },
        client::ports::{ClientRepository, RedirectUriRepository},
        common::entities::app_errors::CoreError,
        jwt::{
            entities::{ClaimsTyp, JwkKey},
            ports::KeyStoreRepository,
        },
        realm::ports::RealmRepository,
        user::ports::UserRepository,
    },
};

pub mod services;

impl AuthService for FerriskeyService {
    async fn auth(&self, input: AuthInput) -> Result<AuthOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        let redirect_uri = input.redirect_uri.clone();

        let client_redirect_uris = self
            .redirect_uri_repository
            .get_enabled_by_client_id(client.id)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?;

        if !client_redirect_uris.iter().any(|uri| {
            if uri.value == redirect_uri {
                return true;
            }

            if let Ok(regex) = regex::Regex::new(&uri.value) {
                return regex.is_match(&redirect_uri);
            }

            false
        }) {
            return Err(CoreError::InvalidClient);
        }

        if !client.enabled {
            return Err(CoreError::InvalidClient);
        }

        let params = AuthSessionParams {
            realm_id: realm.id,
            client_id: client.id,
            redirect_uri,
            response_type: input.response_type,
            scope: input.scope.unwrap_or_default(),
            state: input.state.clone(),
            nonce: None,
            user_id: None,
            code: None,
            authenticated: false,
        };
        let session = self
            .auth_session_repository
            .create(&AuthSession::new(params))
            .await
            .map_err(|_| CoreError::SessionCreateError)?;

        let login_url = format!(
            "?client_id={}&redirect_uri={}&state={}",
            client.client_id,
            input.redirect_uri,
            input.state.unwrap_or_default()
        );

        Ok(AuthOutput { login_url, session })
    }

    async fn get_certs(&self, realm_name: String) -> Result<Vec<JwkKey>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let jwk_keypair = self
            .keystore_repository
            .get_or_generate_key(realm.id)
            .await
            .map_err(|_| CoreError::RealmKeyNotFound)?;

        let jwk_key = jwk_keypair
            .to_jwk_key()
            .map_err(|e| CoreError::InvalidKey(e.to_string()))?;

        Ok(vec![jwk_key])
    }

    async fn exchange_token(
        &self,
        input: crate::domain::authentication::entities::ExchangeTokenInput,
    ) -> Result<crate::domain::authentication::entities::JwtToken, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        self.client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        let params = GrantTypeParams {
            realm_id: realm.id,
            base_url: input.base_url,
            realm_name: realm.name,
            client_id: input.client_id,
            client_secret: input.client_secret,
            code: input.code,
            username: input.username,
            password: input.password,
            refresh_token: input.refresh_token,
            redirect_uri: None,
        };

        self.grant_type_strategies
            .authenticate_with_grant_type(input.grant_type, params)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn authorize_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> Result<AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Bearer {
            return Err(CoreError::InternalServerError);
        }

        let user = self
            .user_repository
            .get_by_id(input.claims.sub)
            .await
            .map_err(|e| {
                tracing::error!("faield to get user by id {}: {:?}", input.claims.sub, e);

                CoreError::InvalidUser
            })?;

        self.grant_type_strategies
            .verify_token(input.token, user.realm_id)
            .await?;

        let identity: Identity = match input.claims.is_service_account() {
            true => {
                let client_id = input.claims.client_id.ok_or(CoreError::InvalidClient)?;
                let client_id = Uuid::parse_str(&client_id).map_err(|e| {
                    tracing::error!("failed to parse client id: {:?}", e);
                    CoreError::InvalidClient
                })?;

                let client = self
                    .client_repository
                    .get_by_id(client_id)
                    .await
                    .map_err(|e| {
                        tracing::error!("failed to get client by id {}: {:?}", client_id, e);
                        CoreError::InvalidClient
                    })?;

                Identity::Client(client)
            }
            false => Identity::User(user),
        };

        Ok(AuthorizeRequestOutput { identity })
    }

    async fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> Result<AuthenticateOutput, CoreError> {
        let auth_session = self
            .auth_session_repository
            .get_by_session_code(input.session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        match input.auth_method {
            AuthenticationMethod::ExistingToken { token } => {
                self.authenticate_factory
                    .handle_token_refresh(token, realm.id, auth_session, input.session_code)
                    .await
            }
            AuthenticationMethod::UserCredentials { username, password } => {
                let params = CredentialsAuthParams {
                    realm_name: input.realm_name,
                    client_id: input.client_id,
                    session_code: input.session_code,
                    base_url: input.base_url,
                    username,
                    password,
                };

                self.authenticate_factory
                    .handle_user_credentials_authentication(params, auth_session)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::infrastructure::credential::CredentialRepoAny;
    use crate::infrastructure::refresh_token::RefreshTokenRepoAny;
    use crate::infrastructure::repositories::RepoBundle;
    use crate::domain::authentication::services::grant_type_service::GrantTypeStrategies;
    use crate::infrastructure::user::UserRepoAny;
    use std::future::ready;

    use jsonwebtoken::Header;
    use uuid::Uuid;

    use crate::application::authentication::services::AuthenticateFactory;
    use crate::application::common::permissions::FerriskeyPolicy;
    use crate::application::common::{DefaultJwtService, FerriskeyService};
    use crate::domain::authentication::entities::{
        AuthInput, AuthSession, AuthorizeRequestInput,
    };
    use crate::domain::authentication::ports::AuthService;
    use crate::domain::authentication::value_objects::Identity;
    use crate::domain::client::entities::redirect_uri::RedirectUri;
    use crate::domain::client::entities::{Client, ClientConfig};
    use crate::domain::common::entities::app_errors::CoreError;
    use crate::domain::common::{DatabaseConfig, FerriskeyConfig};
    use crate::domain::jwt::entities::{ClaimsTyp, JwtClaim, JwtKeyPair};
    use crate::domain::realm::entities::Realm;
    use crate::domain::user::entities::{User, UserConfig};
    use crate::infrastructure::auth_session::AuthSessionRepoAny;
    use crate::infrastructure::client::repositories::{ClientRepoAny, RedirectUriRepoAny};
    use crate::infrastructure::jwt::KeyStoreRepoAny;
    use crate::infrastructure::realm::repositories::RealmRepoAny;
    use crate::infrastructure::repositories::test::build_repos_mock;


    // Mocks
    use crate::domain::authentication::ports::test::{get_mock_auth_session_repository_with_clone_expectations};
    use crate::domain::client::ports::test::{get_mock_client_repository_with_clone_expectations, get_mock_redirect_uri_repository_with_clone_expectations};
    use crate::domain::jwt::ports::test::{get_mock_key_store_repository_with_clone_expectations};
    use crate::domain::jwt::services::JwtServiceImpl;
    use crate::domain::realm::ports::test::{get_mock_realm_repository_with_clone_expectations};
    use crate::domain::user::ports::test::{get_mock_user_repository_with_clone_expectations};
    use crate::infrastructure::hasher::HasherRepoAny;
    use crate::infrastructure::user::repositories::user_role_repository::UserRoleRepoAny;

    fn test_config() -> FerriskeyConfig {
        FerriskeyConfig {
            database: DatabaseConfig {
                host: "test".to_string(),
                port: 1495,
                username: "username".to_string(),
                password: "password".to_string(),
                name: "name".to_string(),
            },
        }
    }

    fn build_policy_mock(repos: RepoBundle) -> FerriskeyPolicy {
        FerriskeyPolicy::new(
            repos.user_repository,
            repos.client_repository,
            repos.user_role_repository,
        )
    }

    fn build_grant_type_strategies_mock(repos: RepoBundle) -> GrantTypeStrategies {
        GrantTypeStrategies::new(
            repos.credential_repository,
            repos.hasher_repository,
            repos.auth_session_repository,
            repos.user_repository,
            repos.keystore_repository,
            repos.refresh_token_repository,
            repos.client_repository,
        )
    }

    fn build_authenticate_factory_mock(repos: RepoBundle, jwt_service: DefaultJwtService) -> AuthenticateFactory {
        AuthenticateFactory::new(
            repos.auth_session_repository,
            repos.user_repository,
            repos.realm_repository,
            repos.client_repository,
            repos.credential_repository,
            repos.hasher_repository,
            jwt_service,
        )
    }

    fn build_jwt_service_mock(repos: RepoBundle) -> DefaultJwtService {
        DefaultJwtService::new(
            repos.refresh_token_repository,
            repos.keystore_repository,
            repos.realm_repository,
        )
    }

    fn build_service_from_repos(repos: RepoBundle, policy: FerriskeyPolicy, grant_type_strategies: GrantTypeStrategies, authenticate_factory: AuthenticateFactory) -> FerriskeyService {

        FerriskeyService {
            config: test_config(),
            realm_repository: repos.realm_repository,
            client_repository: repos.client_repository,
            user_repository: repos.user_repository,
            credential_repository: repos.credential_repository,
            hasher_repository: repos.hasher_repository,
            auth_session_repository: repos.auth_session_repository,
            refresh_token_repository: repos.refresh_token_repository,
            redirect_uri_repository: repos.redirect_uri_repository,
            role_repository: repos.role_repository,
            keystore_repository: repos.keystore_repository,
            user_role_repository: repos.user_role_repository,
            user_required_action_repository: repos.user_required_action_repository,
            health_check_repository: repos.health_check_repository,
            webhook_repository: repos.webhook_repository,
            webhook_notifier_repository: repos.webhook_notifier_repository,
            policy,
            grant_type_strategies,
            authenticate_factory,
        }
    }

    #[tokio::test]
    async fn auth_success_returns_login_url_and_persists_session() {
        // Arrange
        let mut realm_repo = get_mock_realm_repository_with_clone_expectations();
        let mut client_repo = get_mock_client_repository_with_clone_expectations();
        let mut redirect_repo = get_mock_redirect_uri_repository_with_clone_expectations();
        let mut session_repo = get_mock_auth_session_repository_with_clone_expectations();

        let realm = Realm::new("master".to_string());
        let realm_name = realm.name.clone();
        let realm_id = realm.id;
        let client = Client::new(ClientConfig {
            realm_id: realm.id,
            name: "app".to_string(),
            client_id: "client123".to_string(),
            secret: None,
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: true,
            service_account_enabled: false,
            client_type: "public".to_string(),
            direct_access_grants_enabled: Some(true),
        });
        let redirect_url = "https://example.com/callback".to_string();
                let redirect = RedirectUri::new(client.id, redirect_url.clone(), true);

        realm_repo
            .expect_get_by_name()
            .returning(move |_| Box::pin(ready(Ok(Some(realm.clone())))));
        let client_clone = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |cid: String, rid: Uuid| {
                assert_eq!(cid, client_clone.client_id);
                assert_eq!(rid, client_clone.realm_id);
                Box::pin(ready(Ok(client_clone.clone())))
            });
        redirect_repo
            .expect_get_enabled_by_client_id()
            .returning(move |id: Uuid| {
                assert_eq!(id, client.id);
                Box::pin(ready(Ok(vec![redirect.clone()])))
            });

        session_repo
            .expect_create()
            .returning(|s: &AuthSession| Box::pin(ready(Ok(s.clone()))));

        let mut repos = build_repos_mock();
        repos.realm_repository = RealmRepoAny::Mock(realm_repo);
        repos.client_repository = ClientRepoAny::Mock(client_repo);
        repos.redirect_uri_repository = RedirectUriRepoAny::Mock(redirect_repo);
        repos.auth_session_repository = AuthSessionRepoAny::Mock(session_repo);

        let service = build_service_from_repos(repos, build_policy_mock(build_repos_mock()), build_grant_type_strategies_mock(build_repos_mock()), build_authenticate_factory_mock(build_repos_mock(), build_jwt_service_mock(build_repos_mock())));

        let input = AuthInput {
            client_id: client.client_id.clone(),
            realm_name: realm_name.clone(),
            redirect_uri: redirect_url.clone(),
            response_type: "code".to_string(),
            scope: Some("openid".to_string()),
            state: Some("abc123".to_string()),
        };

        // Act
        let out = <FerriskeyService as AuthService>::auth(&service, input).await.unwrap();

        // Assert
        assert!(out.login_url.contains("client_id=client123"));
        assert!(out.login_url.contains("redirect_uri=https://example.com/callback"));
        assert!(out.login_url.contains("state=abc123"));
        assert_eq!(out.session.client_id, client.id);
        assert_eq!(out.session.realm_id, realm_id);
    }

    #[tokio::test]
    async fn auth_fails_on_invalid_redirect_uri() {
        let mut realm_repo = get_mock_realm_repository_with_clone_expectations();
        let mut client_repo = get_mock_client_repository_with_clone_expectations();
        let mut redirect_repo = get_mock_redirect_uri_repository_with_clone_expectations();

        let realm = Realm::new("master".to_string());
        let client = Client::new(ClientConfig {
            realm_id: realm.id,
            name: "app".to_string(),
            client_id: "client123".to_string(),
            secret: None,
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: true,
            service_account_enabled: false,
            client_type: "public".to_string(),
            direct_access_grants_enabled: Some(true),
        });

        realm_repo
            .expect_get_by_name()
            .returning(move |_| Box::pin(ready(Ok(Some(realm.clone())))));
        let client_clone = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |_cid: String, _rid: Uuid| Box::pin(ready(Ok(client_clone.clone()))));
        redirect_repo
            .expect_get_enabled_by_client_id()
            .returning(move |_id: Uuid| Box::pin(ready(Ok(vec![]))));

        let mut repos = build_repos_mock();
        repos.realm_repository = RealmRepoAny::Mock(realm_repo);
        repos.client_repository = ClientRepoAny::Mock(client_repo);
        repos.redirect_uri_repository = RedirectUriRepoAny::Mock(redirect_repo);

        let service = build_service_from_repos(repos, build_policy_mock(build_repos_mock()), build_grant_type_strategies_mock(build_repos_mock()), build_authenticate_factory_mock(build_repos_mock(), build_jwt_service_mock(build_repos_mock())));

        let input = AuthInput {
            client_id: client.client_id.clone(),
            realm_name: "master".to_string(),
            redirect_uri: "https://evil.com".to_string(),
            response_type: "code".to_string(),
            scope: None,
            state: None,
        };

        let err = <FerriskeyService as AuthService>::auth(&service, input).await.err().unwrap();
        assert!(matches!(err, CoreError::InvalidClient));
    }

    #[tokio::test]
    async fn get_certs_returns_jwk() {
        // Arrange
        let mut realm_repo = get_mock_realm_repository_with_clone_expectations();
        let mut keystore_repo = get_mock_key_store_repository_with_clone_expectations();

        let realm = Realm::new("master".to_string());
        let realm_for_mock = realm.clone();
        realm_repo
            .expect_get_by_name()
            .returning(move |_| Box::pin(ready(Ok(Some(realm_for_mock.clone())))));

        // generate keypair
        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let realm_id = realm.id;
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm_id, Uuid::new_v4()).unwrap();
        let kid = keypair.id.to_string();

        keystore_repo
            .expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(keypair.clone()))));

        let mut repos = build_repos_mock();
        repos.realm_repository = RealmRepoAny::Mock(realm_repo);
        repos.keystore_repository = KeyStoreRepoAny::Mock(keystore_repo);

        let service = build_service_from_repos(repos, build_policy_mock(build_repos_mock()), build_grant_type_strategies_mock(build_repos_mock()), build_authenticate_factory_mock(build_repos_mock(), build_jwt_service_mock(build_repos_mock())));

        // Act
        let certs = service.get_certs("master".to_string()).await.unwrap();

        // Assert
        assert_eq!(certs.len(), 1);
        assert_eq!(certs[0].kid, kid);
        assert_eq!(certs[0].alg, "RS256");
    }

    #[tokio::test]
    async fn authorize_request_errors_on_non_bearer() {
        let service = build_service_from_repos(build_repos_mock(), build_policy_mock(build_repos_mock()), build_grant_type_strategies_mock(build_repos_mock()), build_authenticate_factory_mock(build_repos_mock(), build_jwt_service_mock(build_repos_mock())));
        let claims = JwtClaim::new(
            Uuid::new_v4(),
            "user".to_string(),
            "iss".to_string(),
            vec!["aud".to_string()],
            ClaimsTyp::Refresh,
            "azp".to_string(),
            None,
        );
        let input = AuthorizeRequestInput { claims, token: "t".to_string() };
        let err = service.authorize_request(input).await.err().unwrap();
        assert!(matches!(err, CoreError::InternalServerError));
    }

    #[tokio::test]
    async fn authorize_request_service_account_returns_client_identity() {
        // Arrange
        let mut user_repo = get_mock_user_repository_with_clone_expectations();
        let mut client_repo = get_mock_client_repository_with_clone_expectations();
        let mut keystore_repo = get_mock_key_store_repository_with_clone_expectations();

        let realm = Realm::new("master".to_string());
        let client = Client::new(ClientConfig {
            realm_id: realm.id,
            name: "svc".to_string(),
            client_id: "client123".to_string(),
            secret: None,
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: true,
            service_account_enabled: true,
            client_type: "service".to_string(),
            direct_access_grants_enabled: Some(true),
        });
        let user = User::new(UserConfig {
            realm_id: realm.id,
            client_id: None,
            username: "alice".to_string(),
            firstname: "Alice".to_string(),
            lastname: "Doe".to_string(),
            email: "alice@example.com".to_string(),
            email_verified: true,
            enabled: true,
        });
        let user_for_mock = user.clone();

        user_repo
            .expect_get_by_id()
            .returning(move |uid: Uuid| {
                assert_eq!(uid, user_for_mock.id);
                Box::pin(ready(Ok(user_for_mock.clone())))
            });
        let client_clone = client.clone();
        client_repo
            .expect_get_by_id()
            .returning(move |id: Uuid| {
                assert_eq!(id, client_clone.id);
                Box::pin(ready(Ok(client_clone.clone())))
            });

        // Generate keys and a valid bearer token
        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm.id, Uuid::new_v4()).unwrap();
        let keypair_for_mock = keypair.clone();
        keystore_repo
            .expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(keypair_for_mock.clone()))));

        // token: signed with same key; verify_token will accept it
        let mut claims = JwtClaim::new(
            user.id,
            user.username.clone(),
            "iss".to_string(),
            vec!["aud".to_string()],
            ClaimsTyp::Bearer,
            client.client_id.clone(),
            Some(user.email.clone()),
        );
        claims.client_id = Some(client.id.to_string());
        let token = jsonwebtoken::encode(&Header::new(jsonwebtoken::Algorithm::RS256), &claims, &keypair.encoding_key).unwrap();

        let mut generic_repos = build_repos_mock();
        generic_repos.user_repository = UserRepoAny::Mock(user_repo);
        generic_repos.client_repository = ClientRepoAny::Mock(client_repo);

        let mut grant_type_strategies_mock = build_repos_mock();
        grant_type_strategies_mock.keystore_repository = KeyStoreRepoAny::Mock(keystore_repo);

        let service = build_service_from_repos(generic_repos, build_policy_mock(build_repos_mock()), build_grant_type_strategies_mock(grant_type_strategies_mock), build_authenticate_factory_mock(build_repos_mock(), build_jwt_service_mock(build_repos_mock())));

        // Act
        let out = service
            .authorize_request(AuthorizeRequestInput { claims, token })
            .await
            .unwrap();

        // Assert
        match out.identity {
            Identity::Client(c) => assert_eq!(c.id, client.id),
            _ => panic!("expected client identity"),
        }
    }

    #[tokio::test]
    async fn auth_fails_when_client_disabled() {
        // Arrange
        let mut realm_repo = get_mock_realm_repository_with_clone_expectations();
        let mut client_repo = get_mock_client_repository_with_clone_expectations();
        let mut redirect_repo = get_mock_redirect_uri_repository_with_clone_expectations();

        let realm = Realm::new("master".to_string());
        let client = Client::new(ClientConfig {
            realm_id: realm.id,
            name: "app".to_string(),
            client_id: "client123".to_string(),
            secret: None,
            enabled: false, // disabled client
            protocol: "openid-connect".to_string(),
            public_client: true,
            service_account_enabled: false,
            client_type: "public".to_string(),
            direct_access_grants_enabled: Some(true),
        });
        let redirect_url = "https://example.com/callback".to_string();
        let redirect = RedirectUri::new(client.id, redirect_url.clone(), true);

        realm_repo
            .expect_get_by_name()
            .returning(move |_| Box::pin(ready(Ok(Some(realm.clone())))));
        let client_clone = client.clone();
        client_repo
            .expect_get_by_client_id()
            .returning(move |_cid: String, _rid: Uuid| Box::pin(ready(Ok(client_clone.clone()))));
        redirect_repo
            .expect_get_enabled_by_client_id()
            .returning(move |_id: Uuid| Box::pin(ready(Ok(vec![redirect.clone()]))));

        let mut repos = build_repos_mock();
        repos.realm_repository = RealmRepoAny::Mock(realm_repo);
        repos.client_repository = ClientRepoAny::Mock(client_repo);
        repos.redirect_uri_repository = RedirectUriRepoAny::Mock(redirect_repo);

        let service = build_service_from_repos(repos, build_policy_mock(build_repos_mock()), build_grant_type_strategies_mock(build_repos_mock()), build_authenticate_factory_mock(build_repos_mock(), build_jwt_service_mock(build_repos_mock())));

        let input = AuthInput {
            client_id: client.client_id.clone(),
            realm_name: "master".to_string(),
            redirect_uri: redirect_url,
            response_type: "code".to_string(),
            scope: None,
            state: None,
        };

        // Act
        let err = <FerriskeyService as AuthService>::auth(&service, input).await.err().unwrap();

        // Assert
        assert!(matches!(err, CoreError::InvalidClient));
    }

    #[tokio::test]
    async fn authorize_request_user_returns_user_identity() {
        // Arrange
        let mut user_repo = get_mock_user_repository_with_clone_expectations();
        let mut keystore_repo = get_mock_key_store_repository_with_clone_expectations();

        let realm = Realm::new("master".to_string());
        let user = User::new(UserConfig {
            realm_id: realm.id,
            client_id: None,
            username: "bob".to_string(),
            firstname: "Bob".to_string(),
            lastname: "Smith".to_string(),
            email: "bob@example.com".to_string(),
            email_verified: true,
            enabled: true,
        });
        let user_for_mock = user.clone();

        user_repo
            .expect_get_by_id()
            .returning(move |uid: Uuid| {
                assert_eq!(uid, user_for_mock.id);
                Box::pin(ready(Ok(user_for_mock.clone())))
            });

        let (priv_pem, pub_pem) = JwtKeyPair::generate().unwrap();
        let keypair = JwtKeyPair::from_pem(&priv_pem, &pub_pem, realm.id, Uuid::new_v4()).unwrap();
        let keypair_for_mock = keypair.clone();
        keystore_repo
            .expect_get_or_generate_key()
            .returning(move |_rid: Uuid| Box::pin(ready(Ok(keypair_for_mock.clone()))));

        let claims = JwtClaim::new(
            user.id,
            user.username.clone(),
            "iss".to_string(),
            vec!["aud".to_string()],
            ClaimsTyp::Bearer,
            "azp".to_string(),
            Some(user.email.clone()),
        );
        let token = jsonwebtoken::encode(&Header::new(jsonwebtoken::Algorithm::RS256), &claims, &keypair.encoding_key).unwrap();

        let mut generic_repos = build_repos_mock();
        generic_repos.user_repository = UserRepoAny::Mock(user_repo);

        let mut gt_repos = build_repos_mock();
        gt_repos.keystore_repository = KeyStoreRepoAny::Mock(keystore_repo);

        let service = build_service_from_repos(generic_repos, build_policy_mock(build_repos_mock()), build_grant_type_strategies_mock(gt_repos), build_authenticate_factory_mock(build_repos_mock(), build_jwt_service_mock(build_repos_mock())));

        // Act
        let out = service
            .authorize_request(AuthorizeRequestInput { claims, token })
            .await
            .unwrap();

        // Assert
        match out.identity {
            Identity::User(u) => assert_eq!(u.id, user.id),
            _ => panic!("expected user identity"),
        }
    }
}
