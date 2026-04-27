use std::sync::Arc;

use ferriskey_compass::recorder::FlowRecorder;

use crate::{
    domain::{
        abyss::{
            BrokerServiceImpl, IdentityProviderServiceImpl,
            federation::services::FederationServiceImpl,
        },
        aegis::services::{
            ClientScopeServiceImpl, ProtocolMapperServiceImpl, ScopeMappingServiceImpl,
        },
        authentication::{mapper_engine::MapperEngine, services::AuthServiceImpl},
        client::services::ClientServiceImpl,
        common::{
            FerriskeyConfig, entities::app_errors::CoreError, policies::FerriskeyPolicy,
            services::CoreServiceImpl,
        },
        compass::services::CompassServiceImpl,
        credential::services::CredentialServiceImpl,
        email_template::services::EmailTemplateServiceImpl,
        email_verification::services::EmailVerificationServiceImpl,
        health::services::HealthServiceImpl,
        maintenance::services::MaintenanceServiceImpl,
        organization::services::OrganizationServiceImpl,
        password_policy::service::PasswordPolicyService,
        realm::services::{MailServiceImpl, RealmServiceImpl},
        role::services::RoleServiceImpl,
        seawatch::services::SecurityEventServiceImpl,
        trident::services::TridentServiceImpl,
        user::services::UserServiceImpl,
        webhook::services::WebhookServiceImpl,
    },
    infrastructure::{
        abyss::federation::repository::FederationRepositoryImpl,
        aegis::repositories::{
            client_scope_postgres_repository::PostgresClientScopeRepository,
            protocol_mapper_postgres_repository::PostgresProtocolMapperRepository,
            scope_mapping_postgres_repository::PostgresScopeMappingRepository,
        },
        client::repositories::{
            client_postgres_repository::PostgresClientRepository,
            post_logout_redirect_uri_postgres_repository::PostgresPostLogoutRedirectUriRepository,
            redirect_uri_postgres_repository::PostgresRedirectUriRepository,
        },
        compass::{
            repositories::{PostgresCompassFlowRepository, PostgresCompassFlowStepRepository},
            writer::compass_writer_task,
        },
        db::postgres::{Postgres, PostgresConfig},
        email::SmtpEmailPort,
        email_template::{
            renderer::mjml_renderer::MjmlTemplateRenderer,
            repositories::email_template_repository::PostgresEmailTemplateRepository,
        },
        health::repositories::PostgresHealthCheckRepository,
        identity_provider::{
            PostgresBrokerAuthSessionRepository, PostgresIdentityProviderLinkRepository,
            PostgresIdentityProviderRepository, ReqwestOAuthClient,
        },
        maintenance::repositories::{
            maintenance_whitelist_repository::PostgresMaintenanceWhitelistRepository,
            realm_maintenance_whitelist_repository::PostgresRealmMaintenanceWhitelistRepository,
        },
        organization::{
            organization_attribute_repository::PostgresOrganizationAttributeRepository,
            organization_member_repository::PostgresOrganizationMemberRepository,
            organization_repository::PostgresOrganizationRepository,
        },
        realm::repositories::{
            realm_postgres_repository::PostgresRealmRepository,
            smtp_config_postgres_repository::PostgresSmtpConfigRepository,
        },
        repositories::{
            access_token_repository::PostgresAccessTokenRepository,
            argon2_hasher::Argon2HasherRepository,
            auth_session_repository::PostgresAuthSessionRepository,
            credential_repository::PostgresCredentialRepository,
            email_verification_token_repository::PostgresEmailVerificationTokenRepository,
            keystore_repository::PostgresKeyStoreRepository,
            magic_link_repository::PostgresMagicLinkRepository,
            password_policy_repository::PostgresPasswordPolicyRepository,
            password_reset_token_repository::PostgresPasswordResetTokenRepository,
            random_bytes_recovery_code::RandBytesRecoveryCodeRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
        },
        role::repositories::role_postgres_repository::PostgresRoleRepository,
        seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository,
        user::{
            repositories::{
                user_attribute_repository::PostgresUserAttributeRepository,
                user_required_action_repository::PostgresUserRequiredActionRepository,
                user_role_repository::PostgresUserRoleRepository,
            },
            repository::PostgresUserRepository,
        },
        webhook::repositories::webhook_repository::PostgresWebhookRepository,
    },
};

