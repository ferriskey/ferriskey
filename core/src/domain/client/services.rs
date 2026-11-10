use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::{
        entities::{
            Client, ClientFilter, ClientRegistrationSource, ClientSortField, CreateClientInput,
            CreatePostLogoutRedirectUriInput, CreateRedirectUriInput, CreateRoleInput,
            CreateSamlAttributeMapperInput, CreateWebOriginInput, DeleteClientInput,
            DeletePostLogoutRedirectUriInput, DeleteRedirectUriInput,
            DeleteSamlAttributeMapperInput, DeleteWebOriginInput, GetClientInput,
            GetClientRolesInput, GetClientSamlConfigInput, GetPostLogoutRedirectUrisInput,
            GetRedirectUrisInput, GetSamlAttributeMappersInput, GetWebOriginsInput,
            RevealedClientSecret, RevealedPreviousSecret, RotateClientSecretInput,
            RotatedClientSecret, SetClientSamlConfigInput, UpdateClientInput,
            UpdatePostLogoutRedirectUriInput, UpdateRedirectUriInput,
            redirect_uri::RedirectUri,
            saml::{
                AcsUrl, ClientSamlConfig, NameIdFormat, SamlAttributeMapper,
                SamlAttributeMapperDefinition, SamlAttributeName, SamlAttributeNameFormat,
                SamlAttributeSource, SamlConfigSettings, SpEntityId,
            },
            web_origin::{Origin, WebOrigin, WebOriginValue},
        },
        ports::{
            ClientPolicy, ClientRepository, ClientSamlRepository, ClientService,
            PostLogoutRedirectUriRepository, RedirectUriRepository, WebOriginRepository,
            WebOriginResolver,
        },
        value_objects::{CreateClientRequest, UpdateClientRequest},
        web_origin_resolution::resolve_allowed_origins,
    },
    common::{
        entities::app_errors::CoreError,
        generate_random_string,
        pagination::{Page, PageRequest},
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::{
        entities::{Realm, RealmScope, Scoped},
        ports::RealmRepository,
    },
    role::{
        entities::Role,
        ports::{RolePolicy, RoleRepository},
        value_objects::CreateRoleRequest,
    },
    seawatch::{ActorType, EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType},
    user::{
        ports::{UserPolicy, UserRepository, UserRoleRepository},
        value_objects::CreateUserRequest,
    },
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};
use ferriskey_aegis::ports::{ClientScopeMappingRepository, ClientScopeRepository};
use tracing::warn;

#[derive(Clone, Debug)]
pub struct ClientServiceImpl<R, U, C, UR, W, RU, PLRU, WO, SA, RO, SE, CS, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    WO: WebOriginRepository,
    SA: ClientSamlRepository,
    RO: RoleRepository,
    SE: SecurityEventRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) webhook_repository: Arc<W>,
    pub(crate) redirect_uri_repository: Arc<RU>,
    pub(crate) post_logout_redirect_uri_repository: Arc<PLRU>,
    pub(crate) web_origin_repository: Arc<WO>,
    pub(crate) client_saml_repository: Arc<SA>,
    pub(crate) role_repository: Arc<RO>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) client_scope_repository: Arc<CS>,
    pub(crate) scope_mapping_repository: Arc<CSM>,

    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, W, RU, PLRU, WO, SA, RO, SE, CS, CSM>
    ClientServiceImpl<R, U, C, UR, W, RU, PLRU, WO, SA, RO, SE, CS, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    WO: WebOriginRepository,
    SA: ClientSamlRepository,
    RO: RoleRepository,
    SE: SecurityEventRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        user_repository: Arc<U>,
        client_repository: Arc<C>,
        webhook_repository: Arc<W>,
        redirect_uri_repository: Arc<RU>,
        post_logout_redirect_uri_repository: Arc<PLRU>,
        web_origin_repository: Arc<WO>,
        client_saml_repository: Arc<SA>,
        role_repository: Arc<RO>,
        security_event_repository: Arc<SE>,
        client_scope_repository: Arc<CS>,
        scope_mapping_repository: Arc<CSM>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            user_repository,
            client_repository,
            webhook_repository,
            redirect_uri_repository,
            post_logout_redirect_uri_repository,
            web_origin_repository,
            client_saml_repository,
            role_repository,
            security_event_repository,
            client_scope_repository,
            scope_mapping_repository,
            policy,
        }
    }

    // A client secret is the credential of the client's service account, so
    // handing it over hands that account over: the subject must hold what
    // the account holds, as for a password reset.
    async fn ensure_covers_service_account(
        &self,
        identity: &Identity,
        realm: &Realm,
        client_id: Uuid,
        action: &'static str,
    ) -> Result<(), CoreError> {
        let service_account = match self.user_repository.get_by_client_id(client_id).await {
            Ok(user) => user.across_realms(),
            Err(CoreError::NotFound) => return Ok(()),
            Err(error) => return Err(error),
        };

        if self
            .policy
            .can_manage_user(identity, realm, &service_account)
            .await?
        {
            return Ok(());
        }

        warn!(
            %client_id,
            action,
            "Refused to hand over the secret of a client whose service account holds permissions the subject does not hold"
        );
        Err(CoreError::Forbidden(format!(
            "cannot {action} the secret of a client holding permissions you do not hold"
        )))
    }
}

fn ensure_update_allowed_for_source(
    client: &Client,
    payload: &UpdateClientRequest,
) -> Result<(), CoreError> {
    if client.registration_source == ClientRegistrationSource::Admin {
        return Ok(());
    }

    if payload.token_exchange_enabled == Some(true) {
        return Err(CoreError::Forbidden(
            "token exchange cannot be enabled on a self-registered client".to_string(),
        ));
    }

    if payload.direct_access_grants_enabled == Some(true) {
        return Err(CoreError::Forbidden(
            "direct access grants cannot be enabled on a self-registered client".to_string(),
        ));
    }

    if payload.oauth_device_code_grant_enabled == Some(true) {
        return Err(CoreError::Forbidden(
            "the device code grant cannot be enabled on a self-registered client".to_string(),
        ));
    }

    if payload.client_id.is_some() {
        return Err(CoreError::Forbidden(
            "the client_id of a self-registered client cannot change".to_string(),
        ));
    }

    Ok(())
}

