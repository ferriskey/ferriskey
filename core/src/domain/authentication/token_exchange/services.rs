//! The RFC 8693 token exchange: a confidential client swaps an access token
//! it is a party to for a new one with a narrower scope and/or a specific
//! audience, within the limits of its delegation policy.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use ferriskey_compass::entities::{FlowStatus, FlowStepName};
use ferriskey_compass::recorder::FlowRecorder;
use ferriskey_compass::value_objects::StepOutcome;
use ferriskey_domain::generate_uuid_v7;
use ferriskey_domain::token_lifetime::TokenLifetimes;
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

use crate::domain::authentication::OidcScope;
use crate::domain::authentication::entities::GrantType;
use crate::domain::authentication::services::client_secret_matches;
use crate::domain::authentication::token_exchange::entities::{TokenExchangeError, TokenType};
use crate::domain::authentication::token_exchange::ports::{
    SubjectTokenIssuer, TokenExchangeService,
};
use crate::domain::authentication::token_exchange::value_objects::{
    TokenExchangeOutput, TokenExchangeParams,
};
use crate::domain::client::entities::Client;
use crate::domain::client::entities::token_exchange_policy::TokenExchangePolicy;
use crate::domain::client::ports::{ClientRepository, TokenExchangePolicyRepository};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::jwt::entities::{ClaimsTyp, JwtClaim};
use crate::domain::realm::entities::{RealmScope, UnscopedOption};
use crate::domain::seawatch::{
    EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType,
};
use crate::domain::user::entities::User;
use crate::domain::user::ports::UserRepository;

#[derive(Clone, Debug)]
pub struct TokenExchangeServiceImpl<C, U, P, SER, I>
where
    C: ClientRepository,
    U: UserRepository,
    P: TokenExchangePolicyRepository,
    SER: SecurityEventRepository,
    I: SubjectTokenIssuer,
{
    pub(crate) client_repository: Arc<C>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) policy_repository: Arc<P>,
    pub(crate) security_event_repository: Arc<SER>,
    pub(crate) token_issuer: Arc<I>,
    pub(crate) flow_recorder: FlowRecorder,
}

/// What an authenticated exchange learned before it succeeded or failed, for
/// the audit trail. Filled in as each check passes.
struct ExchangeTrace {
    client: Uuid,
    subject: Option<Uuid>,
    actor: Option<Uuid>,
    policy: Option<Uuid>,
}