pub mod services;

pub mod abyss;
pub mod aegis;
pub mod auth;
pub mod broker;
pub mod client;
pub mod compass;
pub mod credential;
pub mod email_template;
pub mod health;
pub mod identity_provider;
pub mod mail;
pub mod maintenance;
pub mod migrate;
pub mod organization;
pub mod realm;
pub mod role;
pub mod seawatch;
pub mod trident;
pub mod user;
pub mod webhook;

pub use services::ApplicationService;

pub async fn create_service(config: FerriskeyConfig) -> Result<ApplicationService, CoreError> {
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}?options=-c search_path={}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name,
        urlencoding::encode(&config.database.schema)
    );

    let postgres = Postgres::new(PostgresConfig { database_url })
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let realm = Arc::new(PostgresRealmRepository::new(postgres.get_db()));
    let client = Arc::new(PostgresClientRepository::new(postgres.get_db()));
    let user = Arc::new(PostgresUserRepository::new(postgres.get_db()));
    let credential = Arc::new(PostgresCredentialRepository::new(postgres.get_db()));
    let hasher = Arc::new(Argon2HasherRepository::new());
    let auth_session = Arc::new(PostgresAuthSessionRepository::new(postgres.get_db()));
    let redirect_uri = Arc::new(PostgresRedirectUriRepository::new(postgres.get_db()));
    let post_logout_redirect_uri = Arc::new(PostgresPostLogoutRedirectUriRepository::new(
        postgres.get_db(),
    ));
    let role = Arc::new(PostgresRoleRepository::new(postgres.get_db()));
    let keystore = Arc::new(PostgresKeyStoreRepository::new(postgres.get_db()));
    let user_role = Arc::new(PostgresUserRoleRepository::new(postgres.get_db()));
    let user_required_action =
        Arc::new(PostgresUserRequiredActionRepository::new(postgres.get_db()));
    let user_attribute = Arc::new(PostgresUserAttributeRepository::new(postgres.get_db()));
    let health_check = Arc::new(PostgresHealthCheckRepository::new(postgres.get_db()));
    let webhook = Arc::new(PostgresWebhookRepository::new(postgres.get_db()));
    let refresh_token = Arc::new(PostgresRefreshTokenRepository::new(postgres.get_db()));
    let access_token = Arc::new(PostgresAccessTokenRepository::new(postgres.get_db()));
    let recovery_code = Arc::new(RandBytesRecoveryCodeRepository::new(hasher.clone()));
    let security_event = Arc::new(PostgresSecurityEventRepository::new(postgres.get_db()));
    let identity_provider = Arc::new(PostgresIdentityProviderRepository::new(postgres.get_db()));
    let federation = Arc::new(FederationRepositoryImpl::new(postgres.get_db()));
    let broker_auth_session = Arc::new(PostgresBrokerAuthSessionRepository::new(postgres.get_db()));
    let identity_provider_link = Arc::new(PostgresIdentityProviderLinkRepository::new(
        postgres.get_db(),
    ));
    let oauth_client = Arc::new(ReqwestOAuthClient::new());
    let magic_link = Arc::new(PostgresMagicLinkRepository::new(postgres.get_db()));
    let client_scope = Arc::new(PostgresClientScopeRepository::new(postgres.get_db()));
    let protocol_mapper = Arc::new(PostgresProtocolMapperRepository::new(postgres.get_db()));
    let scope_mapping = Arc::new(PostgresScopeMappingRepository::new(postgres.get_db()));
    let compass_flow = Arc::new(PostgresCompassFlowRepository::new(postgres.get_db()));
    let compass_flow_step = Arc::new(PostgresCompassFlowStepRepository::new(postgres.get_db()));
    let smtp_config = Arc::new(PostgresSmtpConfigRepository::new(postgres.get_db()));
    let email_port = Arc::new(SmtpEmailPort::new());
    let password_reset_token =
        Arc::new(PostgresPasswordResetTokenRepository::new(postgres.get_db()));

    let email_template = Arc::new(PostgresEmailTemplateRepository::new(postgres.get_db()));
    let mjml_renderer = Arc::new(MjmlTemplateRenderer::new());
    let organization = Arc::new(PostgresOrganizationRepository::new(postgres.get_db()));
    let organization_attribute = Arc::new(PostgresOrganizationAttributeRepository::new(
        postgres.get_db(),
    ));
    let organization_member =
        Arc::new(PostgresOrganizationMemberRepository::new(postgres.get_db()));
    let email_verification_token_repo = Arc::new(PostgresEmailVerificationTokenRepository::new(
        postgres.get_db(),
    ));

    let maintenance_whitelist = Arc::new(PostgresMaintenanceWhitelistRepository::new(
        postgres.get_db(),
    ));
    let realm_maintenance_whitelist = Arc::new(PostgresRealmMaintenanceWhitelistRepository::new(
        postgres.get_db(),
    ));

    let (compass_tx, compass_rx) = tokio::sync::mpsc::channel(1024);
    tokio::spawn(compass_writer_task(
        compass_rx,
        PostgresCompassFlowRepository::new(postgres.get_db()),
        PostgresCompassFlowStepRepository::new(postgres.get_db()),
    ));
    let flow_recorder = FlowRecorder::new(compass_tx);

    let policy = Arc::new(FerriskeyPolicy::new(
        user.clone(),
        client.clone(),
        user_role.clone(),
    ));

    let email_verification_service = EmailVerificationServiceImpl::new(
        email_verification_token_repo,
        user.clone(),
        realm.clone(),
        user_required_action.clone(),
        email_port.clone(),
        smtp_config.clone(),
        email_template.clone(),
        mjml_renderer.clone(),
    );

    let app = ApplicationService {
        maintenance_service: MaintenanceServiceImpl::new(
            realm.clone(),
            client.clone(),
            webhook.clone(),
            security_event.clone(),
            maintenance_whitelist.clone(),
            realm_maintenance_whitelist.clone(),
            policy.clone(),
        ),
        auth_service: AuthServiceImpl::new(
            realm.clone(),
            client.clone(),
            redirect_uri.clone(),
            post_logout_redirect_uri.clone(),
            user.clone(),
            user_role.clone(),
            credential.clone(),
            hasher.clone(),
            auth_session.clone(),
            keystore.clone(),
            refresh_token.clone(),
            access_token.clone(),
            federation.clone(),
            scope_mapping.clone(),
            protocol_mapper.clone(),
            organization_member.clone(),
            organization.clone(),
            organization_attribute.clone(),
            user_required_action.clone(),
            maintenance_whitelist.clone(),
            realm_maintenance_whitelist.clone(),
            user_attribute.clone(),
            email_verification_service.clone(),
            Arc::new(MapperEngine::new()),
            flow_recorder.clone(),
        ),
        client_service: ClientServiceImpl::new(
            realm.clone(),
            user.clone(),
            client.clone(),
            webhook.clone(),
            redirect_uri.clone(),
            post_logout_redirect_uri.clone(),
            role.clone(),
            security_event.clone(),
            client_scope.clone(),
            scope_mapping.clone(),
            policy.clone(),
        ),
        credential_service: CredentialServiceImpl::new(
            realm.clone(),
            credential.clone(),
            policy.clone(),
        ),
        health_service: HealthServiceImpl::new(health_check.clone()),
        realm_service: RealmServiceImpl::new(
            realm.clone(),
            user.clone(),
            user_role.clone(),
            role.clone(),
            client.clone(),
            webhook.clone(),
            identity_provider.clone(),
            client_scope.clone(),
            protocol_mapper.clone(),
            scope_mapping.clone(),
            redirect_uri.clone(),
            policy.clone(),
        ),
        mail_service: MailServiceImpl::new(realm.clone(), smtp_config.clone(), policy.clone()),
        role_service: RoleServiceImpl::new(
            realm.clone(),
            role.clone(),
            security_event.clone(),
            webhook.clone(),
            user_role.clone(),
            policy.clone(),
        ),
        security_event_service: SecurityEventServiceImpl::new(
            realm.clone(),
            security_event.clone(),
            policy.clone(),
        ),
        trident_service: TridentServiceImpl::new(
            credential.clone(),
            recovery_code.clone(),
            auth_session.clone(),
            hasher.clone(),
            user_required_action.clone(),
            magic_link.clone(),
            user.clone(),
            realm.clone(),
            email_port.clone(),
            smtp_config.clone(),
            password_reset_token.clone(),
            security_event.clone(),
            webhook.clone(),
            email_template.clone(),
            mjml_renderer.clone(),
        ),
        user_service: UserServiceImpl::new(
            realm.clone(),
            user.clone(),
            credential.clone(),
            hasher.clone(),
            user_role.clone(),
            role.clone(),
            user_required_action.clone(),
            user_attribute.clone(),
            webhook.clone(),
            security_event.clone(),
            policy.clone(),
        ),
        webhook_service: WebhookServiceImpl::new(realm.clone(), webhook.clone(), policy.clone()),
        email_template_service: EmailTemplateServiceImpl::new(
            realm.clone(),
            email_template.clone(),
            mjml_renderer.clone(),
            policy.clone(),
        ),
        core_service: CoreServiceImpl::new(
            realm.clone(),
            keystore.clone(),
            client.clone(),
            user.clone(),
            role.clone(),
            user_role.clone(),
            hasher.clone(),
            credential.clone(),
            redirect_uri.clone(),
        ),
        identity_provider_service: IdentityProviderServiceImpl::new(
            identity_provider.clone(),
            policy.clone(),
            realm.clone(),
        ),
        federation_service: FederationServiceImpl::new(
            realm.clone(),
            federation.clone(),
            user.clone(),
            credential.clone(),
            policy.clone(),
        ),
        broker_service: BrokerServiceImpl::new(
            realm.clone(),
            identity_provider.clone(),
            broker_auth_session.clone(),
            identity_provider_link.clone(),
            client.clone(),
            redirect_uri.clone(),
            user.clone(),
            auth_session.clone(),
            oauth_client.clone(),
            flow_recorder.clone(),
        ),
        client_scope_service: ClientScopeServiceImpl::new(
            realm.clone(),
            client_scope.clone(),
            protocol_mapper.clone(),
            policy.clone(),
        ),
        protocol_mapper_service: ProtocolMapperServiceImpl::new(
            realm.clone(),
            client_scope.clone(),
            protocol_mapper.clone(),
            policy.clone(),
        ),
        scope_mapping_service: ScopeMappingServiceImpl::new(
            realm.clone(),
            client_scope.clone(),
            scope_mapping.clone(),
            policy.clone(),
        ),
        compass_service: CompassServiceImpl::new(
            realm.clone(),
            compass_flow.clone(),
            compass_flow_step.clone(),
            policy.clone(),
        ),
        password_policy_service: PasswordPolicyService::new(
            Arc::new(PostgresPasswordPolicyRepository::new(postgres.get_db())),
            policy.clone(),
        ),
        organization_service: OrganizationServiceImpl::new(
            realm.clone(),
            user.clone(),
            organization.clone(),
            organization_attribute.clone(),
            organization_member.clone(),
            policy.clone(),
        ),
        flow_recorder,
        db: postgres.get_db(),
        email_verification_service,
    };

    Ok(app)
}