impl<R, U, C, UR, W, RU, PLRU, WO, SA, RO, SE, CS, CSM> ClientService
    for ClientServiceImpl<R, U, C, UR, W, RU, PLRU, WO, SA, RO, SE, CS, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    WO: WebOriginRepository,
    SA: ClientSamlRepository,
    RO: RoleRepository,
    SE: SecurityEventRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    async fn create_client(
        &self,
        identity: Identity,
        input: CreateClientInput,
    ) -> Result<Client, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_create_client(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let secret = (!input.public_client).then(generate_random_string);

        let client = self
            .client_repository
            .create_client(CreateClientRequest {
                realm_id,
                name: input.name,
                client_id: input.client_id.clone(),
                secret,
                enabled: input.enabled,
                protocol: input.protocol,
                public_client: input.public_client,
                service_account_enabled: input.service_account_enabled,
                direct_access_grants_enabled: input.direct_access_grants_enabled,
                oauth_device_code_grant_enabled: input.oauth_device_code_grant_enabled,
                token_exchange_enabled: input.token_exchange_enabled,
                client_type: input.client_type,
                require_pkce: input.require_pkce,
                consent_required: input.consent_required,
                registration_source:
                    crate::domain::client::entities::ClientRegistrationSource::Admin,
            })
            .await?;

        let realm_scopes = self
            .client_scope_repository
            .find_by_realm_id(realm_id)
            .await?;

        for scope in realm_scopes
            .into_iter()
            .filter(|s| s.default_scope_type == ferriskey_aegis::entities::ScopeType::Default)
        {
            self.scope_mapping_repository
                .assign_scope_to_client(client.id, scope.id, true, false)
                .await?;
        }

        if input.service_account_enabled {
            let service_account_username = format!("service-account-{}", input.client_id);
            let service_account_email = format!("{}@serviceaccount.local", input.client_id);

            self.user_repository
                .create_user(CreateUserRequest {
                    id: None,
                    realm_id,
                    client_id: Some(client.id),
                    username: service_account_username,
                    firstname: Some("Service".to_string()),
                    lastname: Some("Account".to_string()),
                    email: Some(service_account_email),
                    email_verified: true,
                    enabled: true,
                })
                .await
                .map_err(|_| CoreError::InternalServerError)?;
        }

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::ClientCreated,
                    realm_id.into(),
                    Some(client.clone()),
                ),
            )
            .await?;

        self.security_event_repository
            .store_event(SecurityEvent::new(
                realm_id,
                SecurityEventType::ClientCreated,
                EventStatus::Success,
                identity.id(),
            ))
            .await?;

        Ok(client)
    }

    async fn create_redirect_uri(
        &self,
        identity: Identity,
        input: CreateRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        let redirect_uri = self
            .redirect_uri_repository
            .create_redirect_uri(input.client_id, input.payload.value, input.payload.enabled)
            .await
            .map_err(|_| CoreError::InvalidRedirectUri)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriCreated,
                    realm_id.into(),
                    Some(redirect_uri.clone()),
                ),
            )
            .await?;

        Ok(redirect_uri)
    }

    async fn create_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: CreatePostLogoutRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        let redirect_uri = self
            .post_logout_redirect_uri_repository
            .create_redirect_uri(input.client_id, input.payload.value, input.payload.enabled)
            .await
            .map_err(|_| CoreError::InvalidRedirectUri)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriCreated,
                    realm_id.into(),
                    Some(redirect_uri.clone()),
                ),
            )
            .await?;

        Ok(redirect_uri)
    }

    async fn create_web_origin(
        &self,
        identity: Identity,
        input: CreateWebOriginInput,
    ) -> Result<WebOrigin, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        let value = WebOriginValue::from_str(&input.payload.value)?;

        let web_origin = self
            .web_origin_repository
            .create(input.client_id, value)
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::WebOriginCreated,
                    realm_id.into(),
                    Some(web_origin.clone()),
                ),
            )
            .await?;

        Ok(web_origin)
    }

    async fn get_web_origins(
        &self,
        identity: Identity,
        input: GetWebOriginsInput,
    ) -> Result<Vec<WebOrigin>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.web_origin_repository
            .get_by_client_id(input.client_id)
            .await
    }

    async fn delete_web_origin(
        &self,
        identity: Identity,
        input: DeleteWebOriginInput,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.web_origin_repository
            .delete(input.client_id, input.web_origin_id)
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::WebOriginDeleted,
                    realm_id.into(),
                    Some(input.web_origin_id),
                ),
            )
            .await?;

        Ok(())
    }

    async fn get_client_saml_config(
        &self,
        identity: Identity,
        input: GetClientSamlConfigInput,
    ) -> Result<ClientSamlConfig, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.client_saml_repository
            .get_config_by_client_id(input.client_id)
            .await?
            .ok_or(CoreError::SamlConfigNotFound)?
            .in_realm(&scope)
            .map(Scoped::into_inner)
    }

    async fn set_client_saml_config(
        &self,
        identity: Identity,
        input: SetClientSamlConfigInput,
    ) -> Result<ClientSamlConfig, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?
            .get()
            .ensure_speaks_saml()?;

        let settings = SamlConfigSettings {
            sp_entity_id: SpEntityId::from_str(&input.payload.sp_entity_id)?,
            acs_url: AcsUrl::from_str(&input.payload.acs_url)?,
            name_id_format: NameIdFormat::from_str(&input.payload.name_id_format)?,
            sign_assertions: input.payload.sign_assertions,
            sign_documents: input.payload.sign_documents,
            want_authn_requests_signed: input.payload.want_authn_requests_signed,
        };

        let config = self
            .client_saml_repository
            .upsert_config(realm_id, input.client_id, settings)
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::ClientSamlConfigUpdated,
                    realm_id.into(),
                    Some(config.clone()),
                ),
            )
            .await?;

        Ok(config)
    }

    async fn create_saml_attribute_mapper(
        &self,
        identity: Identity,
        input: CreateSamlAttributeMapperInput,
    ) -> Result<SamlAttributeMapper, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?
            .get()
            .ensure_speaks_saml()?;

        let definition = SamlAttributeMapperDefinition {
            name: SamlAttributeName::from_str(&input.payload.name)?,
            name_format: SamlAttributeNameFormat::from_str(&input.payload.name_format)?,
            source: SamlAttributeSource::from_str(&input.payload.source)?,
        };

        let mapper = self
            .client_saml_repository
            .create_attribute_mapper(input.client_id, definition)
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::SamlAttributeMapperCreated,
                    realm_id.into(),
                    Some(mapper.clone()),
                ),
            )
            .await?;

        Ok(mapper)
    }

    async fn get_saml_attribute_mappers(
        &self,
        identity: Identity,
        input: GetSamlAttributeMappersInput,
    ) -> Result<Vec<SamlAttributeMapper>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.client_saml_repository
            .get_attribute_mappers_by_client_id(input.client_id)
            .await
    }

    async fn delete_saml_attribute_mapper(
        &self,
        identity: Identity,
        input: DeleteSamlAttributeMapperInput,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.client_saml_repository
            .delete_attribute_mapper(input.client_id, input.mapper_id)
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::SamlAttributeMapperDeleted,
                    realm_id.into(),
                    Some(input.mapper_id),
                ),
            )
            .await?;

        Ok(())
    }

    async fn create_role(
        &self,
        identity: Identity,
        input: CreateRoleInput,
    ) -> Result<Role, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_role(&identity, &realm).await,
            "insufficient permissions",
        )?;

        if !self
            .policy
            .can_grant_permissions(&identity, &realm, &input.permissions)
            .await?
        {
            return Err(CoreError::Forbidden(
                "cannot grant a permission you do not hold".to_string(),
            ));
        }
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        let role = self
            .role_repository
            .create(CreateRoleRequest {
                client_id: Some(input.client_id),
                description: input.description,
                name: input.name,
                permissions: input.permissions,
                realm_id,
            })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.security_event_repository
            .store_event(SecurityEvent::new(
                realm_id,
                SecurityEventType::RoleCreated,
                EventStatus::Success,
                identity.id(),
            ))
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RoleCreated,
                    realm_id.into(),
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(role)
    }

    async fn delete_client(
        &self,
        identity: Identity,
        input: DeleteClientInput,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_client(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let client = self
            .client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.client_repository.delete_by_id(&client).await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::ClientDeleted,
                    realm_id.into(),
                    Some(input.client_id),
                ),
            )
            .await?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::ClientDeleted,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("client".to_string(), input.client_id, None),
            )
            .await?;

        Ok(())
    }

    async fn delete_redirect_uri(
        &self,
        identity: Identity,
        input: DeleteRedirectUriInput,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.redirect_uri_repository
            .delete(input.client_id, input.uri_id)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriDeleted,
                    realm_id.into(),
                    Some(input.uri_id),
                ),
            )
            .await?;

        Ok(())
    }

    async fn delete_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: DeletePostLogoutRedirectUriInput,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.post_logout_redirect_uri_repository
            .delete(input.client_id, input.uri_id)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriDeleted,
                    realm_id.into(),
                    Some(input.uri_id),
                ),
            )
            .await?;

        Ok(())
    }

    async fn get_client_by_id(
        &self,
        identity: Identity,
        input: GetClientInput,
    ) -> Result<Client, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)
            .map(Scoped::into_inner)
    }

    async fn reveal_client_secret(
        &self,
        identity: Identity,
        input: GetClientInput,
    ) -> Result<RevealedClientSecret, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let client = self
            .client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.ensure_covers_service_account(&identity, &realm, client.get().id, "reveal")
            .await?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::ClientSecretViewed,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("client".to_string(), client.get().id, None),
            )
            .await?;

        let now = Utc::now();
        let previous = client
            .get()
            .previous_secret
            .as_ref()
            .filter(|previous| previous.is_valid_at(now))
            .map(|previous| RevealedPreviousSecret {
                client_secret: previous.secret.expose().clone(),
                expires_at: previous.expires_at,
            });

        Ok(RevealedClientSecret {
            client_secret: client.get().secret_str().map(str::to_string),
            previous,
        })
    }

    async fn rotate_client_secret(
        &self,
        identity: Identity,
        input: RotateClientSecretInput,
    ) -> Result<RotatedClientSecret, CoreError> {
        let grace_period = input.grace_period()?;
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let client = self
            .client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        if client.get().public_client || client.get().secret.is_none() {
            return Err(CoreError::ClientHasNoSecret);
        }

        self.ensure_covers_service_account(&identity, &realm, client.get().id, "rotate")
            .await?;

        let new_secret = generate_random_string();
        let previous_expires_at = grace_period.map(|grace| Utc::now() + grace);

        self.client_repository
            .rotate_secret(&client, new_secret.clone(), previous_expires_at)
            .await?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::ClientSecretRotated,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("client".to_string(), client.get().id, None)
                .with_details(serde_json::json!({
                    "previous_secret_expires_at": previous_expires_at,
                })),
            )
            .await?;

        Ok(RotatedClientSecret {
            client_secret: new_secret,
            previous_secret_expires_at: previous_expires_at,
        })
    }

    async fn revoke_previous_client_secret(
        &self,
        identity: Identity,
        input: GetClientInput,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let client = self
            .client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        if !self
            .client_repository
            .revoke_previous_secret(&client)
            .await?
        {
            return Err(CoreError::NotFound);
        }

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::ClientSecretRevoked,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("client".to_string(), client.get().id, None),
            )
            .await?;

        Ok(())
    }

    async fn expire_previous_client_secrets(&self) -> Result<usize, CoreError> {
        let expired = self
            .client_repository
            .expire_previous_secrets(Utc::now())
            .await?;

        for secret in &expired {
            let mut event = SecurityEvent::without_actor(
                secret.realm_id,
                SecurityEventType::ClientSecretExpired,
                EventStatus::Success,
            )
            .with_target("client".to_string(), secret.client_id, None)
            .with_details(serde_json::json!({ "expired_at": secret.expired_at }));
            event.actor_type = Some(ActorType::System);

            if let Err(error) = self.security_event_repository.store_event(event).await {
                tracing::error!(
                    client = %secret.client_id,
                    ?error,
                    "failed to record the expiry of a previous client secret"
                );
            }
        }

        Ok(expired.len())
    }

    async fn get_client_roles(
        &self,
        identity: Identity,
        input: GetClientRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.role_repository
            .get_by_client_id(input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)
    }

    async fn list_clients(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<ClientFilter, ClientSortField>,
    ) -> Result<Page<Client>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_view_client(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        self.client_repository.list(&scope, &request).await
    }

    async fn get_redirect_uris(
        &self,
        identity: Identity,
        input: GetRedirectUrisInput,
    ) -> Result<Vec<RedirectUri>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.redirect_uri_repository
            .get_by_client_id(input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)
    }

    async fn get_post_logout_redirect_uris(
        &self,
        identity: Identity,
        input: GetPostLogoutRedirectUrisInput,
    ) -> Result<Vec<RedirectUri>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        self.post_logout_redirect_uri_repository
            .get_by_client_id(input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)
    }

    async fn update_client(
        &self,
        identity: Identity,
        input: UpdateClientInput,
    ) -> Result<Client, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let target = self
            .client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        ensure_update_allowed_for_source(target.get(), &input.payload)?;

        let client = self
            .client_repository
            .update_client(&target, input.payload)
            .await
            .map_err(|_| CoreError::NotFound)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::ClientUpdated,
                    realm_id.into(),
                    Some(client.clone()),
                ),
            )
            .await?;

        Ok(client)
    }

    async fn update_redirect_uri(
        &self,
        identity: Identity,
        input: UpdateRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        let redirect_uri = self
            .redirect_uri_repository
            .update_enabled(input.client_id, input.redirect_uri_id, input.enabled)
            .await
            .map_err(|_| CoreError::NotFound)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriUpdated,
                    realm_id.into(),
                    Some(redirect_uri.clone()),
                ),
            )
            .await?;

        Ok(redirect_uri)
    }

    async fn update_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: UpdatePostLogoutRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_update_client(&identity, &realm).await,
            "insufficient permissions",
        )?;
        self.client_repository
            .get_by_id(scope.id(), input.client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;

        let redirect_uri = self
            .post_logout_redirect_uri_repository
            .update_enabled(input.client_id, input.redirect_uri_id, input.enabled)
            .await
            .map_err(|_| CoreError::NotFound)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RedirectUriUpdated,
                    realm_id.into(),
                    Some(redirect_uri.clone()),
                ),
            )
            .await?;

        Ok(redirect_uri)
    }
}