impl<C, U, P, SER, I> TokenExchangeServiceImpl<C, U, P, SER, I>
where
    C: ClientRepository,
    U: UserRepository,
    P: TokenExchangePolicyRepository,
    SER: SecurityEventRepository,
    I: SubjectTokenIssuer,
{
    pub fn new(
        client_repository: Arc<C>,
        user_repository: Arc<U>,
        policy_repository: Arc<P>,
        security_event_repository: Arc<SER>,
        token_issuer: Arc<I>,
        flow_recorder: FlowRecorder,
    ) -> Self {
        Self {
            client_repository,
            user_repository,
            policy_repository,
            security_event_repository,
            token_issuer,
            flow_recorder,
        }
    }

    /// The requesting client, authenticated. Public clients cannot keep a
    /// secret, so they never get to exchange a token.
    async fn authenticate_client(
        &self,
        params: &TokenExchangeParams,
    ) -> Result<Client, TokenExchangeError> {
        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm.id())
            .await
            .map_err(|_| TokenExchangeError::InvalidClient)?
            .in_realm(&params.realm)
            .map_err(|_| TokenExchangeError::InvalidClient)?
            .into_inner();

        if client.public_client
            || !client_secret_matches(client.secret_str(), params.client_secret.as_deref())
        {
            return Err(TokenExchangeError::InvalidClient);
        }

        if !client.enabled || !client.token_exchange_enabled {
            return Err(TokenExchangeError::UnauthorizedClient);
        }

        Ok(client)
    }

    /// The subject token's claims and user, once the token is proven to be
    /// a live access token of this realm that `client` is a party to.
    async fn verify_subject(
        &self,
        realm: &RealmScope,
        client: &Client,
        subject_token: String,
    ) -> Result<(JwtClaim, User), TokenExchangeError> {
        let claims = self
            .token_issuer
            .verify_subject_token(subject_token, realm.id())
            .await
            .map_err(|err| match err {
                CoreError::InternalServerError => {
                    TokenExchangeError::ServerError("could not verify the subject token".into())
                }
                _ => TokenExchangeError::InvalidRequest,
            })?;

        if claims.typ != ClaimsTyp::Bearer {
            return Err(TokenExchangeError::InvalidRequest);
        }

        // Without this, any client with the grant enabled could exchange a
        // token it intercepted.
        let is_party = claims.azp == client.client_id || claims.aud.contains(&client.client_id);
        if !is_party {
            return Err(TokenExchangeError::UnauthorizedClient);
        }

        let user = self
            .user_repository
            .get_by_id(claims.sub)
            .await
            .map_err(|err| match err {
                CoreError::NotFound => TokenExchangeError::InvalidRequest,
                _ => TokenExchangeError::ServerError("could not load the subject".into()),
            })?
            .in_realm(realm)
            .map_err(|_| TokenExchangeError::InvalidRequest)?
            .into_inner();

        if !user.enabled {
            return Err(TokenExchangeError::InvalidRequest);
        }

        Ok((claims, user))
    }

    /// The actor token's claims, once it is proven to be a live access token
    /// of this realm issued to `client`: a client can only present itself, or
    /// a user it holds a token for, as the actor.
    async fn verify_actor(
        &self,
        realm: &RealmScope,
        client: &Client,
        actor_token: String,
    ) -> Result<JwtClaim, TokenExchangeError> {
        let claims = self
            .token_issuer
            .verify_subject_token(actor_token, realm.id())
            .await
            .map_err(|err| match err {
                CoreError::InternalServerError => {
                    TokenExchangeError::ServerError("could not verify the actor token".into())
                }
                _ => TokenExchangeError::InvalidRequest,
            })?;

        if claims.typ != ClaimsTyp::Bearer || claims.azp != client.client_id {
            return Err(TokenExchangeError::InvalidRequest);
        }

        Ok(claims)
    }

    /// The delegation policy letting `client` target `audience`, with the
    /// target client itself. A delegated exchange (with an actor) needs the
    /// policy to allow delegation; a plain one acts as the subject and needs
    /// it to allow impersonation.
    async fn policy_for(
        &self,
        realm: &RealmScope,
        client: &Client,
        audience: &str,
        delegated: bool,
    ) -> Result<(TokenExchangePolicy, Client), TokenExchangeError> {
        let target = self
            .client_repository
            .get_by_client_id(audience.to_string(), realm.id())
            .await
            .map_err(|_| TokenExchangeError::InvalidTarget)?
            .in_realm(realm)
            .map_err(|_| TokenExchangeError::InvalidTarget)?
            .into_inner();

        if !target.enabled {
            return Err(TokenExchangeError::InvalidTarget);
        }

        let policy = self
            .policy_repository
            .find_for_target(client.id, audience.to_string())
            .await
            .map_err(|_| TokenExchangeError::ServerError("could not load the policy".into()))?
            .in_realm(realm)
            .map_err(|_| TokenExchangeError::InvalidTarget)?
            .ok_or(TokenExchangeError::InvalidTarget)?
            .into_inner();

        let allowed = if delegated {
            policy.allow_delegation
        } else {
            policy.allow_impersonation
        };
        if !allowed {
            return Err(TokenExchangeError::UnauthorizedClient);
        }

        Ok((policy, target))
    }

    async fn exchange_as(
        &self,
        client: &Client,
        params: TokenExchangeParams,
        trace: &mut ExchangeTrace,
    ) -> Result<TokenExchangeOutput, TokenExchangeError> {
        let input = params.input;
        TokenType::from_urn(&input.subject_token_type)?.ensure_supported()?;
        if let Some(requested) = input.requested_token_type.as_deref() {
            TokenType::from_urn(requested)?.ensure_supported()?;
        }

        // RFC 8693 §2.1: `actor_token_type` is required with `actor_token`
        // and must not appear without it.
        let actor_token = match (input.actor_token, input.actor_token_type.as_deref()) {
            (Some(token), Some(urn)) => {
                TokenType::from_urn(urn)?.ensure_supported()?;
                Some(token)
            }
            (None, None) => None,
            _ => return Err(TokenExchangeError::InvalidRequest),
        };

        if input.resource.is_some() {
            return Err(TokenExchangeError::InvalidTarget);
        }

        let (subject, user) = self
            .verify_subject(&params.realm, client, input.subject_token)
            .await?;
        trace.subject = Some(subject.sub);

        let actor = match actor_token {
            Some(token) => Some(self.verify_actor(&params.realm, client, token).await?),
            None => None,
        };
        trace.actor = actor.as_ref().map(|actor| actor.sub);

        let audience = input
            .audience
            .filter(|audience| !audience.trim().is_empty());
        let (policy, target) = match audience.as_deref() {
            Some(audience) => {
                let (policy, target) = self
                    .policy_for(&params.realm, client, audience, actor.is_some())
                    .await?;
                (Some(policy), Some(target))
            }
            // Delegation is only ever granted by a policy, and without an
            // audience there is none to grant it.
            None if actor.is_some() => return Err(TokenExchangeError::UnauthorizedClient),
            None => (None, None),
        };
        trace.policy = policy.as_ref().map(|policy| policy.id);

        let scope = resolve_scope(
            input.scope.as_deref(),
            subject.scope.as_deref(),
            policy
                .as_ref()
                .and_then(|policy| policy.allowed_scopes.as_deref()),
        )?;

        let settings = params
            .realm
            .realm()
            .settings
            .as_ref()
            .ok_or_else(|| TokenExchangeError::ServerError("realm has no settings".into()))?;
        let lifetime = TokenLifetimes::resolve(settings, client).access_token;

        // The token is shaped by the client that will consume it: the target
        // when there is one, otherwise the requester narrowing its own token.
        let mapper_client = target.unwrap_or_else(|| client.clone());
        let mapped = self
            .token_issuer
            .mapped_claims(params.realm.clone(), user, mapper_client, scope.clone())
            .await
            .map_err(|err| TokenExchangeError::ServerError(err.to_string()))?;

        let now = Utc::now().timestamp();
        let claims = exchanged_claims(
            &subject,
            ExchangedToken {
                azp: client.client_id.clone(),
                audience,
                scope: scope.clone(),
                mapped,
                act: act_claim(&subject, actor.as_ref()),
            },
            now,
            lifetime,
        );
        let exp = claims.exp.unwrap_or(now);

        let jwt = self
            .token_issuer
            .issue_access_token(params.realm.id(), claims)
            .await
            .map_err(|err| TokenExchangeError::ServerError(err.to_string()))?;

        Ok(TokenExchangeOutput {
            access_token: jwt.token,
            issued_token_type: TokenType::AccessToken,
            token_type: "Bearer".to_string(),
            expires_in: (exp - now).max(0),
            scope: (scope != subject.scope).then_some(scope).flatten(),
        })
    }

    async fn audit(
        &self,
        params: &TokenExchangeParams,
        trace: &ExchangeTrace,
        result: &Result<TokenExchangeOutput, TokenExchangeError>,
    ) {
        let realm_id = params.realm.id();
        let status = match result {
            Ok(_) => EventStatus::Success,
            Err(_) => EventStatus::Failure,
        };

        let event = match trace.subject {
            Some(subject) => {
                SecurityEvent::new(realm_id, SecurityEventType::TokenExchanged, status, subject)
            }
            None => {
                SecurityEvent::without_actor(realm_id, SecurityEventType::TokenExchanged, status)
            }
        }
        .with_target(
            "client".to_string(),
            trace.client,
            Some(params.client_id.clone()),
        );

        let mut details = json!({
            "client_id": params.client_id,
            "audience": params.input.audience,
            "policy_id": trace.policy,
            "actor": trace.actor,
        });
        if let Err(err) = result {
            details["error_code"] = json!(err.to_string());
        }

        if let Err(err) = self
            .security_event_repository
            .store_event(event.with_details(details))
            .await
        {
            warn!("Failed to store TokenExchanged security event: {err}");
        }
    }
}