#[cfg(test)]
mod tests {
    use std::env;

    use sqlx::Executor;
    use uuid::Uuid;

    use crate::{
        application::create_service,
        domain::{
            authentication::value_objects::Identity,
            client::{
                entities::{ClientType, CreateClientInput},
                ports::ClientService,
            },
            common::FerriskeyConfig,
            realm::entities::Realm,
            realm::ports::{RealmRepository, RealmService},
            role::{
                entities::permission::Permissions, ports::RoleRepository,
                value_objects::CreateRoleRequest,
            },
            user::{
                entities::User,
                ports::{UserRepository, UserRoleRepository},
                value_objects::CreateUserRequest,
            },
        },
    };

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(default)
    }

    #[tokio::test]
    #[ignore]
    async fn creates_client_with_realm_default_scopes_assigned() {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("test_schema_{}", Uuid::new_v4().simple());

        let admin_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let admin_pool = sqlx::PgPool::connect(&admin_url)
            .await
            .expect("connect admin pool");

        admin_pool
            .execute(format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema).as_str())
            .await
            .expect("create schema");

        let schema_url = format!(
            "postgres://{}:{}@{}:{}/{}?options=-c search_path={}",
            db_user,
            db_password,
            db_host,
            db_port,
            db_name,
            urlencoding::encode(&schema)
        );

        let pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let app = create_service(FerriskeyConfig {
            database: crate::domain::common::DatabaseConfig {
                host: db_host,
                port: db_port,
                username: db_user,
                password: db_password,
                name: db_name,
                schema: schema.clone(),
            },
        })
        .await
        .expect("create service");

        let realm_name = format!("realm-{}", Uuid::new_v4().simple());
        let realm = app
            .realm_service
            .realm_repository
            .create_realm(realm_name.clone())
            .await
            .expect("create realm");

        app.seed_default_scopes(realm.id)
            .await
            .expect("seed default scopes");

        let role = app
            .realm_service
            .role_repository
            .create(CreateRoleRequest {
                client_id: None,
                description: None,
                name: "test-manage-clients".to_string(),
                permissions: vec![Permissions::ManageClients.name()],
                realm_id: realm.id,
            })
            .await
            .expect("create role");

        let user = app
            .realm_service
            .user_repository
            .create_user(CreateUserRequest {
                realm_id: realm.id,
                client_id: None,
                username: "test-user".to_string(),
                firstname: "Test".to_string(),
                lastname: "User".to_string(),
                email: "test-user@example.com".to_string(),
                email_verified: true,
                enabled: true,
            })
            .await
            .expect("create user");

        app.realm_service
            .user_role_repository
            .assign_role(user.id, role.id)
            .await
            .expect("assign role");

        let identity = Identity::User(User {
            realm: Some(Realm {
                id: realm.id,
                name: realm.name.clone(),
                settings: None,
                created_at: realm.created_at,
                updated_at: realm.updated_at,
            }),
            ..user.clone()
        });

        let client = app
            .create_client(
                identity,
                CreateClientInput {
                    client_id: format!("client-{}", Uuid::new_v4().simple()),
                    client_type: ClientType::Public,
                    public_client: true,
                    realm_name: realm_name.clone(),
                    enabled: true,
                    name: "Test Client".to_string(),
                    protocol: "openid-connect".to_string(),
                    service_account_enabled: false,
                    direct_access_grants_enabled: false,
                },
            )
            .await
            .expect("create client");

        let default_scopes: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM client_scopes WHERE realm_id = $1 AND is_default = true",
        )
        .bind::<Uuid>(realm.id.into())
        .fetch_all(&pool)
        .await
        .expect("fetch default scopes");

        assert!(!default_scopes.is_empty());

        for (scope_id,) in default_scopes {
            let row: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM client_scope_mappings WHERE client_id = $1 AND client_scope_id = $2 AND is_default = true AND is_optional = false",
            )
            .bind(client.id)
            .bind(scope_id)
            .fetch_one(&pool)
            .await
            .expect("count mapping");

            assert_eq!(row.0, 1);
        }

        let optional_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM client_scope_mappings m JOIN client_scopes s ON s.id = m.client_scope_id WHERE m.client_id = $1 AND s.realm_id = $2 AND s.is_default = false",
        )
        .bind(client.id)
        .bind::<Uuid>(realm.id.into())
        .fetch_one(&pool)
        .await
        .expect("count optional mappings");

        assert_eq!(optional_count.0, 0);

        admin_pool
            .execute(format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema).as_str())
            .await
            .expect("drop schema");
    }
}