impl<R, U, C, UR, W, RU, PLRU, WO, SA, RO, SE, CS, CSM> WebOriginResolver
    for ClientServiceImpl<R, U, C, UR, W, RU, PLRU, WO, SA, RO, SE, CS, CSM>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    WO: WebOriginRepository,
    SA: ClientSamlRepository,
    RO: RoleRepository,
    SE: SecurityEventRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    async fn resolve_realm_origins(
        &self,
        realm_name: String,
    ) -> Result<HashSet<Origin>, CoreError> {
        let sources = self
            .web_origin_repository
            .get_origin_sources_by_realm_name(realm_name)
            .await?;

        Ok(resolve_allowed_origins(&sources))
    }
}

pub async fn expire_previous_client_secrets_task<S>(service: S, period: Duration)
where
    S: ClientService,
{
    let mut ticker = tokio::time::interval(period);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;
        match service.expire_previous_client_secrets().await {
            Ok(0) => {}
            Ok(expired) => tracing::info!(expired, "Expired previous client secrets"),
            Err(error) => {
                tracing::warn!(error = ?error, "Failed to expire previous client secrets")
            }
        }
    }
}

#[cfg(test)]
mod tests {

    mod source_restrictions {
        use super::*;

        fn payload(token_exchange: Option<bool>, client_id: Option<&str>) -> UpdateClientRequest {
            UpdateClientRequest {
                name: None,
                client_id: client_id.map(str::to_string),
                enabled: None,
                direct_access_grants_enabled: None,
                oauth_device_code_grant_enabled: None,
                token_exchange_enabled: token_exchange,
                require_pkce: None,
                access_token_lifetime: None,
                refresh_token_lifetime: None,
                id_token_lifetime: None,
                temporary_token_lifetime: None,
                maintenance_enabled: None,
                maintenance_reason: None,
                maintenance_session_strategy: None,
                backchannel_logout_uri: None,
                backchannel_logout_session_required: None,
                consent_required: None,
            }
        }