impl<C, U, P, SER, I> TokenExchangeService for TokenExchangeServiceImpl<C, U, P, SER, I>
where
    C: ClientRepository,
    U: UserRepository,
    P: TokenExchangePolicyRepository,
    SER: SecurityEventRepository,
    I: SubjectTokenIssuer,
{
    async fn exchange(
        &self,
        params: TokenExchangeParams,
    ) -> Result<TokenExchangeOutput, TokenExchangeError> {
        let started = Utc::now();

        // Nothing is traced before the client proves who it is: an unknown
        // client_id must not be able to fill the Compass and audit tables.
        let client = self.authenticate_client(&params).await?;

        let flow_id = self
            .flow_recorder
            .start_flow(
                params.realm.realm(),
                Some(params.client_id.clone()),
                GrantType::TokenExchange.to_string(),
                params.input.ip_address.clone(),
                params.input.user_agent.clone(),
            )
            .await;

        let mut trace = ExchangeTrace {
            client: client.id,
            subject: None,
            actor: None,
            policy: None,
        };
        let result = self.exchange_as(&client, params.clone(), &mut trace).await;

        let duration = (Utc::now() - started).num_milliseconds();
        let (outcome, flow_status) = match &result {
            Ok(_) => (StepOutcome::success(), FlowStatus::Success),
            Err(err) => (StepOutcome::failure(err.to_string()), FlowStatus::Failure),
        };
        self.flow_recorder.record_step(
            flow_id.clone(),
            FlowStepName::SubjectTokenExchange,
            outcome.with_duration(duration),
        );
        self.flow_recorder
            .complete_flow(flow_id, flow_status, duration, trace.subject);

        self.audit(&params, &trace, &result).await;

        if let Err(TokenExchangeError::ServerError(detail)) = &result {
            warn!(detail = %detail, "Token exchange failed on the server side");
        }

        result
    }
}

fn split_scope(scope: Option<&str>) -> Vec<&str> {
    let mut scopes: Vec<&str> = Vec::new();
    for scope in scope.unwrap_or_default().split_whitespace() {
        if !scopes.contains(&scope) {
            scopes.push(scope);
        }
    }
    scopes
}

/// The scope of the issued token.
///
/// A requested scope must fit within the subject token's scope and within the
/// policy ceiling, when there is one. With nothing requested the token keeps
/// what both allow. `offline_access` never survives: an exchange issues no
/// refresh token.
fn resolve_scope(
    requested: Option<&str>,
    subject: Option<&str>,
    ceiling: Option<&[String]>,
) -> Result<Option<String>, TokenExchangeError> {
    let offline = OidcScope::OfflineAccess.as_str();
    let subject = split_scope(subject);
    let allowed = |scope: &str| {
        subject.contains(&scope)
            && ceiling.is_none_or(|ceiling| ceiling.iter().any(|allowed| allowed == scope))
    };

    let requested = split_scope(requested);
    let granted: Vec<&str> = if requested.is_empty() {
        subject
            .iter()
            .copied()
            .filter(|scope| *scope != offline && allowed(scope))
            .collect()
    } else {
        if requested
            .iter()
            .any(|scope| *scope == offline || !allowed(scope))
        {
            return Err(TokenExchangeError::InvalidScope);
        }
        requested
    };

    Ok((!granted.is_empty()).then(|| granted.join(" ")))
}

/// What the exchange decided for the issued token.
struct ExchangedToken {
    /// The requesting client, which holds the new token.
    azp: String,
    audience: Option<String>,
    scope: Option<String>,
    /// Claims from the consuming client's protocol mappers, for `scope` only.
    mapped: HashMap<String, serde_json::Value>,
    /// The RFC 8693 §4.1 `act` claim, when anyone acts for the subject.
    act: Option<serde_json::Value>,
}

/// The `act` claim of the issued token (RFC 8693 §4.1). A new actor wraps
/// any delegation already recorded on the subject token, so the chain reads
/// from the current actor down to the first one. Without a new actor the
/// existing chain is carried over untouched: an exchange never erases who
/// acted before.
fn act_claim(subject: &JwtClaim, actor: Option<&JwtClaim>) -> Option<serde_json::Value> {
    let previous = subject.additional_claims.get("act").cloned();
    match actor {
        Some(actor) => {
            let mut act = json!({ "sub": actor.sub, "client_id": actor.azp });
            if let Some(previous) = previous {
                act["act"] = previous;
            }
            Some(act)
        }
        None => previous,
    }
}