        fn client_from(source: ClientRegistrationSource) -> Client {
            let realm = create_test_realm_with_name("acme");
            let mut client = Client::from_realm_and_client_id(realm.id, "app".to_string());
            client.registration_source = source;
            client
        }

        #[test]
        fn an_admin_client_may_enable_token_exchange() {
            let client = client_from(ClientRegistrationSource::Admin);
            assert!(
                ensure_update_allowed_for_source(&client, &payload(Some(true), Some("x"))).is_ok()
            );
        }

        #[test]
        fn a_self_registered_client_cannot_enable_direct_access_or_device_grants() {
            for source in [
                ClientRegistrationSource::Dynamic,
                ClientRegistrationSource::MetadataDocument,
            ] {
                let client = client_from(source);

                let mut direct = payload(None, None);
                direct.direct_access_grants_enabled = Some(true);
                assert!(matches!(
                    ensure_update_allowed_for_source(&client, &direct),
                    Err(CoreError::Forbidden(_))
                ));

                let mut device = payload(None, None);
                device.oauth_device_code_grant_enabled = Some(true);
                assert!(matches!(
                    ensure_update_allowed_for_source(&client, &device),
                    Err(CoreError::Forbidden(_))
                ));

                let mut disabling = payload(None, None);
                disabling.direct_access_grants_enabled = Some(false);
                disabling.oauth_device_code_grant_enabled = Some(false);
                assert!(ensure_update_allowed_for_source(&client, &disabling).is_ok());
            }
        }

        #[test]
        fn an_admin_client_may_enable_direct_access_and_device_grants() {
            let client = client_from(ClientRegistrationSource::Admin);
            let mut update = payload(None, None);
            update.direct_access_grants_enabled = Some(true);
            update.oauth_device_code_grant_enabled = Some(true);
            assert!(ensure_update_allowed_for_source(&client, &update).is_ok());
        }

        #[test]
        fn a_self_registered_client_cannot_enable_token_exchange_or_change_its_id() {
            for source in [
                ClientRegistrationSource::Dynamic,
                ClientRegistrationSource::MetadataDocument,
            ] {
                let client = client_from(source);
                assert!(matches!(
                    ensure_update_allowed_for_source(&client, &payload(Some(true), None)),
                    Err(CoreError::Forbidden(_))
                ));
                assert!(matches!(
                    ensure_update_allowed_for_source(&client, &payload(None, Some("x"))),
                    Err(CoreError::Forbidden(_))
                ));
                assert!(
                    ensure_update_allowed_for_source(&client, &payload(Some(false), None)).is_ok()
                );
            }
        }
    }
    use super::*;
    use crate::domain::aegis::mocks::{
        MockClientScopeMappingRepository, MockClientScopeRepository,
    };
    use crate::domain::{
        client::{
            entities::ClientConfig,
            ports::{
                MockClientRepository, MockClientSamlRepository,
                MockPostLogoutRedirectUriRepository, MockRedirectUriRepository,
                MockWebOriginRepository,
            },
            value_objects::UpdateClientRequest,
        },
        common::services::tests::{
            create_test_realm_with_name, create_test_user_identity_with_realm,
        },
        realm::{
            entities::{Realm, Unscoped},
            ports::MockRealmRepository,
        },
        role::{entities::permission::Permissions, ports::MockRoleRepository},
        seawatch::ports::MockSecurityEventRepository,
        user::ports::{MockUserRepository, MockUserRoleRepository},
        webhook::ports::MockWebhookRepository,
    };
    use ferriskey_domain::authentication::entities::AuthProtocol;
    use ferriskey_domain::client::entities::ClientType;
    use uuid::Uuid;

    type TestClientService = ClientServiceImpl<
        MockRealmRepository,
        MockUserRepository,
        MockClientRepository,
        MockUserRoleRepository,
        MockWebhookRepository,
        MockRedirectUriRepository,
        MockPostLogoutRedirectUriRepository,
        MockWebOriginRepository,
        MockClientSamlRepository,
        MockRoleRepository,
        MockSecurityEventRepository,
        MockClientScopeRepository,
        MockClientScopeMappingRepository,
    >;

    struct Mocks {
        realm: MockRealmRepository,
        client: MockClientRepository,
        user: MockUserRepository,
        user_role: MockUserRoleRepository,
        webhook: MockWebhookRepository,
        security_event: MockSecurityEventRepository,
        client_scope: MockClientScopeRepository,
    }

    impl Mocks {
        fn for_admin_of(realm: &Realm, identity: &Identity) -> Self {
            let mut realm_repo = MockRealmRepository::new();
            let found = realm.clone();
            realm_repo.expect_get_by_name().returning(move |_| {
                let found = found.clone();
                Box::pin(async move { Ok(Some(found)) })
            });

            let admin_role = Role {
                id: Uuid::new_v4(),
                name: "admin".to_string(),
                description: None,
                permissions: vec![Permissions::ManageClients.name()],
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
                    let roles = vec![admin_role.clone()];
                    Box::pin(async move { Ok(roles) })
                });

            let mut webhook = MockWebhookRepository::new();
            webhook
                .expect_notify::<Client>()
                .returning(|_, _| Box::pin(async { Ok(()) }));

            Self {
                realm: realm_repo,
                client: MockClientRepository::new(),
                user: MockUserRepository::new(),
                user_role,
                webhook,
                security_event: MockSecurityEventRepository::new(),
                client_scope: MockClientScopeRepository::new(),
            }
        }