/// The claims of the issued token. `iss`, `sub` and `sid` come from the
/// subject token and cannot change; the mapped claims are recomputed for the
/// narrowed scope, so nothing the subject token carried outside it survives.
/// The new token never outlives the subject token.
fn exchanged_claims(
    subject: &JwtClaim,
    token: ExchangedToken,
    now: i64,
    lifetime: i64,
) -> JwtClaim {
    let mut claims = subject.clone();
    claims.jti = generate_uuid_v7();
    claims.iat = now;
    claims.typ = ClaimsTyp::Bearer;
    claims.azp = token.azp;
    claims.scope = token.scope;
    claims.additional_claims = token.mapped;
    if let Some(act) = token.act {
        claims.additional_claims.insert("act".to_string(), act);
    }
    // Identity claims now reach tokens only through the mapped claims.
    claims.preferred_username = None;
    claims.email = None;
    let audience = token.audience;
    claims.exp = Some(match subject.exp {
        Some(subject_exp) => subject_exp.min(now + lifetime),
        None => now + lifetime,
    });
    if let Some(audience) = audience {
        claims.aud = vec![audience];
    }
    claims
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use crate::domain::authentication::token_exchange::ports::MockSubjectTokenIssuer;
    use crate::domain::authentication::token_exchange::value_objects::TokenExchangeInput;
    use crate::domain::client::entities::token_exchange_policy::TokenExchangePolicyDefinition;
    use crate::domain::client::ports::{MockClientRepository, MockTokenExchangePolicyRepository};
    use crate::domain::common::services::tests::{
        create_test_realm_with_name, create_test_user_with_realm,
    };
    use crate::domain::jwt::entities::Jwt;
    use crate::domain::realm::entities::{Realm, RealmSetting, Unscoped};
    use crate::domain::seawatch::ports::MockSecurityEventRepository;
    use crate::domain::user::entities::User;
    use crate::domain::user::ports::MockUserRepository;

    const ACCESS_TOKEN_URN: &str = "urn:ietf:params:oauth:token-type:access_token";
    const SECRET: &str = "s3cret";
    const REQUESTER: &str = "gateway";
    const AUDIENCE: &str = "orders-api";
    const ACTOR_TOKEN: &str = "actor-token";

    type TestService = TokenExchangeServiceImpl<
        MockClientRepository,
        MockUserRepository,
        MockTokenExchangePolicyRepository,
        MockSecurityEventRepository,
        MockSubjectTokenIssuer,
    >;

    fn realm() -> Realm {
        let mut realm = create_test_realm_with_name("acme");
        let mut settings = RealmSetting::new(realm.id, None);
        settings.access_token_lifetime = 300;
        realm.settings = Some(settings);
        realm
    }

    fn client(realm: &Realm, client_id: &str) -> Client {
        let mut client = Client::from_realm_and_client_id(realm.id, client_id.to_string());
        client.secret = Some(maskass::Masked::new(SECRET.to_string()));
        client.token_exchange_enabled = true;
        client
    }

    fn subject_claims(user: &User, azp: &str, scope: &str, exp_in: i64) -> JwtClaim {
        let now = Utc::now().timestamp();
        JwtClaim {
            sub: user.id,
            iat: now,
            jti: Uuid::new_v4(),
            iss: "https://auth.example/realms/acme".to_string(),
            typ: ClaimsTyp::Bearer,
            azp: azp.to_string(),
            aud: vec!["acme-realm".to_string(), "account".to_string()],
            scope: Some(scope.to_string()),
            exp: Some(now + exp_in),
            preferred_username: Some(user.username.clone()),
            email: user.email.clone(),
            client_id: None,
            sid: Some(Uuid::new_v4()),
            additional_claims: HashMap::from([("roles".to_string(), json!(["reader"]))]),
        }
    }

    fn policy(
        realm: &Realm,
        client: &Client,
        allowed_scopes: Option<&[&str]>,
    ) -> TokenExchangePolicy {
        TokenExchangePolicy::new(
            realm.id,
            client.id,
            TokenExchangePolicyDefinition {
                target_audience: AUDIENCE.to_string(),
                allowed_scopes: allowed_scopes
                    .map(|scopes| scopes.iter().map(|scope| scope.to_string()).collect()),
                allow_impersonation: true,
                allow_delegation: false,
            },
        )
    }

    fn input(scope: Option<&str>, audience: Option<&str>) -> TokenExchangeInput {
        TokenExchangeInput {
            subject_token: "subject-token".to_string(),
            subject_token_type: ACCESS_TOKEN_URN.to_string(),
            requested_token_type: None,
            audience: audience.map(str::to_string),
            resource: None,
            scope: scope.map(str::to_string),
            actor_token: None,
            actor_token_type: None,
            ip_address: None,
            user_agent: None,
        }
    }

    fn params(realm: &Realm, input: TokenExchangeInput) -> TokenExchangeParams {
        TokenExchangeParams {
            realm: RealmScope::from_realm(realm.clone()),
            client_id: REQUESTER.to_string(),
            client_secret: Some(SECRET.to_string()),
            input,
        }
    }

    /// Mocks for one exchange. Anything left unset refuses to be called, so a
    /// test also proves which lookups a refusal skips.
    struct Harness {
        clients: Vec<Client>,
        user: Option<User>,
        subject: Result<JwtClaim, CoreError>,
        actor: Option<JwtClaim>,
        policy: Option<TokenExchangePolicy>,
        issued: Arc<Mutex<Option<JwtClaim>>>,
        events: Arc<Mutex<Vec<SecurityEvent>>>,
    }

    impl Harness {
        fn new(realm: &Realm, requester: Client, subject: JwtClaim) -> Self {
            let mut user = create_test_user_with_realm(realm);
            user.id = subject.sub;
            Self {
                clients: vec![requester],
                user: Some(user),
                subject: Ok(subject),
                actor: None,
                policy: None,
                issued: Arc::default(),
                events: Arc::default(),
            }
        }

        fn with_actor(mut self, actor: JwtClaim) -> Self {
            self.actor = Some(actor);
            self
        }

        fn with_client(mut self, client: Client) -> Self {
            self.clients.push(client);
            self
        }

        fn with_policy(mut self, policy: TokenExchangePolicy) -> Self {
            self.policy = Some(policy);
            self
        }

        fn build(&self) -> TestService {
            let mut clients = MockClientRepository::new();
            let known = self.clients.clone();
            clients
                .expect_get_by_client_id()
                .returning(move |client_id, _| {
                    let found = known.iter().find(|c| c.client_id == client_id).cloned();
                    Box::pin(async move { found.map(Unscoped::new).ok_or(CoreError::NotFound) })
                });

            let mut users = MockUserRepository::new();
            let user = self.user.clone();
            users.expect_get_by_id().returning(move |_| {
                let user = user.clone();
                Box::pin(async move { user.map(Unscoped::new).ok_or(CoreError::NotFound) })
            });

            let mut policies = MockTokenExchangePolicyRepository::new();
            let policy = self.policy.clone();
            policies
                .expect_find_for_target()
                .returning(move |client_id, audience| {
                    let found = policy
                        .clone()
                        .filter(|p| p.client_id == client_id && p.target_audience == audience)
                        .map(Unscoped::new);
                    Box::pin(async move { Ok(found) })
                });

            let mut events = MockSecurityEventRepository::new();
            let stored = self.events.clone();
            events.expect_store_event().returning(move |event| {
                stored.lock().expect("events lock").push(event);
                Box::pin(async { Ok(()) })
            });

            let mut issuer = MockSubjectTokenIssuer::new();
            let subject = self.subject.clone();
            let actor = self.actor.clone();
            issuer
                .expect_verify_subject_token()
                .returning(move |token, _| {
                    let verified = match (token.as_str(), actor.clone()) {
                        (ACTOR_TOKEN, Some(actor)) => Ok(actor),
                        (ACTOR_TOKEN, None) => Err(CoreError::InvalidToken),
                        _ => subject.clone(),
                    };
                    Box::pin(async move { verified })
                });
            // Stands in for the protocol mappers: says which client shaped the
            // token and for which scope.
            issuer
                .expect_mapped_claims()
                .returning(|_, user, client, scope| {
                    let mapped = HashMap::from([
                        ("mapped_for".to_string(), json!(client.client_id)),
                        ("mapped_scope".to_string(), json!(scope)),
                        ("mapped_user".to_string(), json!(user.id)),
                    ]);
                    Box::pin(async move { Ok(mapped) })
                });
            let issued = self.issued.clone();
            issuer
                .expect_issue_access_token()
                .returning(move |_, claims| {
                    let expires_at = claims.exp.unwrap_or_default();
                    *issued.lock().expect("issued lock") = Some(claims);
                    Box::pin(async move {
                        Ok(Jwt {
                            token: "exchanged-token".to_string(),
                            expires_at,
                        })
                    })
                });

            TokenExchangeServiceImpl::new(
                Arc::new(clients),
                Arc::new(users),
                Arc::new(policies),
                Arc::new(events),
                Arc::new(issuer),
                FlowRecorder::disabled(),
            )
        }

        fn issued(&self) -> JwtClaim {
            self.issued
                .lock()
                .expect("issued lock")
                .clone()
                .expect("a token should have been issued")
        }

        fn nothing_issued(&self) -> bool {
            self.issued.lock().expect("issued lock").is_none()
        }

        fn events(&self) -> Vec<SecurityEvent> {
            self.events.lock().expect("events lock").clone()
        }
    }

    fn standard() -> (Realm, Client, User) {
        let realm = realm();
        let requester = client(&realm, REQUESTER);
        let user = create_test_user_with_realm(&realm);
        (realm, requester, user)
    }

    #[tokio::test]
    async fn downscoping_without_an_audience_keeps_the_subject_audience() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid profile email", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject.clone());

        let output = harness
            .build()
            .exchange(params(&realm, input(Some("profile"), None)))
            .await
            .expect("exchange should succeed");

        assert_eq!(output.access_token, "exchanged-token");
        assert_eq!(output.issued_token_type, TokenType::AccessToken);
        assert_eq!(output.token_type, "Bearer");
        assert_eq!(output.scope.as_deref(), Some("profile"));

        let issued = harness.issued();
        assert_eq!(issued.aud, subject.aud);
        assert_eq!(issued.azp, REQUESTER);
        assert_eq!(issued.scope.as_deref(), Some("profile"));
        assert_eq!(issued.sid, subject.sid);
        assert_eq!(issued.sub, subject.sub);
        assert_ne!(issued.jti, subject.jti);
        assert_eq!(issued.iss, subject.iss);
    }

    #[tokio::test]
    async fn downscoping_remaps_the_claims_for_the_requester_and_the_narrowed_scope() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid profile email", 3600);
        let harness = Harness::new(&realm, requester, subject);

        harness
            .build()
            .exchange(params(&realm, input(Some("profile"), None)))
            .await
            .expect("exchange should succeed");

        let issued = harness.issued();
        assert_eq!(issued.additional_claims["mapped_for"], json!(REQUESTER));
        assert_eq!(issued.additional_claims["mapped_scope"], json!("profile"));
        assert_eq!(issued.additional_claims["mapped_user"], json!(user.id));
        assert!(
            !issued.additional_claims.contains_key("roles"),
            "claims of the subject token must not be copied over"
        );
        assert_eq!(issued.email, None);
        assert_eq!(issued.preferred_username, None);
    }

    #[tokio::test]
    async fn an_audience_is_mapped_by_the_target_client() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid orders:read", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(policy(&realm, &requester, None));

        harness
            .build()
            .exchange(params(&realm, input(Some("orders:read"), Some(AUDIENCE))))
            .await
            .expect("exchange should succeed");

        let issued = harness.issued();
        assert_eq!(issued.additional_claims["mapped_for"], json!(AUDIENCE));
        assert_eq!(
            issued.additional_claims["mapped_scope"],
            json!("orders:read")
        );
        assert_eq!(issued.azp, REQUESTER, "the requester still holds the token");
    }

    #[tokio::test]
    async fn an_allowed_audience_replaces_the_audience() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid orders:read orders:write", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(policy(&realm, &requester, None));

        let output = harness
            .build()
            .exchange(params(&realm, input(Some("orders:read"), Some(AUDIENCE))))
            .await
            .expect("exchange should succeed");

        assert_eq!(output.scope.as_deref(), Some("orders:read"));
        assert_eq!(harness.issued().aud, vec![AUDIENCE.to_string()]);
    }

    #[tokio::test]
    async fn the_issued_token_never_outlives_the_subject_token() {
        let (realm, requester, user) = standard();
        // The client lifetime (300s) is longer than what the subject has left.
        let subject = subject_claims(&user, REQUESTER, "openid", 60);
        let harness = Harness::new(&realm, requester, subject.clone());

        let output = harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect("exchange should succeed");

        assert_eq!(harness.issued().exp, subject.exp);
        assert!(output.expires_in <= 60);
    }

    #[tokio::test]
    async fn the_issued_token_uses_the_client_lifetime_when_shorter() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester, subject.clone());

        let output = harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect("exchange should succeed");

        let issued = harness.issued();
        assert!(issued.exp < subject.exp);
        assert_eq!(issued.exp, Some(issued.iat + 300));
        assert!(output.expires_in <= 300);
    }

    #[tokio::test]
    async fn no_requested_scope_keeps_the_subject_scope_and_omits_it() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid profile", 3600);
        let harness = Harness::new(&realm, requester, subject);

        let output = harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect("exchange should succeed");

        assert_eq!(harness.issued().scope.as_deref(), Some("openid profile"));
        assert_eq!(output.scope, None);
    }

    #[tokio::test]
    async fn a_wider_scope_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid profile", 3600);
        let harness = Harness::new(&realm, requester, subject);

        let err = harness
            .build()
            .exchange(params(&realm, input(Some("profile email"), None)))
            .await
            .expect_err("a wider scope must be refused");

        assert_eq!(err, TokenExchangeError::InvalidScope);
        assert!(harness.nothing_issued());
    }

    #[tokio::test]
    async fn offline_access_is_refused_even_when_the_subject_has_it() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid offline_access", 3600);
        let harness = Harness::new(&realm, requester, subject);

        let err = harness
            .build()
            .exchange(params(&realm, input(Some("offline_access"), None)))
            .await
            .expect_err("offline_access must be refused");
        assert_eq!(err, TokenExchangeError::InvalidScope);

        harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect("an inherited offline_access is dropped, not refused");
        assert_eq!(harness.issued().scope.as_deref(), Some("openid"));
    }

    #[tokio::test]
    async fn an_unknown_audience_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester, subject);

        let err = harness
            .build()
            .exchange(params(&realm, input(None, Some("nobody"))))
            .await
            .expect_err("an unknown audience must be refused");

        assert_eq!(err, TokenExchangeError::InvalidTarget);
    }

    #[tokio::test]
    async fn an_audience_without_a_policy_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness =
            Harness::new(&realm, requester, subject).with_client(client(&realm, AUDIENCE));

        let err = harness
            .build()
            .exchange(params(&realm, input(None, Some(AUDIENCE))))
            .await
            .expect_err("an audience with no policy must be refused");

        assert_eq!(err, TokenExchangeError::InvalidTarget);
        assert!(harness.nothing_issued());
    }

    #[tokio::test]
    async fn a_policy_from_another_realm_does_not_count() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let mut foreign = policy(&realm, &requester, None);
        foreign.realm_id = realm_with_other_id().id;
        let harness = Harness::new(&realm, requester, subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(foreign);

        let err = harness
            .build()
            .exchange(params(&realm, input(None, Some(AUDIENCE))))
            .await
            .expect_err("a policy of another realm must not apply");

        assert_eq!(err, TokenExchangeError::InvalidTarget);
    }

    fn realm_with_other_id() -> Realm {
        create_test_realm_with_name("other")
    }

    #[tokio::test]
    async fn a_scope_within_the_policy_ceiling_is_granted() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid orders:read orders:write", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(policy(&realm, &requester, Some(&["orders:read"])));

        harness
            .build()
            .exchange(params(&realm, input(Some("orders:read"), Some(AUDIENCE))))
            .await
            .expect("a scope within the ceiling should be granted");

        assert_eq!(harness.issued().scope.as_deref(), Some("orders:read"));
    }

    #[tokio::test]
    async fn a_scope_above_the_policy_ceiling_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid orders:read orders:write", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(policy(&realm, &requester, Some(&["orders:read"])));

        let err = harness
            .build()
            .exchange(params(&realm, input(Some("orders:write"), Some(AUDIENCE))))
            .await
            .expect_err("a scope above the ceiling must be refused");

        assert_eq!(err, TokenExchangeError::InvalidScope);
    }

    #[tokio::test]
    async fn no_requested_scope_takes_the_intersection_with_the_ceiling() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid orders:read orders:write", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(policy(
                &realm,
                &requester,
                Some(&["orders:read", "billing:read"]),
            ));

        let output = harness
            .build()
            .exchange(params(&realm, input(None, Some(AUDIENCE))))
            .await
            .expect("exchange should succeed");

        assert_eq!(harness.issued().scope.as_deref(), Some("orders:read"));
        assert_eq!(output.scope.as_deref(), Some("orders:read"));
    }

    #[tokio::test]
    async fn a_policy_without_impersonation_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let mut no_impersonation = policy(&realm, &requester, None);
        no_impersonation.allow_impersonation = false;
        let harness = Harness::new(&realm, requester, subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(no_impersonation);

        let err = harness
            .build()
            .exchange(params(&realm, input(None, Some(AUDIENCE))))
            .await
            .expect_err("impersonation must be allowed by the policy");

        assert_eq!(err, TokenExchangeError::UnauthorizedClient);
    }

    #[tokio::test]
    async fn a_public_client_is_refused() {
        let (realm, mut requester, user) = standard();
        requester.public_client = true;
        requester.secret = None;
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester, subject);
        let mut request = params(&realm, input(None, None));
        request.client_secret = None;

        let err = harness
            .build()
            .exchange(request)
            .await
            .expect_err("a public client must be refused");

        assert_eq!(err, TokenExchangeError::InvalidClient);
    }

    #[tokio::test]
    async fn a_wrong_secret_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester, subject);
        let mut request = params(&realm, input(None, None));
        request.client_secret = Some("wrong".to_string());

        let err = harness
            .build()
            .exchange(request)
            .await
            .expect_err("a wrong secret must be refused");

        assert_eq!(err, TokenExchangeError::InvalidClient);
        assert!(
            harness.events().is_empty(),
            "an unauthenticated caller must not be able to write to the audit trail"
        );
    }

    #[tokio::test]
    async fn a_client_without_the_flag_is_refused() {
        let (realm, mut requester, user) = standard();
        requester.token_exchange_enabled = false;
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester, subject);

        let err = harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect_err("the grant must be enabled on the client");

        assert_eq!(err, TokenExchangeError::UnauthorizedClient);
    }

    #[tokio::test]
    async fn an_invalid_subject_token_is_refused() {
        for failure in [
            CoreError::ExpiredToken,
            CoreError::InvalidToken,
            CoreError::TokenValidationError("bad signature".to_string()),
        ] {
            let (realm, requester, user) = standard();
            let mut harness = Harness::new(
                &realm,
                requester,
                subject_claims(&user, REQUESTER, "openid", 3600),
            );
            harness.subject = Err(failure);

            let err = harness
                .build()
                .exchange(params(&realm, input(None, None)))
                .await
                .expect_err("an invalid subject token must be refused");

            assert_eq!(err, TokenExchangeError::InvalidRequest);
        }
    }

    #[tokio::test]
    async fn a_refresh_token_is_not_a_valid_subject() {
        let (realm, requester, user) = standard();
        let mut subject = subject_claims(&user, REQUESTER, "openid", 3600);
        subject.typ = ClaimsTyp::Refresh;
        let harness = Harness::new(&realm, requester, subject);

        let err = harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect_err("a refresh token must be refused");

        assert_eq!(err, TokenExchangeError::InvalidRequest);
    }

    #[tokio::test]
    async fn a_client_that_is_not_a_party_of_the_subject_token_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, "someone-else", "openid", 3600);
        let harness = Harness::new(&realm, requester, subject);

        let err = harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect_err("a client outside azp and aud must be refused");

        assert_eq!(err, TokenExchangeError::UnauthorizedClient);
    }

    #[tokio::test]
    async fn a_client_listed_in_the_subject_audience_may_exchange() {
        let (realm, requester, user) = standard();
        let mut subject = subject_claims(&user, "frontend", "openid", 3600);
        subject.aud.push(REQUESTER.to_string());
        let harness = Harness::new(&realm, requester, subject);

        harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect("a client in aud is a party of the token");
    }

    #[tokio::test]
    async fn a_disabled_subject_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let mut harness = Harness::new(&realm, requester, subject);
        if let Some(user) = harness.user.as_mut() {
            user.enabled = false;
        }

        let err = harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect_err("a disabled user must be refused");

        assert_eq!(err, TokenExchangeError::InvalidRequest);
    }

    #[tokio::test]
    async fn a_resource_is_not_supported_yet() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester, subject);
        let mut request = input(None, None);
        request.resource = Some("https://orders.example".to_string());

        let err = harness
            .build()
            .exchange(params(&realm, request))
            .await
            .expect_err("resource is not supported");

        assert_eq!(err, TokenExchangeError::InvalidTarget);
    }

    #[tokio::test]
    async fn only_access_tokens_can_be_exchanged_or_requested() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester, subject);

        let mut id_token_subject = input(None, None);
        id_token_subject.subject_token_type =
            "urn:ietf:params:oauth:token-type:id_token".to_string();
        let mut jwt_requested = input(None, None);
        jwt_requested.requested_token_type =
            Some("urn:ietf:params:oauth:token-type:jwt".to_string());

        for request in [id_token_subject, jwt_requested] {
            let err = harness
                .build()
                .exchange(params(&realm, request))
                .await
                .expect_err("only access tokens are supported");
            assert_eq!(err, TokenExchangeError::UnsupportedTokenType);
        }
    }

    #[tokio::test]
    async fn every_exchange_leaves_a_security_event() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let policy = policy(&realm, &requester, None);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(policy.clone());

        let service = harness.build();
        service
            .exchange(params(&realm, input(None, Some(AUDIENCE))))
            .await
            .expect("exchange should succeed");
        service
            .exchange(params(&realm, input(Some("admin"), None)))
            .await
            .expect_err("a wider scope must be refused");

        let events = harness.events();
        assert_eq!(events.len(), 2);

        let success = &events[0];
        assert_eq!(success.event_type, SecurityEventType::TokenExchanged);
        assert_eq!(success.status, EventStatus::Success);
        assert_eq!(success.actor_id, Some(user.id));
        assert_eq!(success.target_id, Some(requester.id));
        let details = success.details.clone().expect("details");
        assert_eq!(details["audience"], AUDIENCE);
        assert_eq!(details["policy_id"], json!(policy.id));

        let failure = &events[1];
        assert_eq!(failure.status, EventStatus::Failure);
        assert_eq!(
            failure.details.clone().expect("details")["error_code"],
            "invalid_scope"
        );
    }

    #[test]
    fn resolve_scope_follows_the_subject_and_the_ceiling() {
        let ceiling = ["a".to_string(), "b".to_string()];

        assert_eq!(resolve_scope(None, None, None), Ok(None));
        assert_eq!(
            resolve_scope(Some("b a b"), Some("a b c"), None),
            Ok(Some("b a".to_string()))
        );
        assert_eq!(
            resolve_scope(None, Some("a b c"), Some(&ceiling)),
            Ok(Some("a b".to_string()))
        );
        assert_eq!(
            resolve_scope(Some("c"), Some("a b c"), Some(&ceiling)),
            Err(TokenExchangeError::InvalidScope)
        );
        assert_eq!(
            resolve_scope(Some("  "), Some("a"), None),
            Ok(Some("a".to_string()))
        );
    }

    fn delegated(audience: Option<&str>) -> TokenExchangeInput {
        let mut request = input(None, audience);
        request.actor_token = Some(ACTOR_TOKEN.to_string());
        request.actor_token_type = Some(ACCESS_TOKEN_URN.to_string());
        request
    }

    fn actor_claims(realm: &Realm, azp: &str) -> JwtClaim {
        let actor = create_test_user_with_realm(realm);
        let mut claims = subject_claims(&actor, azp, "openid", 3600);
        claims.sub = Uuid::new_v4();
        claims
    }

    fn delegating_policy(realm: &Realm, client: &Client) -> TokenExchangePolicy {
        let mut policy = policy(realm, client, None);
        policy.allow_impersonation = false;
        policy.allow_delegation = true;
        policy
    }

    #[tokio::test]
    async fn a_delegated_exchange_names_the_actor() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let actor = actor_claims(&realm, REQUESTER);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(delegating_policy(&realm, &requester))
            .with_actor(actor.clone());

        harness
            .build()
            .exchange(params(&realm, delegated(Some(AUDIENCE))))
            .await
            .expect("delegation is allowed by the policy");

        let issued = harness.issued();
        assert_eq!(issued.sub, user.id, "the subject stays the subject");
        assert_eq!(
            issued.additional_claims["act"],
            json!({ "sub": actor.sub, "client_id": REQUESTER })
        );
    }

    #[tokio::test]
    async fn delegation_needs_the_policy_to_allow_it() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        // Impersonation on, delegation off.
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(policy(&realm, &requester, None))
            .with_actor(actor_claims(&realm, REQUESTER));

        let err = harness
            .build()
            .exchange(params(&realm, delegated(Some(AUDIENCE))))
            .await
            .expect_err("delegation must be allowed by the policy");

        assert_eq!(err, TokenExchangeError::UnauthorizedClient);
        assert!(harness.nothing_issued());
    }

    #[tokio::test]
    async fn a_delegating_policy_does_not_allow_impersonation() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(delegating_policy(&realm, &requester));

        let err = harness
            .build()
            .exchange(params(&realm, input(None, Some(AUDIENCE))))
            .await
            .expect_err("a plain exchange needs impersonation");

        assert_eq!(err, TokenExchangeError::UnauthorizedClient);
    }

    #[tokio::test]
    async fn an_actor_token_issued_to_another_client_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(delegating_policy(&realm, &requester))
            .with_actor(actor_claims(&realm, "someone-else"));

        let err = harness
            .build()
            .exchange(params(&realm, delegated(Some(AUDIENCE))))
            .await
            .expect_err("an actor token of another client must be refused");

        assert_eq!(err, TokenExchangeError::InvalidRequest);
    }

    #[tokio::test]
    async fn an_invalid_actor_token_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        // No actor registered: the actor token fails verification.
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(delegating_policy(&realm, &requester));

        let err = harness
            .build()
            .exchange(params(&realm, delegated(Some(AUDIENCE))))
            .await
            .expect_err("an invalid actor token must be refused");

        assert_eq!(err, TokenExchangeError::InvalidRequest);
    }

    #[tokio::test]
    async fn an_actor_without_an_audience_is_refused() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness =
            Harness::new(&realm, requester, subject).with_actor(actor_claims(&realm, REQUESTER));

        let err = harness
            .build()
            .exchange(params(&realm, delegated(None)))
            .await
            .expect_err("delegation is only granted by a policy");

        assert_eq!(err, TokenExchangeError::UnauthorizedClient);
    }

    #[tokio::test]
    async fn the_actor_token_and_its_type_come_together() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let harness =
            Harness::new(&realm, requester, subject).with_actor(actor_claims(&realm, REQUESTER));

        let mut token_only = delegated(Some(AUDIENCE));
        token_only.actor_token_type = None;
        let mut type_only = input(None, Some(AUDIENCE));
        type_only.actor_token_type = Some(ACCESS_TOKEN_URN.to_string());
        for request in [token_only, type_only] {
            let err = harness
                .build()
                .exchange(params(&realm, request))
                .await
                .expect_err("actor_token and actor_token_type go together");
            assert_eq!(err, TokenExchangeError::InvalidRequest);
        }

        let mut id_token_actor = delegated(Some(AUDIENCE));
        id_token_actor.actor_token_type =
            Some("urn:ietf:params:oauth:token-type:id_token".to_string());
        let err = harness
            .build()
            .exchange(params(&realm, id_token_actor))
            .await
            .expect_err("only access tokens can act");
        assert_eq!(err, TokenExchangeError::UnsupportedTokenType);
    }

    #[tokio::test]
    async fn a_chained_delegation_nests_the_previous_actor() {
        let (realm, requester, user) = standard();
        let mut subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let first_actor = json!({ "sub": Uuid::new_v4(), "client_id": "frontend" });
        subject
            .additional_claims
            .insert("act".to_string(), first_actor.clone());
        let actor = actor_claims(&realm, REQUESTER);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(delegating_policy(&realm, &requester))
            .with_actor(actor.clone());

        harness
            .build()
            .exchange(params(&realm, delegated(Some(AUDIENCE))))
            .await
            .expect("exchange should succeed");

        assert_eq!(
            harness.issued().additional_claims["act"],
            json!({ "sub": actor.sub, "client_id": REQUESTER, "act": first_actor })
        );
    }

    #[tokio::test]
    async fn a_plain_exchange_keeps_the_recorded_delegation() {
        let (realm, requester, user) = standard();
        let mut subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let previous = json!({ "sub": Uuid::new_v4(), "client_id": "frontend" });
        subject
            .additional_claims
            .insert("act".to_string(), previous.clone());
        let harness = Harness::new(&realm, requester, subject);

        harness
            .build()
            .exchange(params(&realm, input(None, None)))
            .await
            .expect("exchange should succeed");

        assert_eq!(harness.issued().additional_claims["act"], previous);
    }

    #[tokio::test]
    async fn the_audit_trail_names_the_actor() {
        let (realm, requester, user) = standard();
        let subject = subject_claims(&user, REQUESTER, "openid", 3600);
        let actor = actor_claims(&realm, REQUESTER);
        let harness = Harness::new(&realm, requester.clone(), subject)
            .with_client(client(&realm, AUDIENCE))
            .with_policy(delegating_policy(&realm, &requester))
            .with_actor(actor.clone());

        harness
            .build()
            .exchange(params(&realm, delegated(Some(AUDIENCE))))
            .await
            .expect("exchange should succeed");

        let events = harness.events();
        let details = events[0].details.clone().expect("details");
        assert_eq!(details["actor"], json!(actor.sub));
    }
}