        fn build(self) -> TestClientService {
            let user = Arc::new(self.user);
            let client = Arc::new(self.client);
            let user_role = Arc::new(self.user_role);
            let policy = Arc::new(FerriskeyPolicy::new(
                user.clone(),
                client.clone(),
                user_role,
            ));

            ClientServiceImpl::new(
                Arc::new(self.realm),
                user,
                client,
                Arc::new(self.webhook),
                Arc::new(MockRedirectUriRepository::new()),
                Arc::new(MockPostLogoutRedirectUriRepository::new()),
                Arc::new(MockWebOriginRepository::new()),
                Arc::new(MockClientSamlRepository::new()),
                Arc::new(MockRoleRepository::new()),
                Arc::new(self.security_event),
                Arc::new(self.client_scope),
                Arc::new(MockClientScopeMappingRepository::new()),
                policy,
            )
        }
    }

    fn stored_client(realm: &Realm, token_exchange_enabled: bool) -> Client {
        Client::new(ClientConfig {
            realm_id: realm.id,
            name: "api".to_string(),
            client_id: "api".to_string(),
            secret: None,
            enabled: true,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: false,
            service_account_enabled: false,
            client_type: ClientType::Confidential,
            direct_access_grants_enabled: None,
            oauth_device_code_grant_enabled: None,
            token_exchange_enabled: Some(token_exchange_enabled),
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
        })
    }

    #[tokio::test]
    async fn a_new_client_does_not_allow_token_exchange_unless_asked() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity);

        let created_in = realm.clone();
        mocks
            .client
            .expect_create_client()
            .withf(|req| !req.token_exchange_enabled)
            .times(1)
            .returning(move |req| {
                let client = stored_client(&created_in, req.token_exchange_enabled);
                Box::pin(async move { Ok(client) })
            });
        mocks
            .client_scope
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(vec![]) }));
        mocks
            .security_event
            .expect_store_event()
            .returning(|_| Box::pin(async { Ok(()) }));

        let client = mocks
            .build()
            .create_client(
                identity,
                CreateClientInput {
                    realm_name: "acme".to_string(),
                    name: "api".to_string(),
                    client_id: "api".to_string(),
                    client_type: ClientType::Confidential,
                    service_account_enabled: false,
                    public_client: false,
                    protocol: AuthProtocol::OpenIdConnect,
                    enabled: true,
                    direct_access_grants_enabled: false,
                    oauth_device_code_grant_enabled: false,
                    token_exchange_enabled: false,
                    consent_required: false,
                    require_pkce: true,
                },
            )
            .await
            .expect("client is created");

        assert!(!client.token_exchange_enabled);
    }

    #[tokio::test]
    async fn an_admin_can_turn_token_exchange_on_for_a_client() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity);

        let existing = stored_client(&realm, false);
        let existing_id = existing.id;
        mocks.client.expect_get_by_id().returning(move |_, _| {
            let existing = existing.clone();
            Box::pin(async move { Ok(Unscoped::new(existing)) })
        });
        let updated_in = realm.clone();
        mocks
            .client
            .expect_update_client()
            .withf(|_, data| data.token_exchange_enabled == Some(true))
            .times(1)
            .returning(move |_, data| {
                let client =
                    stored_client(&updated_in, data.token_exchange_enabled.unwrap_or_default());
                Box::pin(async move { Ok(client) })
            });

        let client = mocks
            .build()
            .update_client(
                identity,
                UpdateClientInput {
                    realm_name: "acme".to_string(),
                    client_id: existing_id,
                    payload: UpdateClientRequest {
                        name: None,
                        client_id: None,
                        enabled: None,
                        direct_access_grants_enabled: None,
                        oauth_device_code_grant_enabled: None,
                        token_exchange_enabled: Some(true),
                        require_pkce: None,
                        access_token_lifetime: None,
                        refresh_token_lifetime: None,
                        id_token_lifetime: None,
                        temporary_token_lifetime: None,
                        maintenance_enabled: None,
                        maintenance_reason: None,
                        maintenance_session_strategy: None,
                        backchannel_logout_uri: None,
                        backchannel_logout_session_required: None,
                        consent_required: None,
                    },
                },
            )
            .await
            .expect("client is updated");

        assert!(client.token_exchange_enabled);
    }

    fn list_request() -> PageRequest<ClientFilter, ClientSortField> {
        PageRequest {
            filter: ClientFilter {
                name: Some("api".to_string()),
                enabled: Some(true),
                protocol: Some(AuthProtocol::Saml),
                ..ClientFilter::default()
            },
            ..PageRequest::default()
        }
    }

    #[tokio::test]
    async fn list_clients_refuses_a_caller_without_view_rights_before_listing() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity);
        let mut user_role = MockUserRoleRepository::new();
        user_role
            .expect_get_user_roles()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        mocks.user_role = user_role;
        mocks.client.expect_list().never();
        let service = mocks.build();

        let result = service
            .list_clients(identity, realm.name.clone(), list_request())
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "got {result:?}"
        );
    }

    #[tokio::test]
    async fn list_clients_pages_the_resolved_realm_with_the_request() {
        let realm = create_test_realm_with_name("acme");
        let identity = create_test_user_identity_with_realm(&realm);
        let mut mocks = Mocks::for_admin_of(&realm, &identity);
        let listed = stored_client(&realm, false);
        let expected_id = listed.id;
        let realm_id = realm.id;
        mocks
            .client
            .expect_list()
            .withf(move |scope, request| scope.id() == realm_id && *request == list_request())
            .times(1)
            .return_once(move |_, request| {
                let page = Page::new(vec![listed], 1, request.page, request.limit);
                Box::pin(async move { Ok(page) })
            });
        let service = mocks.build();

        let page = service
            .list_clients(identity, realm.name.clone(), list_request())
            .await
            .expect("listing succeeds");

        assert_eq!(page.metadata().total, 1);
        assert_eq!(page.data()[0].id, expected_id);
    }

    mod reveal_client_secret {
        use super::*;
        use crate::domain::common::services::tests::create_test_user_with_realm;
        use maskass::Masked;

        const SECRET: &str = "s3cret";

        fn client_with_secret(realm: &Realm) -> Client {
            let mut client = stored_client(realm, false);
            client.secret = Some(Masked::new(SECRET.to_string()));
            client
        }

        fn role_with(realm: &Realm, permission: Permissions) -> Role {
            Role {
                id: Uuid::new_v4(),
                name: "held".to_string(),
                description: None,
                permissions: vec![permission.name()],
                realm_id: realm.id,
                client_id: None,
                client: None,
                require_mfa: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        }

        fn input(client: &Client) -> GetClientInput {
            GetClientInput {
                realm_name: "acme".to_string(),
                client_id: client.id,
            }
        }

        fn expect_client(mocks: &mut Mocks, client: &Client) {
            let client = client.clone();
            mocks.client.expect_get_by_id().returning(move |_, _| {
                let client = client.clone();
                Box::pin(async move { Ok(Unscoped::new(client)) })
            });
        }

        pub(super) fn expect_service_account_holding(
            mocks: &mut Mocks,
            realm: &Realm,
            permission: Permissions,
        ) {
            let service_account = create_test_user_with_realm(realm);
            let account_id = service_account.id;
            mocks.user.expect_get_by_client_id().returning(move |_| {
                let account = service_account.clone();
                Box::pin(async move { Ok(Unscoped::new(account)) })
            });
            let role = role_with(realm, permission);
            mocks
                .user_role
                .expect_get_user_roles()
                .withf(move |id| *id == account_id)
                .returning(move |_| {
                    let roles = vec![role.clone()];
                    Box::pin(async move { Ok(roles) })
                });
        }

        #[tokio::test]
        async fn refuses_a_service_account_that_holds_more_than_the_caller() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = client_with_secret(&realm);
            expect_client(&mut mocks, &client);
            expect_service_account_holding(&mut mocks, &realm, Permissions::ManageRealm);
            mocks.security_event.expect_store_event().never();

            let result = mocks
                .build()
                .reveal_client_secret(identity, input(&client))
                .await;

            assert!(
                matches!(result, Err(CoreError::Forbidden(_))),
                "got {:?}",
                result.err()
            );
        }

        #[tokio::test]
        async fn reveals_the_secret_of_a_service_account_the_caller_covers() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = client_with_secret(&realm);
            expect_client(&mut mocks, &client);
            expect_service_account_holding(&mut mocks, &realm, Permissions::ManageClients);
            mocks
                .security_event
                .expect_store_event()
                .times(1)
                .returning(|_| Box::pin(async { Ok(()) }));

            let secret = mocks
                .build()
                .reveal_client_secret(identity, input(&client))
                .await
                .expect("the caller holds what the service account holds");

            assert_eq!(secret.client_secret.as_deref(), Some(SECRET));
        }

        #[tokio::test]
        async fn fails_closed_when_the_service_account_lookup_fails() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = client_with_secret(&realm);
            expect_client(&mut mocks, &client);
            mocks
                .user
                .expect_get_by_client_id()
                .returning(|_| Box::pin(async { Err(CoreError::InternalServerError) }));
            mocks.security_event.expect_store_event().never();

            let result = mocks
                .build()
                .reveal_client_secret(identity, input(&client))
                .await;

            assert!(
                matches!(result, Err(CoreError::InternalServerError)),
                "got {:?}",
                result.err()
            );
        }

        #[tokio::test]
        async fn reveals_the_secret_of_a_client_without_a_service_account() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = client_with_secret(&realm);
            expect_client(&mut mocks, &client);
            mocks
                .user
                .expect_get_by_client_id()
                .returning(|_| Box::pin(async { Err(CoreError::NotFound) }));
            mocks
                .security_event
                .expect_store_event()
                .times(1)
                .returning(|_| Box::pin(async { Ok(()) }));

            let secret = mocks
                .build()
                .reveal_client_secret(identity, input(&client))
                .await
                .expect("there is no account to hand over");

            assert_eq!(secret.client_secret.as_deref(), Some(SECRET));
        }
    }

    mod secret_rotation {
        use super::*;
        use crate::domain::client::entities::ExpiredClientSecret;
        use crate::domain::client::entities::{PreviousClientSecret, RotateClientSecretInput};
        use crate::domain::seawatch::{ActorType, SecurityEvent};
        use chrono::Duration;
        use mockall::predicate::always;

        fn confidential(realm: &Realm) -> Client {
            let mut client = stored_client(realm, false);
            client.secret = Some(maskass::Masked::new("first-secret".to_string()));
            client
        }

        fn serving(mocks: &mut Mocks, client: Client) {
            mocks.client.expect_get_by_id().returning(move |_, _| {
                let client = client.clone();
                Box::pin(async move { Ok(Unscoped::new(client)) })
            });
        }

        fn without_service_account(mocks: &mut Mocks) {
            mocks
                .user
                .expect_get_by_client_id()
                .returning(|_| Box::pin(async { Err(CoreError::NotFound) }));
        }

        fn recording(mocks: &mut Mocks, expected: SecurityEventType) {
            mocks
                .security_event
                .expect_store_event()
                .withf(move |event: &SecurityEvent| event.event_type == expected)
                .times(1)
                .returning(|_| Box::pin(async { Ok(()) }));
        }

        fn rotation(client_id: Uuid, grace_period_secs: Option<i64>) -> RotateClientSecretInput {
            RotateClientSecretInput {
                client_id,
                realm_name: "acme".to_string(),
                grace_period_secs,
            }
        }

        #[tokio::test]
        async fn rotating_hands_back_a_new_secret_and_when_the_old_one_stops() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = confidential(&realm);
            let client_id = client.id;
            serving(&mut mocks, client.clone());
            without_service_account(&mut mocks);
            let rotated = client.clone();
            mocks
                .client
                .expect_rotate_secret()
                .withf(|_, new_secret, expires_at| {
                    !new_secret.is_empty()
                        && new_secret != "first-secret"
                        && expires_at.is_some_and(|at| {
                            let wait = at - Utc::now();
                            wait > Duration::seconds(3_500) && wait <= Duration::seconds(3_600)
                        })
                })
                .times(1)
                .returning(move |_, _, _| {
                    let rotated = rotated.clone();
                    Box::pin(async move { Ok(rotated) })
                });
            recording(&mut mocks, SecurityEventType::ClientSecretRotated);

            let result = mocks
                .build()
                .rotate_client_secret(identity, rotation(client_id, Some(3_600)))
                .await
                .expect("the secret is rotated");

            assert!(!result.client_secret.is_empty());
            assert_ne!(result.client_secret, "first-secret");
            assert!(result.previous_secret_expires_at.is_some());
        }

        #[tokio::test]
        async fn rotating_refuses_a_service_account_that_holds_more_than_the_caller() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = confidential(&realm);
            let client_id = client.id;
            serving(&mut mocks, client);
            super::reveal_client_secret::expect_service_account_holding(
                &mut mocks,
                &realm,
                Permissions::ManageRealm,
            );
            mocks.client.expect_rotate_secret().never();
            mocks.security_event.expect_store_event().never();

            let result = mocks
                .build()
                .rotate_client_secret(identity, rotation(client_id, Some(3_600)))
                .await;

            assert!(
                matches!(result, Err(CoreError::Forbidden(_))),
                "got {:?}",
                result.err()
            );
        }

        #[tokio::test]
        async fn rotating_without_a_grace_period_keeps_no_previous_secret() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = confidential(&realm);
            let client_id = client.id;
            serving(&mut mocks, client.clone());
            without_service_account(&mut mocks);
            mocks
                .client
                .expect_rotate_secret()
                .withf(|_, _, expires_at| expires_at.is_none())
                .times(1)
                .returning(move |_, _, _| {
                    let client = client.clone();
                    Box::pin(async move { Ok(client) })
                });
            recording(&mut mocks, SecurityEventType::ClientSecretRotated);

            let result = mocks
                .build()
                .rotate_client_secret(identity, rotation(client_id, None))
                .await
                .expect("the secret is rotated");

            assert!(result.previous_secret_expires_at.is_none());
        }

        #[tokio::test]
        async fn a_grace_period_out_of_range_is_refused_before_anything_changes() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mocks = Mocks::for_admin_of(&realm, &identity);

            let result = mocks
                .build()
                .rotate_client_secret(identity, rotation(Uuid::new_v4(), Some(-5)))
                .await;

            assert!(matches!(
                result,
                Err(CoreError::InvalidSecretGracePeriod(_))
            ));
        }

        #[tokio::test]
        async fn a_public_client_has_no_secret_to_rotate() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let mut public = stored_client(&realm, false);
            public.public_client = true;
            let client_id = public.id;
            serving(&mut mocks, public);

            let result = mocks
                .build()
                .rotate_client_secret(identity, rotation(client_id, Some(60)))
                .await;

            assert!(matches!(result, Err(CoreError::ClientHasNoSecret)));
        }

        #[tokio::test]
        async fn rotating_needs_the_right_to_update_clients() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            mocks.user_role = MockUserRoleRepository::new();
            mocks
                .user_role
                .expect_get_user_roles()
                .returning(|_| Box::pin(async { Ok(vec![]) }));

            let result = mocks
                .build()
                .rotate_client_secret(identity, rotation(Uuid::new_v4(), Some(60)))
                .await;

            assert!(matches!(result, Err(CoreError::Forbidden(_))));
        }

        #[tokio::test]
        async fn revoking_records_the_event_only_when_a_previous_secret_existed() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = confidential(&realm);
            let client_id = client.id;
            serving(&mut mocks, client);
            mocks
                .client
                .expect_revoke_previous_secret()
                .times(1)
                .returning(|_| Box::pin(async { Ok(true) }));
            recording(&mut mocks, SecurityEventType::ClientSecretRevoked);

            mocks
                .build()
                .revoke_previous_client_secret(
                    identity,
                    GetClientInput {
                        client_id,
                        realm_name: "acme".to_string(),
                    },
                )
                .await
                .expect("the previous secret is revoked");
        }

        #[tokio::test]
        async fn revoking_when_there_is_no_previous_secret_is_not_found_and_silent() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client = confidential(&realm);
            let client_id = client.id;
            serving(&mut mocks, client);
            mocks
                .client
                .expect_revoke_previous_secret()
                .returning(|_| Box::pin(async { Ok(false) }));
            mocks.security_event.expect_store_event().never();

            let result = mocks
                .build()
                .revoke_previous_client_secret(
                    identity,
                    GetClientInput {
                        client_id,
                        realm_name: "acme".to_string(),
                    },
                )
                .await;

            assert!(matches!(result, Err(CoreError::NotFound)));
        }

        #[tokio::test]
        async fn revealing_shows_the_previous_secret_only_while_it_still_works() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);

            for (offset, shown) in [(Duration::minutes(5), true), (-Duration::minutes(5), false)] {
                let mut mocks = Mocks::for_admin_of(&realm, &identity);
                let mut client = confidential(&realm);
                client.previous_secret = Some(PreviousClientSecret {
                    secret: maskass::Masked::new("old-secret".to_string()),
                    expires_at: Utc::now() + offset,
                });
                let client_id = client.id;
                serving(&mut mocks, client);
                mocks
                    .user
                    .expect_get_by_client_id()
                    .returning(|_| Box::pin(async { Err(CoreError::NotFound) }));
                recording(&mut mocks, SecurityEventType::ClientSecretViewed);

                let revealed = mocks
                    .build()
                    .reveal_client_secret(
                        identity.clone(),
                        GetClientInput {
                            client_id,
                            realm_name: "acme".to_string(),
                        },
                    )
                    .await
                    .expect("the secrets are revealed");

                assert_eq!(revealed.client_secret.as_deref(), Some("first-secret"));
                assert_eq!(
                    revealed
                        .previous
                        .as_ref()
                        .map(|previous| previous.client_secret.as_str()),
                    shown.then_some("old-secret")
                );
            }
        }

        #[tokio::test]
        async fn each_expired_previous_secret_is_recorded_as_a_system_event() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let client_id = Uuid::new_v4();
            let realm_id = realm.id;
            let expired_at = Utc::now() - Duration::minutes(1);
            mocks
                .client
                .expect_expire_previous_secrets()
                .with(always())
                .times(1)
                .returning(move |_| {
                    Box::pin(async move {
                        Ok(vec![ExpiredClientSecret {
                            client_id,
                            realm_id,
                            expired_at,
                        }])
                    })
                });
            mocks
                .security_event
                .expect_store_event()
                .withf(move |event: &SecurityEvent| {
                    event.event_type == SecurityEventType::ClientSecretExpired
                        && event.realm_id == realm_id
                        && event.target_id == Some(client_id)
                        && event.actor_type == Some(ActorType::System)
                })
                .times(1)
                .returning(|_| Box::pin(async { Ok(()) }));

            let expired = mocks
                .build()
                .expire_previous_client_secrets()
                .await
                .expect("the sweep succeeds");

            assert_eq!(expired, 1);
        }

        #[tokio::test]
        async fn a_failing_event_store_does_not_stop_the_sweep() {
            let realm = create_test_realm_with_name("acme");
            let identity = create_test_user_identity_with_realm(&realm);
            let mut mocks = Mocks::for_admin_of(&realm, &identity);
            let realm_id = realm.id;
            mocks
                .client
                .expect_expire_previous_secrets()
                .returning(move |_| {
                    Box::pin(async move {
                        Ok(["a", "b"]
                            .map(|_| ExpiredClientSecret {
                                client_id: Uuid::new_v4(),
                                realm_id,
                                expired_at: Utc::now(),
                            })
                            .into())
                    })
                });
            mocks
                .security_event
                .expect_store_event()
                .times(2)
                .returning(|_| Box::pin(async { Err(CoreError::InternalServerError) }));

            let expired = mocks
                .build()
                .expire_previous_client_secrets()
                .await
                .expect("the sweep still reports what it dropped");

            assert_eq!(expired, 2);
        }
    }
}
