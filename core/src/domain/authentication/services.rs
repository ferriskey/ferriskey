use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use base64::prelude::{BASE64_URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, TimeZone, Utc};
use ferriskey_security::jwt::ports::KeyStoreRepository;
use jsonwebtoken::{Header, Validation};
use serde::Serialize;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tracing::{Instrument, error, info, info_span, instrument, warn};
use uuid::Uuid;

use ferriskey_aegis::entities::{ClientScope, ProtocolMapper};
use ferriskey_aegis::ports::{ClientScopeMappingRepository, ProtocolMapperRepository};
use ferriskey_compass::entities::{FlowId, FlowStatus, FlowStepName, StepStatus};
use ferriskey_compass::recorder::FlowRecorder;
use ferriskey_organization::{
    Group, GroupId, GroupTokenRepository, OrganizationAttributeRepository, OrganizationId,
    OrganizationMemberRepository, OrganizationRepository,
};

use crate::domain::authentication::mapper_engine::{ContextGroup, ContextOrganization};
use crate::domain::maintenance::ports::{
    MaintenanceWhitelistRepository, RealmMaintenanceWhitelistRepository,
};
use crate::domain::trident::mfa_policy;
use crate::domain::{
    abyss::federation::ports::FederationRepository,
    authentication::{
        OidcScope,
        entities::{
            AuthCompletion, AuthInput, AuthOutput, AuthProtocol, AuthSession, AuthSessionParams,
            AuthenticateOutput, AuthenticationMethod, AuthorizeRequestInput,
            AuthorizeRequestOutput, CredentialsAuthParams, ExchangeTokenInput, GrantType, JwtToken,
            TokenIntrospectionResponse,
        },
        mapper_engine::{MapperContext, MapperEngine, TokenType},
        ports::{AuthService, AuthSessionRepository, LoginActionToken, LoginActionTokenRepository},
        value_objects::{
            AuthenticationResult, CodeChallengeMethod, EndSessionInput, EndSessionOutput,
            EvaluateClientScopesInput, EvaluateClientScopesResult, EvaluatedMapper, EvaluatedRoles,
            EvaluatedScope, GenerateTokenInput, GenerateTokensForUserInput, GetUserInfoInput,
            GrantTypeParams, Identity, IntrospectTokenInput, RegisterUserInput, RegisterUserOutput,
            RegisterUserUrlContext, RevokeTokenInput, UserInfoResponse,
        },
    },
    client::{
        entities::Client,
        ports::{ClientRepository, PostLogoutRedirectUriRepository, RedirectUriRepository},
        redirect_uri_matching::redirect_uri_matches_any,
    },
    common::{entities::app_errors::CoreError, generate_random_string},
    credential::{
        entities::{CredentialData, CredentialType},
        ports::CredentialRepository,
    },
    crypto::HasherRepository,
    email_verification::ports::EmailVerificationService,
    jwt::{
        JwtError,
        entities::{ClaimsTyp, IdTokenClaims, JwkKey, Jwt, JwtClaim, JwtKeyPair},
        ports::{AccessTokenRepository, RefreshTokenRepository, RotateOutcome},
    },
    realm::{
        entities::{RealmId, RealmSetting},
        ports::RealmRepository,
    },
    role::entities::Role,
    seawatch::{EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType},
    session::{entities::UserSession, ports::UserSessionRepository},
    user::{
        entities::{RequiredAction, UserAttribute},
        ports::{
            UserAttributeRepository, UserRepository, UserRequiredActionRepository,
            UserRoleRepository,
        },
        value_objects::CreateUserRequest,
    },
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};
use ferriskey_domain::token_lifetime::TokenLifetimes;
use ferriskey_security::jwt::entities::DEFAULT_TEMPORARY_TOKEN_LIFETIME;

use crate::infrastructure::abyss::federation::ldap::LdapClientImpl;

/// Per-organization role buckets: `org_id -> (realm role names, client roles keyed by client_id)`.
/// Feeds the org-scoped role claim in token assembly.
type OrgScopedRoles = HashMap<Uuid, (Vec<String>, HashMap<String, Vec<String>>)>;

fn lockout_compute_locked_until(
    new_attempts: i32,
    threshold: i32,
    duration_seconds: i32,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    if new_attempts >= threshold {
        let duration = chrono::Duration::seconds(duration_seconds as i64);
        Some(now + duration)
    } else {
        None
    }
}

/// Build the OAuth2 authorization-code redirect URL sent back to the client
/// once authentication (login *or* registration) completes.
///
/// `state` is OPTIONAL per RFC 6749 §4.1.2 and OpenID Connect Core: when the
/// originating authorization request omitted it we must neither invent one nor
/// fail the flow with a 500. We only append `&state=` when the session carries
/// a non-empty value, echoing it back verbatim as required when present.
fn format_authorization_redirect_url(
    auth_session: &AuthSession,
    authorization_code: &str,
) -> String {
    // A registered redirect URI may already carry a query string
    // (`https://app.example/cb?tenant=acme`), so the separator has to be chosen
    // rather than assumed, and `state` echoed back percent-encoded.
    let separator = if auth_session.redirect_uri.contains('?') {
        '&'
    } else {
        '?'
    };

    let url = format!(
        "{}{separator}code={}",
        auth_session.redirect_uri,
        urlencoding::encode(authorization_code)
    );

    match auth_session.state.as_deref() {
        Some(state) if !state.is_empty() => {
            format!("{url}&state={}", urlencoding::encode(state))
        }
        _ => url,
    }
}

pub(crate) fn format_auth_completion(
    auth_session: &AuthSession,
    authorization_code: &str,
) -> Result<AuthCompletion, CoreError> {
    match auth_session.protocol {
        AuthProtocol::OpenIdConnect | AuthProtocol::Saml => Ok(AuthCompletion::Redirect {
            url: format_authorization_redirect_url(auth_session, authorization_code),
        }),
    }
}

/// Decide whether a registration that carried a `FERRISKEY_SESSION` cookie
/// should resume the originating OIDC authorization flow (issue an
/// authorization code and redirect back to the client) instead of falling back
/// to standalone self-service sign-up.
///
/// We only resume a session that is still live (`expires_at` in the future) and
/// has not already been consumed by a prior authentication — a session that
/// already has a `user_id` *and* is flagged `authenticated` is spent, and
/// replaying it would mint a second authorization code for a stale request.
fn auth_session_can_resume(auth_session: &AuthSession, now: DateTime<Utc>) -> bool {
    auth_session.expires_at >= now
        && !(auth_session.user_id.is_some() && auth_session.authenticated)
}

/// Gate the `ExistingToken` branch of `POST /login-actions/authenticate`
/// (FK-003).
///
/// That branch skips the whole interactive login and finalizes the flow on the
/// strength of the presented token alone, so the token must be one that already
/// *stands for* a completed authentication — i.e. a `Bearer` access token.
///
/// Every other `typ` is a mid-flow artifact and replaying it here would let the
/// holder jump the step it was minted for. The `Temporary` token is the sharp
/// case: `using_session_code` hands it out right after the password check and
/// right *before* the OTP challenge, so accepting it here completes login with
/// the password alone. `Refresh` and `Id` tokens are rejected for the same
/// reason — they are not proof of a finished login at this endpoint.
///
/// The `AuthSession` freshness check is repeated here (it also lives in
/// `authenticate`) so this path can never outlive its authorization request,
/// whatever future caller reaches it. `now` is injected so expiry is testable.
fn validate_token_refresh_request(
    claims_typ: &ClaimsTyp,
    auth_session: &AuthSession,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    if auth_session.expires_at < now {
        return Err(CoreError::SessionExpired);
    }

    if *claims_typ != ClaimsTyp::Bearer {
        return Err(CoreError::InvalidToken);
    }

    Ok(())
}

fn validate_session_binding(
    claimed_sid: Option<Uuid>,
    session: Option<&UserSession>,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    let Some(sid) = claimed_sid else {
        return Ok(());
    };

    let Some(session) = session else {
        warn!(session_id = %sid, "Rejecting token: the session it names no longer exists");
        return Err(CoreError::SessionRevoked);
    };

    if session.expires_at < now {
        warn!(session_id = %sid, "Rejecting token: the session it names has expired");
        return Err(CoreError::SessionRevoked);
    }

    Ok(())
}

/// Translate a revoked-session rejection into the token endpoint's vocabulary.
///
/// `verify_token` speaks in authentication terms (`SessionRevoked` -> 401) because
/// most of its callers guard protected resources. The token endpoint answers with
/// the OAuth2 error shape instead, where a grant that can no longer be honoured is
/// `invalid_grant` with HTTP 400.
fn revoked_session_is_an_invalid_grant(error: CoreError) -> CoreError {
    match error {
        CoreError::SessionRevoked => CoreError::InvalidGrant(
            "The session backing this refresh token has been revoked or has expired.".to_string(),
        ),
        other => other,
    }
}

/// Re-derive the required actions that gate the token-refresh path (FK-003).
///
/// `ConfigureOtp` is *computed*, never written to `user_required_actions`: the
/// credentials path builds it on the fly in `using_session_code`. So a path
/// that only reads the persisted `user.required_actions` sees an empty list and
/// finalizes, silently bypassing mandatory MFA enrolment. Replaying the same
/// policy here closes that hole.
///
/// Unlike the credentials path we do not suspend enforcement for a temporary
/// password: no password is presented on this path, so there is nothing to make
/// the exception for, and defaulting to "enforce" is the safe direction.
fn resolve_refresh_required_actions(
    persisted_actions: &[RequiredAction],
    realm_settings: Option<&RealmSetting>,
    user_roles: &[Role],
    has_otp_credential: bool,
) -> Vec<RequiredAction> {
    let mut actions = persisted_actions.to_vec();

    let mfa_required_action = mfa_policy::user_requires_mfa(realm_settings, user_roles)
        .then(|| {
            mfa_policy::required_action_for_mfa(has_otp_credential).filter(|a| !actions.contains(a))
        })
        .flatten();

    if let Some(action) = mfa_required_action {
        actions.push(action);
    }

    actions
}

fn pending_step_message(step: &mfa_policy::PendingAuthStep) -> String {
    match step {
        mfa_policy::PendingAuthStep::RequiredActions(actions) => format!(
            "the required action(s) {} are still owed",
            actions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        mfa_policy::PendingAuthStep::OtpChallenge => {
            "a second authentication factor is still owed".to_string()
        }
    }
}

fn pending_step_refusal(step: &mfa_policy::PendingAuthStep) -> CoreError {
    CoreError::Forbidden(format!(
        "Tokens cannot be issued for this account because {}. Sign in through the browser-based authorization_code flow to complete the remaining step.",
        pending_step_message(step)
    ))
}

fn refuse_token_issuance_when_step_pending(
    step: Option<&mfa_policy::PendingAuthStep>,
) -> Result<(), CoreError> {
    match step {
        Some(step) => Err(pending_step_refusal(step)),
        None => Ok(()),
    }
}

fn refuse_token_issuance_when_actions_pending(
    step: Option<&mfa_policy::PendingAuthStep>,
) -> Result<(), CoreError> {
    match step {
        Some(step @ mfa_policy::PendingAuthStep::RequiredActions(_)) => {
            Err(pending_step_refusal(step))
        }
        _ => Ok(()),
    }
}

/// Lifetime, in seconds, of a `Temporary` step token.
///
/// A step token only has to survive one hop of the login flow (OTP challenge,
/// required-action screen), so it must not borrow the access-token lifetime a
/// realm may have widened to hours. Realms expose a dedicated
/// `temporary_token_lifetime`; when a realm has no settings row we fall back to
/// `DEFAULT_TEMPORARY_TOKEN_LIFETIME` rather than to the access-token default.
pub(crate) const LOGIN_ACTION_SESSION_CLAIM: &str = "afs";

fn temporary_token_lifetime(realm_settings: Option<&RealmSetting>) -> i64 {
    realm_settings
        .map(|s| s.temporary_token_lifetime)
        .unwrap_or(DEFAULT_TEMPORARY_TOKEN_LIFETIME)
}

/// Constant-time comparison of a configured client secret against the one
/// presented on the token endpoint. `(None, None)` means "no secret configured
/// and none presented", which is only ever reached for public clients.
pub(crate) fn normalize_login_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub(crate) fn client_secret_matches(stored: Option<&str>, provided: Option<&str>) -> bool {
    match (stored, provided) {
        (Some(s), Some(p)) => {
            let s_hash = Sha256::digest(s.as_bytes());
            let p_hash = Sha256::digest(p.as_bytes());
            s_hash.ct_eq(&p_hash).into()
        }
        (None, None) => true,
        _ => false,
    }
}

/// Bind an incoming `authorization_code` token request back to the authorization
/// request that minted the code (RFC 6749 §4.1.3, §10.5).
///
/// A bearer authorization code is not a credential on its own: it only proves
/// that *someone* completed a login, not that the caller is the client the code
/// was issued to. Every check below re-establishes one half of that binding, so
/// a code obtained out-of-band (referrer leak, browser history, open redirect)
/// cannot be redeemed by a different client, against a different realm, at a
/// different redirect target, or indefinitely.
///
/// `now` is injected so the expiry branch is testable.
fn validate_authorization_code_request(
    auth_session: &AuthSession,
    client: &Client,
    request_realm_id: RealmId,
    request_redirect_uri: Option<&str>,
    request_client_secret: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    if auth_session.protocol != AuthProtocol::OpenIdConnect {
        warn!(
            auth_session_id = %auth_session.id,
            protocol = ?auth_session.protocol,
            "authorization_code: refusing to redeem a code minted for another protocol"
        );
        return Err(CoreError::InvalidAuthorizationCode);
    }

    if !client.enabled {
        return Err(CoreError::InvalidClient);
    }

    // Confidential clients must authenticate. Skipping this let anyone redeem a
    // code without ever proving they are the client it belongs to.
    if !client.public_client && !client_secret_matches(client.secret_str(), request_client_secret) {
        warn!(
            client_id = %client.client_id,
            "authorization_code: client secret mismatch for confidential client"
        );
        return Err(CoreError::InvalidClientSecret);
    }

    // The code belongs to one client. Without this, any client_id in the realm
    // could redeem another client's code and receive tokens minted under its own
    // `azp`, crossing the client boundary.
    if client.id != auth_session.client_id {
        warn!(
            client_id = %client.client_id,
            expected_client = %auth_session.client_id,
            "authorization_code: code was issued to a different client"
        );
        return Err(CoreError::InvalidAuthorizationCode);
    }

    // Codes are looked up globally by value, so without a realm check a code
    // could be redeemed at another realm's token endpoint and come back signed
    // with that realm's key — a tenant-isolation break.
    if auth_session.realm_id != request_realm_id {
        warn!(
            session_realm = ?auth_session.realm_id,
            request_realm = ?request_realm_id,
            "authorization_code: code was issued for a different realm"
        );
        return Err(CoreError::InvalidAuthorizationCode);
    }

    // RFC 6749 §4.1.3: `redirect_uri` is REQUIRED when the authorization request
    // carried one, and must be identical. Our /auth endpoint always requires it,
    // so an absent value here is a mismatch.
    if request_redirect_uri != Some(auth_session.redirect_uri.as_str()) {
        warn!(
            client_id = %client.client_id,
            "authorization_code: redirect_uri does not match the authorization request"
        );
        return Err(CoreError::InvalidAuthorizationCode);
    }

    if now >= auth_session.expires_at {
        warn!(
            client_id = %client.client_id,
            expires_at = %auth_session.expires_at,
            "authorization_code: code has expired"
        );
        return Err(CoreError::InvalidAuthorizationCode);
    }

    Ok(())
}

/// Verify a PKCE `code_verifier` against the stored `code_challenge`.
///
/// RFC 7636 §4.6: S256 → BASE64URL-ENCODE(SHA256(ASCII(code_verifier)));
///                plain → code_verifier == code_challenge.
fn pkce_verify(code_verifier: &str, code_challenge: &str, method: &CodeChallengeMethod) -> bool {
    match method {
        CodeChallengeMethod::S256 => {
            let digest = Sha256::digest(code_verifier.as_bytes());
            let computed = BASE64_URL_SAFE_NO_PAD.encode(digest);
            computed.as_bytes().ct_eq(code_challenge.as_bytes()).into()
        }
        CodeChallengeMethod::Plain => code_verifier
            .as_bytes()
            .ct_eq(code_challenge.as_bytes())
            .into(),
    }
}

/// Validate `code_verifier` per RFC 7636 §4.1:
/// length 43–128, characters in `[A-Z a-z 0-9 \-._~]`.
fn pkce_validate_verifier(verifier: &str) -> bool {
    let len = verifier.len();
    if !(43..=128).contains(&len) {
        return false;
    }
    verifier
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~'))
}

/// Build `ContextGroup`s (with full `/parent/child` paths) from a flat list of the user's
/// effective groups (which already includes every ancestor, so all parents are present).
/// `direct_ids` marks which of those groups are direct memberships (vs. inherited ancestors).
fn build_context_groups(groups: &[Group], direct_ids: &HashSet<Uuid>) -> Vec<ContextGroup> {
    let by_id: HashMap<GroupId, &Group> = groups.iter().map(|g| (g.id, g)).collect();

    groups
        .iter()
        .map(|group| {
            // Walk up to the root collecting names, then reverse to root-first order.
            let mut names = vec![group.name.clone()];
            let mut cursor = group.parent_group_id;
            while let Some(parent_id) = cursor {
                match by_id.get(&parent_id) {
                    Some(parent) => {
                        names.push(parent.name.clone());
                        cursor = parent.parent_group_id;
                    }
                    None => break,
                }
            }
            names.reverse();
            ContextGroup {
                id: group.id.as_uuid(),
                organization_id: group.organization_id.as_uuid(),
                name: group.name.clone(),
                path: format!("/{}", names.join("/")),
                direct: direct_ids.contains(&group.id.as_uuid()),
            }
        })
        .collect()
}

/// Token claims assembled from a client's scopes + protocol mappers for a given user,
/// independent of signing and persistence. Reused by `create_jwt` (real issuance) and by the
/// client-scope evaluation preview, so the preview reflects exactly what a real token carries.
struct AssembledClaims {
    access_claims: JwtClaim,
    /// ID-token additional claims produced by `IdToken` mappers; `None` when the `openid`
    /// scope is absent (no ID token is issued in that case).
    id_mapper_claims: Option<HashMap<String, serde_json::Value>>,
    /// Client scopes that applied (default scopes + requested optional scopes).
    effective_scopes: Vec<ClientScope>,
    /// Protocol mappers gathered from the effective scopes.
    effective_mappers: Vec<ProtocolMapper>,
}

#[derive(Clone, Debug)]
pub struct AuthServiceImpl<
    R,
    C,
    RU,
    PLRU,
    U,
    UR,
    CR,
    H,
    AS,
    KS,
    RT,
    AT,
    F,
    CSM,
    PM,
    OM,
    OR,
    OAR,
    GT,
    URA,
    MW,
    RMW,
    UAR,
    EV,
    WR,
    SER,
    USR,
    LAT,
> where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
    CSM: ClientScopeMappingRepository,
    PM: ProtocolMapperRepository,
    OM: OrganizationMemberRepository,
    OR: OrganizationRepository,
    OAR: OrganizationAttributeRepository,
    GT: GroupTokenRepository,
    URA: UserRequiredActionRepository,
    MW: MaintenanceWhitelistRepository,
    RMW: RealmMaintenanceWhitelistRepository,
    UAR: UserAttributeRepository,
    EV: EmailVerificationService,
    WR: WebhookRepository,
    SER: SecurityEventRepository,
    USR: UserSessionRepository,
    LAT: LoginActionTokenRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) redirect_uri_repository: Arc<RU>,
    pub(crate) post_logout_redirect_uri_repository: Arc<PLRU>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) user_role_repository: Arc<UR>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) hasher_repository: Arc<H>,
    pub(crate) auth_session_repository: Arc<AS>,
    pub(crate) keystore_repository: Arc<KS>,
    pub(crate) refresh_token_repository: Arc<RT>,
    pub(crate) access_token_repository: Arc<AT>,
    pub(crate) federation_repository: Arc<F>,
    pub(crate) scope_mapping_repository: Arc<CSM>,
    pub(crate) protocol_mapper_repository: Arc<PM>,
    pub(crate) organization_member_repository: Arc<OM>,
    pub(crate) organization_repository: Arc<OR>,
    pub(crate) organization_attribute_repository: Arc<OAR>,
    pub(crate) group_token_repository: Arc<GT>,
    pub(crate) user_required_action_repository: Arc<URA>,
    pub(crate) maintenance_whitelist_repository: Arc<MW>,
    pub(crate) realm_maintenance_whitelist_repository: Arc<RMW>,
    pub(crate) user_attribute_repository: Arc<UAR>,
    pub(crate) email_verification_service: EV,
    pub(crate) webhook_repository: Arc<WR>,
    pub(crate) security_event_repository: Arc<SER>,
    pub(crate) user_session_repository: Arc<USR>,
    pub(crate) login_action_token_repository: Arc<LAT>,
    pub(crate) mapper_engine: Arc<MapperEngine>,
    pub(crate) ldap_client: LdapClientImpl,
    pub(crate) flow_recorder: FlowRecorder,
}

impl<
    R,
    C,
    RU,
    PLRU,
    U,
    UR,
    CR,
    H,
    AS,
    KS,
    RT,
    AT,
    F,
    CSM,
    PM,
    OM,
    OR,
    OAR,
    GT,
    URA,
    MW,
    RMW,
    UAR,
    EV,
    WR,
    SER,
    USR,
    LAT,
>
    AuthServiceImpl<
        R,
        C,
        RU,
        PLRU,
        U,
        UR,
        CR,
        H,
        AS,
        KS,
        RT,
        AT,
        F,
        CSM,
        PM,
        OM,
        OR,
        OAR,
        GT,
        URA,
        MW,
        RMW,
        UAR,
        EV,
        WR,
        SER,
        USR,
        LAT,
    >
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
    CSM: ClientScopeMappingRepository,
    PM: ProtocolMapperRepository,
    OM: OrganizationMemberRepository,
    OR: OrganizationRepository,
    OAR: OrganizationAttributeRepository,
    GT: GroupTokenRepository,
    URA: UserRequiredActionRepository,
    MW: MaintenanceWhitelistRepository,
    RMW: RealmMaintenanceWhitelistRepository,
    UAR: UserAttributeRepository,
    EV: EmailVerificationService,
    WR: WebhookRepository,
    SER: SecurityEventRepository,
    USR: UserSessionRepository,
    LAT: LoginActionTokenRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        redirect_uri_repository: Arc<RU>,
        post_logout_redirect_uri_repository: Arc<PLRU>,
        user_repository: Arc<U>,
        user_role_repository: Arc<UR>,
        credential_repository: Arc<CR>,
        hasher_repository: Arc<H>,
        auth_session_repository: Arc<AS>,
        keystore_repository: Arc<KS>,
        refresh_token_repository: Arc<RT>,
        access_token_repository: Arc<AT>,
        federation_repository: Arc<F>,
        scope_mapping_repository: Arc<CSM>,
        protocol_mapper_repository: Arc<PM>,
        organization_member_repository: Arc<OM>,
        organization_repository: Arc<OR>,
        organization_attribute_repository: Arc<OAR>,
        group_token_repository: Arc<GT>,
        user_required_action_repository: Arc<URA>,
        maintenance_whitelist_repository: Arc<MW>,
        realm_maintenance_whitelist_repository: Arc<RMW>,
        user_attribute_repository: Arc<UAR>,
        email_verification_service: EV,
        webhook_repository: Arc<WR>,
        security_event_repository: Arc<SER>,
        user_session_repository: Arc<USR>,
        login_action_token_repository: Arc<LAT>,
        mapper_engine: Arc<MapperEngine>,
        flow_recorder: FlowRecorder,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            redirect_uri_repository,
            post_logout_redirect_uri_repository,
            user_repository,
            user_role_repository,
            credential_repository,
            hasher_repository,
            auth_session_repository,
            keystore_repository,
            refresh_token_repository,
            access_token_repository,
            federation_repository,
            scope_mapping_repository,
            protocol_mapper_repository,
            organization_member_repository,
            organization_repository,
            organization_attribute_repository,
            group_token_repository,
            user_required_action_repository,
            maintenance_whitelist_repository,
            realm_maintenance_whitelist_repository,
            user_attribute_repository,
            email_verification_service,
            webhook_repository,
            security_event_repository,
            user_session_repository,
            login_action_token_repository,
            mapper_engine,
            ldap_client: LdapClientImpl,
            flow_recorder,
        }
    }
}

impl<
    R,
    C,
    RU,
    PLRU,
    U,
    UR,
    CR,
    H,
    AS,
    KS,
    RT,
    AT,
    F,
    CSM,
    PM,
    OM,
    OR,
    OAR,
    GT,
    URA,
    MW,
    RMW,
    UAR,
    EV,
    WR,
    SER,
    USR,
    LAT,
>
    AuthServiceImpl<
        R,
        C,
        RU,
        PLRU,
        U,
        UR,
        CR,
        H,
        AS,
        KS,
        RT,
        AT,
        F,
        CSM,
        PM,
        OM,
        OR,
        OAR,
        GT,
        URA,
        MW,
        RMW,
        UAR,
        EV,
        WR,
        SER,
        USR,
        LAT,
    >
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
    CSM: ClientScopeMappingRepository,
    PM: ProtocolMapperRepository,
    OM: OrganizationMemberRepository,
    OR: OrganizationRepository,
    OAR: OrganizationAttributeRepository,
    GT: GroupTokenRepository,
    URA: UserRequiredActionRepository,
    MW: MaintenanceWhitelistRepository,
    RMW: RealmMaintenanceWhitelistRepository,
    UAR: UserAttributeRepository,
    EV: EmailVerificationService,
    WR: WebhookRepository,
    SER: SecurityEventRepository,
    USR: UserSessionRepository,
    LAT: LoginActionTokenRepository,
{
    fn expires_in_from(exp: i64) -> u32 {
        let now = Utc::now().timestamp();
        if exp <= now { 0 } else { (exp - now) as u32 }
    }

    /// Returns `Some(locked_until)` when `new_attempts` has reached the threshold,
    /// otherwise `None` (counter incremented but not yet locked).
    fn compute_locked_until(
        new_attempts: i32,
        threshold: i32,
        duration_seconds: i32,
        now: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        lockout_compute_locked_until(new_attempts, threshold, duration_seconds, now)
    }

    async fn resolve_token_lifetimes(
        &self,
        realm_id: RealmId,
        client_uuid: Uuid,
    ) -> Result<TokenLifetimes, CoreError> {
        let realm_settings = self
            .realm_repository
            .get_realm_settings(realm_id)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_id(realm_id, client_uuid)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        Ok(TokenLifetimes::resolve(&realm_settings, &client))
    }

    async fn generate_token(&self, claims: JwtClaim, realm_id: RealmId) -> Result<Jwt, CoreError> {
        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(jwt_key_pair.id.to_string());
        let token =
            jsonwebtoken::encode(&header, &claims, &jwt_key_pair.encoding_key).map_err(|e| {
                tracing::error!("JWT generation error: {}", e);

                CoreError::TokenGenerationError(e.to_string())
            })?;

        let exp = claims.exp.unwrap_or(0);

        Ok(Jwt {
            token,
            expires_at: exp,
        })
    }

    fn encode_token_with_key<T: Serialize>(
        claims: &T,
        expires_at: i64,
        key_pair: &JwtKeyPair,
    ) -> Result<Jwt, CoreError> {
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(key_pair.id.to_string());
        let token = jsonwebtoken::encode(&header, claims, &key_pair.encoding_key).map_err(|e| {
            tracing::error!("JWT generation error: {}", e);
            CoreError::TokenGenerationError(e.to_string())
        })?;

        Ok(Jwt { token, expires_at })
    }

    /// Assemble access-token claims (and the ID-token mapper claims) for `input`, applying the
    /// client's scopes and protocol mappers. Pure computation: it neither signs nor persists a
    /// token. `create_jwt` uses it before signing/persisting; the client-scope evaluation
    /// preview uses it directly to show exactly what a real token would contain.
    async fn assemble_token_claims(
        &self,
        input: &GenerateTokenInput,
    ) -> Result<AssembledClaims, CoreError> {
        let iss = format!("{}/realms/{}", input.base_url, input.realm_name);
        let realm_audit = format!("{}-realm", input.realm_name);

        // Resolve protocol mappers from client scopes (default + requested optional)
        let mut applicable_scopes = self
            .scope_mapping_repository
            .get_default_scopes(input.client_uuid)
            .await
            .unwrap_or_default();

        let optional_scopes = self
            .scope_mapping_repository
            .get_optional_scopes(input.client_uuid)
            .await
            .unwrap_or_default();

        // Include optional scopes whose names appear in the resolved token scope
        let token_scope_names: HashSet<&str> = input
            .scope
            .as_deref()
            .map(|s| s.split_whitespace().collect())
            .unwrap_or_default();

        for scope in optional_scopes {
            if token_scope_names.contains(scope.name.as_str()) {
                applicable_scopes.push(scope);
            }
        }

        let mut all_mappers = Vec::new();
        for scope in &applicable_scopes {
            let mappers = self
                .protocol_mapper_repository
                .get_by_scope_id(scope.id)
                .await
                .unwrap_or_default();
            all_mappers.extend(mappers);
        }

        // Fetch user roles and group client-scoped roles by client string id.
        let user_roles = self
            .user_role_repository
            .get_user_roles(input.user_id)
            .await
            .unwrap_or_default();

        let mut realm_roles: Vec<String> = Vec::new();
        let mut client_roles: HashMap<String, Vec<String>> = HashMap::new();
        for role in &user_roles {
            match &role.client {
                Some(client) => client_roles
                    .entry(client.client_id.clone())
                    .or_default()
                    .push(role.name.clone()),
                None => realm_roles.push(role.name.clone()),
            }
        }

        // Roles inherited from the user's effective (recursive) group memberships, merged into
        // the role maps so they flow into tokens exactly like directly-assigned roles.
        let group_role_ids = self
            .group_token_repository
            .list_effective_role_ids_for_user(input.user_id)
            .await
            .unwrap_or_default();
        if !group_role_ids.is_empty() {
            let group_roles = self
                .user_role_repository
                .get_roles_by_ids(group_role_ids)
                .await
                .unwrap_or_default();
            for role in &group_roles {
                match &role.client {
                    Some(client) => client_roles
                        .entry(client.client_id.clone())
                        .or_default()
                        .push(role.name.clone()),
                    None => realm_roles.push(role.name.clone()),
                }
            }
        }

        // Organization-scoped roles: roles that apply only within a specific organization,
        // aggregated per org from (a) roles assigned directly to the membership and
        // (b) roles inherited from the user's groups in that org. These are surfaced *only*
        // under the `organizations.<alias>` claim (via the org-role mapper), never flattened
        // into the global `realm_access` / `resource_access` claims above.
        let mut org_role_ids: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        for (org_id, role_id) in self
            .group_token_repository
            .list_member_role_ids_by_org_for_user(input.user_id)
            .await
            .unwrap_or_default()
        {
            org_role_ids.entry(org_id).or_default().insert(role_id);
        }
        for (org_id, role_id) in self
            .group_token_repository
            .list_effective_group_role_ids_by_org_for_user(input.user_id)
            .await
            .unwrap_or_default()
        {
            org_role_ids.entry(org_id).or_default().insert(role_id);
        }

        // Resolve the distinct role ids once, then bucket per org into realm vs client roles.
        let mut org_roles: OrgScopedRoles = HashMap::new();
        if !org_role_ids.is_empty() {
            let distinct_ids: Vec<Uuid> = org_role_ids
                .values()
                .flatten()
                .copied()
                .collect::<HashSet<Uuid>>()
                .into_iter()
                .collect();
            let resolved: HashMap<Uuid, _> = self
                .user_role_repository
                .get_roles_by_ids(distinct_ids)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|role| (role.id, role))
                .collect();

            for (org_id, role_ids) in &org_role_ids {
                let entry = org_roles.entry(*org_id).or_default();
                for rid in role_ids {
                    if let Some(role) = resolved.get(rid) {
                        match &role.client {
                            Some(client) => entry
                                .1
                                .entry(client.client_id.clone())
                                .or_default()
                                .push(role.name.clone()),
                            None => entry.0.push(role.name.clone()),
                        }
                    }
                }
            }
        }

        // Effective groups (direct + ancestors) with their full paths, for the group mapper.
        // Direct ids let the mapper emit direct-only membership when configured.
        let effective_groups = self
            .group_token_repository
            .list_effective_groups_for_user(input.user_id)
            .await
            .unwrap_or_default();
        let direct_group_ids: HashSet<Uuid> = self
            .group_token_repository
            .list_direct_group_ids_for_user(input.user_id)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect();
        let groups = build_context_groups(&effective_groups, &direct_group_ids);

        // Load user organization memberships with their attributes
        let org_memberships = self
            .organization_member_repository
            .list_organizations_for_user(input.realm_id, input.user_id)
            .await
            .unwrap_or_default();

        // Union of the orgs the user is a direct member of and any org that granted scoped
        // roles (a user may inherit roles from a group in an org without a direct membership row).
        let mut org_ids: Vec<OrganizationId> = Vec::new();
        let mut seen_org_ids: HashSet<Uuid> = HashSet::new();
        for membership in &org_memberships {
            if seen_org_ids.insert(membership.organization_id.as_uuid()) {
                org_ids.push(membership.organization_id);
            }
        }
        for org_id in org_roles.keys() {
            if seen_org_ids.insert(*org_id) {
                org_ids.push(OrganizationId::new(*org_id));
            }
        }

        let mut organizations = Vec::new();
        for org_id in org_ids {
            if let Ok(Some(org)) = self
                .organization_repository
                .get_organization_by_id(org_id)
                .await
            {
                let raw_attrs = self
                    .organization_attribute_repository
                    .list_attributes(org.id)
                    .await
                    .unwrap_or_default();

                let attributes = raw_attrs.into_iter().map(|a| (a.key, a.value)).collect();

                let (roles, client_roles) = org_roles.remove(&org.id.as_uuid()).unwrap_or_default();

                organizations.push(ContextOrganization {
                    id: org.id,
                    name: org.name,
                    alias: org.alias,
                    domain: org.domain,
                    attributes,
                    roles,
                    client_roles,
                });
            }
        }

        // Load custom user attributes for protocol mappers
        let raw_user_attributes = self
            .user_attribute_repository
            .list_by_user_id(input.user_id)
            .await
            .unwrap_or_default();

        let user_attributes: HashMap<String, serde_json::Value> = raw_user_attributes
            .into_iter()
            .map(|a: UserAttribute| (a.key, serde_json::Value::String(a.value)))
            .collect();

        // Build mapper context
        let context = MapperContext {
            user_id: input.user_id,
            username: input.username.clone(),
            email: input.email.clone(),
            email_verified: input.email_verified,
            firstname: input.firstname.clone(),
            lastname: input.lastname.clone(),
            realm_roles,
            client_roles,
            client_id: input.client_id.clone(),
            client_uuid: input.client_uuid,
            realm_name: input.realm_name.clone(),
            realm_id: input.realm_id,
            user_attributes,
            organizations,
            groups,
        };

        // Apply mappers for access token
        let access_mapper_output =
            self.mapper_engine
                .apply_mappers(&all_mappers, &context, TokenType::AccessToken)?;

        let mut access_claims = JwtClaim::new(
            input.user_id,
            input.username.clone(),
            iss,
            vec![realm_audit, "account".to_string()],
            ClaimsTyp::Bearer,
            input.client_id.clone(),
            Some(input.email.clone()),
            input.scope.clone(),
            input.access_token_lifetime,
        );

        // Apply mapper output to claims
        for aud in &access_mapper_output.additional_audiences {
            if !access_claims.aud.contains(aud) {
                access_claims.aud.push(aud.clone());
            }
        }
        access_claims.additional_claims = access_mapper_output.claims;

        // `preferred_username` and `email` are now injected exclusively via protocol
        // mappers bound to the `profile` / `email` scopes.  Clearing the hard-coded
        // defaults here ensures they never leak into tokens whose scope set does not
        // include those scopes.
        access_claims.preferred_username = None;
        access_claims.email = None;

        // ID-token mapper claims are only produced when the `openid` scope is present.
        let id_mapper_claims = if input.scope.as_ref().is_some_and(|s| s.contains("openid")) {
            let id_mapper_output =
                self.mapper_engine
                    .apply_mappers(&all_mappers, &context, TokenType::IdToken)?;
            Some(id_mapper_output.claims)
        } else {
            None
        };

        Ok(AssembledClaims {
            access_claims,
            id_mapper_claims,
            effective_scopes: applicable_scopes,
            effective_mappers: all_mappers,
        })
    }

    /// Build the ID-token claims from already-assembled access-token claims plus the ID-token
    /// mapper output. `at_hash` is passed in because it depends on the *signed* access token
    /// (so it is `None` for previews that never sign a token).
    fn build_id_token_claims(
        access_claims: &JwtClaim,
        id_mapper_claims: HashMap<String, serde_json::Value>,
        at_hash: Option<String>,
        nonce: Option<String>,
        id_token_lifetime: i64,
    ) -> IdTokenClaims {
        let iat = Utc::now().timestamp();
        // Identity claims (preferred_username, email, email_verified) are injected into
        // `additional_claims` by the protocol mappers attached to the `profile`/`email`
        // scopes. The dedicated struct fields are intentionally left `None` to avoid
        // duplicate keys in the serialised JWT payload.
        IdTokenClaims {
            iss: access_claims.iss.clone(),
            aud: access_claims.azp.clone(),
            azp: Some(access_claims.azp.clone()),
            auth_time: None,
            email: None,
            email_verified: None,
            exp: iat + id_token_lifetime,
            iat,
            jti: Uuid::new_v4(),
            // OIDC `sid`, mirrored from the access token so RP-initiated logout and
            // back-channel logout can identify the session.
            sid: access_claims.sid.map(|sid| sid.to_string()),
            at_hash,
            nonce,
            preferred_username: None,
            sub: access_claims.sub,
            typ: ClaimsTyp::Id,
            additional_claims: id_mapper_claims,
        }
    }

    /// Compute the token claims a user would receive from this client for the given scopes,
    /// **without signing or persisting anything**. Powers the "Evaluate" client-scopes preview.
    /// Authorization and realm/client resolution are the caller's responsibility.
    pub async fn evaluate_client_scopes(
        &self,
        input: EvaluateClientScopesInput,
    ) -> Result<EvaluateClientScopesResult, CoreError> {
        let user = self.user_repository.get_by_id(input.user_id).await?;
        let lifetimes = self
            .resolve_token_lifetimes(input.realm_id, input.client_uuid)
            .await?;

        let gen_input = GenerateTokenInput {
            base_url: input.base_url,
            realm_name: input.realm_name,
            user_id: user.id,
            username: user.username.clone(),
            firstname: user.firstname.clone().unwrap_or_default(),
            lastname: user.lastname.clone().unwrap_or_default(),
            email_verified: user.email_verified,
            client_id: input.client_id,
            client_uuid: input.client_uuid,
            email: user.email.clone().unwrap_or_default(),
            realm_id: input.realm_id,
            scope: input.scope,
            access_token_lifetime: lifetimes.access_token,
            refresh_token_lifetime: lifetimes.refresh_token,
            id_token_lifetime: lifetimes.id_token,
            nonce: None,
            refresh_jti_override: None,
            // Preview only — nothing is signed or persisted, so there is no session.
            session_id: None,
        };

        let assembled = self.assemble_token_claims(&gen_input).await?;

        // Effective role scope mappings: the user's directly-assigned roles plus roles inherited
        // from their effective (recursive) group memberships — matching what tokens carry.
        let mut user_roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .unwrap_or_default();
        let group_role_ids = self
            .group_token_repository
            .list_effective_role_ids_for_user(user.id)
            .await
            .unwrap_or_default();
        if !group_role_ids.is_empty() {
            let group_roles = self
                .user_role_repository
                .get_roles_by_ids(group_role_ids)
                .await
                .unwrap_or_default();
            user_roles.extend(group_roles);
        }
        let mut realm_roles = Vec::new();
        let mut client_roles: HashMap<String, Vec<String>> = HashMap::new();
        for role in &user_roles {
            match &role.client {
                Some(client) => client_roles
                    .entry(client.client_id.clone())
                    .or_default()
                    .push(role.name.clone()),
                None => realm_roles.push(role.name.clone()),
            }
        }
        realm_roles.sort();
        realm_roles.dedup();
        for roles in client_roles.values_mut() {
            roles.sort();
            roles.dedup();
        }

        let access_token = serde_json::to_value(&assembled.access_claims)
            .map_err(|_| CoreError::InternalServerError)?;

        let sub = assembled.access_claims.sub.to_string();
        let (id_token, userinfo) = match assembled.id_mapper_claims {
            Some(id_mapper_claims) => {
                // No signed access token in a preview → no `at_hash`.
                let id_claims = Self::build_id_token_claims(
                    &assembled.access_claims,
                    id_mapper_claims.clone(),
                    None,
                    None,
                    gen_input.id_token_lifetime,
                );
                let id_token =
                    serde_json::to_value(&id_claims).map_err(|_| CoreError::InternalServerError)?;

                // OIDC userinfo carries at least `sub` plus the ID-token mapper claims.
                let mut userinfo: serde_json::Map<String, serde_json::Value> =
                    id_mapper_claims.into_iter().collect();
                userinfo.insert("sub".to_string(), serde_json::Value::String(sub));
                (Some(id_token), serde_json::Value::Object(userinfo))
            }
            None => {
                let mut userinfo = serde_json::Map::new();
                userinfo.insert("sub".to_string(), serde_json::Value::String(sub));
                (None, serde_json::Value::Object(userinfo))
            }
        };

        let effective_scopes = assembled
            .effective_scopes
            .into_iter()
            .map(|s| EvaluatedScope {
                name: s.name,
                protocol: s.protocol,
                default_scope_type: format!("{:?}", s.default_scope_type),
            })
            .collect();

        let effective_mappers = assembled
            .effective_mappers
            .into_iter()
            .map(|m| EvaluatedMapper {
                name: m.name,
                mapper_type: m.mapper_type,
                config: m.config,
            })
            .collect();

        Ok(EvaluateClientScopesResult {
            effective_scopes,
            effective_mappers,
            effective_roles: EvaluatedRoles {
                realm_roles,
                client_roles,
            },
            access_token,
            id_token,
            userinfo,
        })
    }

    /// Open the SSO session a login is bound to, and record it in the audit trail.
    ///
    /// The session is given the same lifetime as the refresh token, so that the
    /// window in which a user can keep renewing tokens is exactly the window in
    /// which their session is alive. A dedicated realm-level SSO session lifetime
    /// would be a better fit and is left for follow-up.
    ///
    /// `user_agent` and `ip_address` are left unset: the token endpoint is a
    /// back-channel call made by the client application, so its transport metadata
    /// describes the client rather than the user's browser. Recording the real
    /// values means carrying them from the `/auth` request through `auth_sessions`.
    async fn create_user_session(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        session_lifetime_seconds: i64,
    ) -> Result<UserSession, CoreError> {
        // The lifetime comes from realm/client settings, so it is operator-supplied.
        // `Duration::seconds` panics out of range — never let a bad setting take the
        // token endpoint down.
        let session_duration =
            Duration::try_seconds(session_lifetime_seconds).ok_or_else(|| {
                warn!(
                    "Refusing to open a session: refresh token lifetime {} is out of range",
                    session_lifetime_seconds
                );
                CoreError::SessionCreateError
            })?;

        if let Err(e) = self
            .user_session_repository
            .delete_expired_for_user(user_id, realm_id.into(), Utc::now())
            .await
        {
            warn!(
                user_id = %user_id,
                error = ?e,
                "Failed to purge expired sessions before opening a new one"
            );
        }

        let session =
            UserSession::new(user_id, realm_id.into(), None, None, session_duration, None);

        self.user_session_repository
            .create(&session)
            .await
            .map_err(|e| {
                warn!(
                    "Failed to create user session for user {}: {:?}",
                    user_id, e
                );
                CoreError::SessionCreateError
            })?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::SessionCreated,
                    EventStatus::Success,
                    user_id,
                )
                .with_target("session".to_string(), session.id, None),
            )
            .await?;

        Ok(session)
    }

    async fn revoke_session_cascade(
        &self,
        session_id: Uuid,
        realm_id: RealmId,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        let (access_revoked, refresh_revoked) = tokio::try_join!(
            self.access_token_repository
                .revoke_by_session_id(session_id),
            self.refresh_token_repository
                .revoke_by_session_id(session_id),
        )
        .map_err(|e| {
            warn!(
                session_id = %session_id,
                error = ?e,
                "Failed to revoke the tokens minted against a session"
            );
            CoreError::InternalServerError
        })?;

        if let Err(e) = self.user_session_repository.delete(&session_id).await {
            warn!(
                session_id = %session_id,
                error = ?e,
                "Tokens revoked but the session row could not be deleted"
            );
        }

        info!(
            session_id = %session_id,
            access_revoked,
            refresh_revoked,
            "Revoked a session and every token minted against it"
        );

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm_id,
                    SecurityEventType::SessionRevoked,
                    EventStatus::Success,
                    user_id,
                )
                .with_target("session".to_string(), session_id, None),
            )
            .await
            .map_err(|err| warn!("Failed to store SessionRevoked security event: {}", err))
            .ok();

        Ok(())
    }

    async fn create_jwt(
        &self,
        input: GenerateTokenInput,
    ) -> Result<(Jwt, Jwt, Option<Jwt>), CoreError> {
        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(input.realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let AssembledClaims {
            access_claims: mut claims,
            id_mapper_claims,
            ..
        } = self.assemble_token_claims(&input).await?;

        // Bind the token pair to the SSO session so introspection and refresh can
        // check it is still alive. Absent for flows that establish no session.
        claims.sid = input.session_id;

        let jwt = Self::encode_token_with_key(&claims, claims.exp.unwrap_or(0), &jwt_key_pair)
            .map_err(|e| {
                warn!("Failed to generate JWT: {:?}", e);
                e
            })?;

        // Persist access tokens so backend services can introspect/revoke them immediately.
        let access_token_hash = format!("{:x}", Sha256::digest(jwt.token.as_bytes()));
        let access_token_claims =
            serde_json::to_value(&claims).map_err(|_| CoreError::InternalServerError)?;
        let access_token_expires_at = claims
            .exp
            .and_then(|exp| Utc.timestamp_opt(exp, 0).single());

        let mut refresh_claims = JwtClaim::new_refresh_token(
            claims.sub,
            claims.iss.clone(),
            claims.aud.clone(),
            claims.azp.clone(),
            claims.scope.clone(),
            input.refresh_token_lifetime,
        );

        refresh_claims.sid = input.session_id;

        // When the caller has already persisted the refresh token row (rotation path),
        // override the jti so the signed JWT matches the DB record exactly.
        if let Some(override_jti) = input.refresh_jti_override {
            refresh_claims.jti = override_jti;
        }

        let refresh_token = Self::encode_token_with_key(
            &refresh_claims,
            refresh_claims.exp.unwrap_or(0),
            &jwt_key_pair,
        )?;

        let id_token: Option<Jwt> = if let Some(id_mapper_claims) = id_mapper_claims {
            // at_hash = base64url(left-half of SHA-256(access_token))
            let at_hash = {
                let digest = Sha256::digest(jwt.token.as_bytes());
                Some(BASE64_URL_SAFE_NO_PAD.encode(&digest[..digest.len() / 2]))
            };

            let id_claims = Self::build_id_token_claims(
                &claims,
                id_mapper_claims,
                at_hash,
                input.nonce.clone(),
                input.id_token_lifetime,
            );
            let t = Self::encode_token_with_key(&id_claims, id_claims.exp, &jwt_key_pair)?;

            Some(t)
        } else {
            None
        };

        let refresh_token_expires_at = Utc
            .timestamp_opt(refresh_token.expires_at, 0)
            .single()
            .ok_or(CoreError::InternalServerError)?;

        // When rotating, the refresh token DB row is already committed by `rotate()`.
        // Skip re-inserting it; only persist the new access token.
        if input.refresh_jti_override.is_some() {
            self.access_token_repository
                .create(
                    access_token_hash,
                    Some(claims.jti),
                    claims.sub,
                    input.realm_id,
                    access_token_expires_at,
                    access_token_claims,
                )
                .await
                .map_err(|_| CoreError::InternalServerError)?;
        } else {
            tokio::try_join!(
                self.access_token_repository.create(
                    access_token_hash,
                    Some(claims.jti),
                    claims.sub,
                    input.realm_id,
                    access_token_expires_at,
                    access_token_claims,
                ),
                self.refresh_token_repository.create(
                    refresh_claims.jti,
                    input.user_id,
                    Some(refresh_token_expires_at),
                    input.session_id,
                )
            )
            .map_err(|_| CoreError::InternalServerError)?;
        }

        Ok((jwt, refresh_token, id_token))
    }

    #[instrument(skip(self, token))]
    async fn verify_token(&self, token: String, realm_id: RealmId) -> Result<JwtClaim, CoreError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);

        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        validation.validate_aud = false;
        let token_data =
            jsonwebtoken::decode::<JwtClaim>(&token, &jwt_key_pair.decoding_key, &validation)
                .map_err(|e| CoreError::TokenValidationError(format!("{:?}: {}", e.kind(), e)))?;

        let current_time = Utc::now().timestamp();

        if let Some(exp) = token_data.claims.exp
            && exp < current_time
        {
            return Err(CoreError::ExpiredToken);
        }

        let session = match token_data.claims.sid {
            Some(sid) => self
                .user_session_repository
                .find_by_id(sid)
                .await
                .map_err(|e| {
                    warn!(session_id = %sid, error = ?e, "Failed to load the session backing a token");
                    CoreError::InternalServerError
                })?,
            None => None,
        };

        validate_session_binding(token_data.claims.sid, session.as_ref(), Utc::now())?;

        // Enforce immediate access token revocation when a persisted token has been marked revoked.
        if token_data.claims.typ == ClaimsTyp::Bearer {
            let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
            if let Some(stored) = self
                .access_token_repository
                .get_by_token_hash(token_hash)
                .await
                .map_err(|_| CoreError::InternalServerError)?
                && stored.revoked
            {
                return Err(CoreError::InvalidToken);
            }
        }

        Ok(token_data.claims)
    }

    async fn verify_password(&self, user_id: Uuid, password: String) -> Result<bool, CoreError> {
        let credential = self
            .credential_repository
            .get_password_credential(user_id)
            .instrument(info_span!("auth.verify_password.credential_fetch"))
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let salt = credential.salt.ok_or(CoreError::InternalServerError)?;

        let CredentialData::Hash {
            hash_iterations,
            algorithm,
        } = credential.credential_data
        else {
            return Err(CoreError::InternalServerError);
        };

        let is_valid = self
            .hasher_repository
            .verify_password(
                &password,
                &credential.secret_data,
                hash_iterations,
                &algorithm,
                &salt,
            )
            .instrument(info_span!(
                "auth.verify_password.hasher_verify",
                hash_algorithm = %algorithm,
                hash_iterations
            ))
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(is_valid)
    }

    async fn refuse_login_identifier_collision(
        &self,
        realm: &crate::domain::realm::entities::Realm,
        username: &str,
        email: &str,
    ) -> Result<(), CoreError> {
        let aliases = realm
            .settings
            .as_ref()
            .map(|settings| settings.login_aliases.clone())
            .unwrap_or_default();

        if aliases.as_slice().len() < 2 {
            return Ok(());
        }

        if self
            .user_repository
            .get_by_email(username, realm.id)
            .await?
            .is_some()
        {
            return Err(CoreError::UsernameAlreadyExists);
        }

        if self
            .user_repository
            .find_by_username(email.to_string(), realm.id)
            .await?
            .is_some()
        {
            return Err(CoreError::EmailAlreadyExists);
        }

        Ok(())
    }

    async fn resolve_pending_auth_step(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
    ) -> Result<Option<mfa_policy::PendingAuthStep>, CoreError> {
        let persisted_actions = self
            .user_required_action_repository
            .get_required_actions(user_id)
            .await
            .map_err(|e| {
                warn!(user_id = %user_id, error = ?e, "Failed to load required actions");
                CoreError::InternalServerError
            })?;

        let credentials = self
            .credential_repository
            .get_credentials_by_user_id(user_id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        let has_otp_credential = credentials
            .iter()
            .any(|credential| matches!(credential.credential_type, CredentialType::Otp));
        let has_temporary_password = credentials.iter().any(|credential| credential.temporary);

        let user_roles = self
            .user_role_repository
            .get_user_roles(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let realm_settings = self.realm_repository.get_realm_settings(realm_id).await?;

        Ok(mfa_policy::pending_auth_step(
            &persisted_actions,
            realm_settings.as_ref(),
            &user_roles,
            has_otp_credential,
            has_temporary_password,
        ))
    }

    async fn verify_refresh_token(
        &self,
        token: String,
        realm_id: RealmId,
    ) -> Result<(JwtClaim, crate::domain::jwt::entities::RefreshToken), CoreError> {
        let claims = self.verify_token(token, realm_id).await?;

        let refresh_token = self
            .refresh_token_repository
            .get_by_jti(claims.jti)
            .await
            .map_err(|error| match error {
                JwtError::InvalidToken | JwtError::ExpiredToken => CoreError::InvalidRefreshToken,
                _ => CoreError::InternalServerError,
            })?;

        if let Some(expires_at) = refresh_token.expires_at
            && expires_at < chrono::Utc::now()
        {
            return Err(CoreError::ExpiredToken);
        }

        Ok((claims, refresh_token))
    }

    /// Resolve the final scope string for a given client and requested scope.
    ///
    /// Rules:
    /// - Client's default scopes are always included.
    /// - Client's optional scopes are included only when explicitly requested.
    /// - Standard OIDC scopes (openid, profile, email, etc.) are always permitted.
    /// - Any requested scope not in the above sets returns `CoreError::InvalidScope`.
    /// - Falls back to `profile email` defaults when the client has no configured scopes.
    pub(crate) async fn resolve_scopes_for_client(
        &self,
        client_uuid: Uuid,
        requested_scope: Option<String>,
    ) -> Result<String, CoreError> {
        let default_scopes = self
            .scope_mapping_repository
            .get_default_scopes(client_uuid)
            .await
            .unwrap_or_default();

        let optional_scopes = self
            .scope_mapping_repository
            .get_optional_scopes(client_uuid)
            .await
            .unwrap_or_default();

        let default_scope_names: HashSet<String> =
            default_scopes.iter().map(|s| s.name.clone()).collect();
        let optional_scope_names: HashSet<String> =
            optional_scopes.iter().map(|s| s.name.clone()).collect();

        // Always include the client's default scopes
        let mut final_scopes: HashSet<String> = default_scope_names.clone();

        // Fall back to OIDC defaults when the client has no configured default scopes
        if final_scopes.is_empty() {
            final_scopes.insert(OidcScope::Profile.to_string());
            final_scopes.insert(OidcScope::Email.to_string());
        }

        if let Some(scope_str) = requested_scope {
            for scope in scope_str.split_whitespace() {
                if OidcScope::is_standard(scope) {
                    // Standard OIDC scopes are always permitted
                    final_scopes.insert(scope.to_string());
                } else if optional_scope_names.contains(scope) {
                    // Optional client scope requested explicitly
                    final_scopes.insert(scope.to_string());
                } else if default_scope_names.contains(scope) {
                    // Already present as a default scope — no-op
                } else {
                    // Scope is not assigned to this client
                    return Err(CoreError::InvalidScope(format!(
                        "Scope '{}' is not assigned to this client",
                        scope
                    )));
                }
            }
        }

        let mut sorted: Vec<String> = final_scopes.into_iter().collect();
        sorted.sort_by(|a, b| {
            if a == OidcScope::OpenId.as_str() {
                std::cmp::Ordering::Less
            } else if b == OidcScope::OpenId.as_str() {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        Ok(sorted.join(" "))
    }

    fn verify_pkce(
        code_verifier: &str,
        code_challenge: &str,
        method: &CodeChallengeMethod,
    ) -> bool {
        pkce_verify(code_verifier, code_challenge, method)
    }

    fn validate_code_verifier(verifier: &str) -> bool {
        pkce_validate_verifier(verifier)
    }

    async fn authorization_code(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let code = params.code.ok_or(CoreError::InternalServerError)?;
        let step_start = Utc::now();

        // Authenticate the caller before touching the code, so an unauthenticated
        // request can never reach the code lookup at all.
        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        // The code itself is a secret: never log it, since log exposure is one of
        // the ways it gets into an attacker's hands in the first place.
        let auth_session = self
            .auth_session_repository
            .get_by_code(code.clone())
            .await
            .map_err(|e| {
                warn!(client_id = %params.client_id, error = ?e, "Auth session lookup failed");

                CoreError::MissingAuthorizationCode
            })?
            .ok_or(CoreError::InvalidAuthorizationCode)?;

        if auth_session.authenticated {
            warn!(
                client_id = %params.client_id,
                "Authorization code has already been used"
            );
            return Err(CoreError::InvalidAuthorizationCode);
        }

        validate_authorization_code_request(
            &auth_session,
            &client,
            params.realm_id,
            params.redirect_uri.as_deref(),
            params.client_secret.as_deref(),
            Utc::now(),
        )?;

        // PKCE verification (RFC 7636 §4.6).
        match (
            &auth_session.code_challenge,
            &auth_session.code_challenge_method,
        ) {
            (Some(challenge), method) => {
                let verifier = params
                    .code_verifier
                    .as_deref()
                    .ok_or(CoreError::CodeVerifierMissing)?;

                if !Self::validate_code_verifier(verifier) {
                    return Err(CoreError::InvalidCodeVerifier);
                }

                let method = method.as_ref().unwrap_or(&CodeChallengeMethod::Plain);
                if !Self::verify_pkce(verifier, challenge, method) {
                    warn!(client_id = %params.client_id, "PKCE verification failed");
                    return Err(CoreError::InvalidCodeVerifier);
                }
            }
            (None, _) => {
                // No challenge stored — check if a verifier was sent unexpectedly
                // (harmless per RFC 7636 §4.6, we simply ignore it).
            }
        }

        let flow_id = auth_session.compass_flow_id.map(FlowId);
        let user_id = auth_session.user_id.ok_or(CoreError::NotFound)?;
        let user = self.user_repository.get_by_id(user_id).await?;

        let pending_step = self
            .resolve_pending_auth_step(user_id, params.realm_id)
            .await?;

        if let Err(error) = refuse_token_issuance_when_actions_pending(pending_step.as_ref()) {
            warn!(
                user_id = %user_id,
                client_id = %params.client_id,
                step = ?pending_step,
                "Refusing an authorization code: the account still owes a required action"
            );
            return Err(error);
        }

        let final_scope = self
            .resolve_scopes_for_client(auth_session.client_id, auth_session.scope.clone())
            .await?;

        info!("Final scope for authorization code grant: {}", final_scope);

        let lifetimes = self
            .resolve_token_lifetimes(params.realm_id, auth_session.client_id)
            .await?;

        // The SSO session backing this login. Every token minted below carries its
        // id as `sid`, which is what lets revocation take effect on introspection
        // and refresh.
        let user_session = self
            .create_user_session(user.id, params.realm_id, lifetimes.refresh_token)
            .await?;

        let (jwt, refresh_token, id_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id.clone(),
                client_uuid: auth_session.client_id,
                email: user.email.clone().unwrap_or_default(),
                email_verified: user.email_verified,
                firstname: user.firstname.clone().unwrap_or_default(),
                lastname: user.lastname.clone().unwrap_or_default(),
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username.clone(),
                scope: Some(final_scope.clone()),
                access_token_lifetime: lifetimes.access_token,
                refresh_token_lifetime: lifetimes.refresh_token,
                id_token_lifetime: lifetimes.id_token,
                nonce: auth_session.nonce.clone(),
                refresh_jti_override: None,
                session_id: Some(user_session.id),
            })
            .await
            .map_err(|e| {
                warn!("Failed to create JWT for authorization code grant: {:?}", e);
                if let Some(ref fid) = flow_id {
                    let duration = (Utc::now() - step_start).num_milliseconds();
                    self.flow_recorder.record_step(
                        fid.clone(),
                        FlowStepName::TokenExchange,
                        StepStatus::Failure,
                        Some(duration),
                        Some(format!("{:?}", e)),
                        None,
                    );
                    self.flow_recorder.complete_flow(
                        fid.clone(),
                        FlowStatus::Failure,
                        duration,
                        Some(user_id),
                    );
                }
                e
            })?;

        if let Some(ref fid) = flow_id {
            let duration = (Utc::now() - step_start).num_milliseconds();
            self.flow_recorder.record_step(
                fid.clone(),
                FlowStepName::TokenExchange,
                StepStatus::Success,
                Some(duration),
                None,
                None,
            );
            self.flow_recorder.complete_flow(
                fid.clone(),
                FlowStatus::Success,
                duration,
                Some(user_id),
            );
        }

        self.auth_session_repository
            .update_authenticated(auth_session.id, true)
            .await
            .map_err(|e| {
                warn!("Failed to mark auth session as authenticated: {:?}", e);
                CoreError::InternalServerError
            })?;

        let id_token_value = id_token.map(|t| t.token);

        info!(
            "Generated JWT for authorization code grant, user_id: {}, client_id: {}, scope: {}",
            user.id, params.client_id, final_scope
        );

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            Self::expires_in_from(refresh_token.expires_at),
            None,
            id_token_value,
        ))
    }

    async fn client_credential(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if !Self::verify_client_secret(client.secret_str(), params.client_secret.as_deref()) {
            return Err(CoreError::InvalidClientSecret);
        }

        if let Some(ref scope_str) = params.scope {
            for scope in scope_str.split_whitespace() {
                if scope == OidcScope::OfflineAccess.as_str() {
                    return Err(CoreError::InvalidScope(
                        "Scope 'offline_access' is not allowed for client credentials grant"
                            .to_string(),
                    ));
                }
            }
        }

        info!("try to fetch user client, client id: {}", client.id);

        let user = self
            .user_repository
            .get_by_client_id(client.id)
            .await
            .map_err(|e| match e {
                CoreError::NotFound => CoreError::ServiceAccountNotFound,
                _ => CoreError::InternalServerError,
            })?;

        let final_scope = self
            .resolve_scopes_for_client(client.id, params.scope)
            .await?;

        let lifetimes = self
            .resolve_token_lifetimes(params.realm_id, client.id)
            .await?;

        let (jwt, refresh_token, id_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                client_uuid: client.id,
                email: user.email.clone().unwrap_or_default(),
                email_verified: user.email_verified,
                firstname: user.firstname.clone().unwrap_or_default(),
                lastname: user.lastname.clone().unwrap_or_default(),
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
                scope: Some(final_scope),
                access_token_lifetime: lifetimes.access_token,
                refresh_token_lifetime: lifetimes.refresh_token,
                id_token_lifetime: lifetimes.id_token,
                nonce: None,
                refresh_jti_override: None,
                // Machine-to-machine: no user is present, so no SSO session exists.
                session_id: None,
            })
            .await?;

        let id_token_value = id_token.map(|t| t.token);

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            Self::expires_in_from(refresh_token.expires_at),
            None,
            id_token_value,
        ))
    }

    async fn password(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let username = params.username.ok_or(CoreError::InternalServerError)?;
        let password = params.password.ok_or(CoreError::InternalServerError)?;

        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .instrument(info_span!("auth.password.client_lookup"))
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if !client.direct_access_grants_enabled {
            // Public clients must have direct access grants enabled for password flow.
            if client.public_client {
                return Err(CoreError::InvalidClient);
            }

            // Confidential clients are still allowed when authenticating with a valid secret.
            if !Self::verify_client_secret(client.secret_str(), params.client_secret.as_deref()) {
                return Err(CoreError::InvalidClientSecret);
            }
        } else if !client.public_client {
            // When direct access grants are enabled, confidential clients may call
            // password flow without a secret; if one is provided, it must be valid.
            if let Some(provided_secret) = &params.client_secret
                && !Self::verify_client_secret(client.secret_str(), Some(provided_secret))
            {
                return Err(CoreError::InvalidClientSecret);
            }
        }

        let login_aliases = self
            .realm_repository
            .get_realm_settings(params.realm_id)
            .await?
            .map(|s| s.login_aliases)
            .unwrap_or_default();

        let user = crate::domain::authentication::login_resolver::resolve_user_by_identifier(
            self.user_repository.as_ref(),
            &username,
            params.realm_id,
            &login_aliases,
        )
        .instrument(info_span!("auth.password.user_lookup"))
        .await?
        .ok_or(CoreError::Invalid)?;

        if !user.enabled {
            return Err(CoreError::UserDisabled);
        }

        let realm_settings = self
            .realm_repository
            .get_realm_settings(params.realm_id)
            .await?;

        let lockout_threshold = realm_settings
            .as_ref()
            .map(|s| s.lockout_threshold)
            .unwrap_or(10);
        let lockout_duration_seconds = realm_settings
            .as_ref()
            .map(|s| s.lockout_duration_seconds)
            .unwrap_or(900);

        let now = Utc::now();
        if user.is_locked(now) {
            return Err(CoreError::AccountLocked);
        }

        let credential = self
            .verify_password(user.id, password)
            .instrument(info_span!("auth.password.verify_password"))
            .await;

        let is_valid = match credential {
            Ok(is_valid) => is_valid,
            Err(_) => {
                let locked_until = Self::compute_locked_until(
                    user.failed_login_attempts + 1,
                    lockout_threshold,
                    lockout_duration_seconds,
                    now,
                );
                let _ = self
                    .user_repository
                    .increment_failed_login_attempts(user.id, locked_until)
                    .await;
                return Err(CoreError::Invalid);
            }
        };

        if !is_valid {
            let locked_until = Self::compute_locked_until(
                user.failed_login_attempts + 1,
                lockout_threshold,
                lockout_duration_seconds,
                now,
            );
            let _ = self
                .user_repository
                .increment_failed_login_attempts(user.id, locked_until)
                .await;
            return Err(CoreError::Invalid);
        }

        let _ = self
            .user_repository
            .reset_failed_login_attempts(user.id)
            .await;

        let pending_step = self
            .resolve_pending_auth_step(user.id, params.realm_id)
            .instrument(info_span!("auth.password.pending_step"))
            .await?;

        if let Err(error) = refuse_token_issuance_when_step_pending(pending_step.as_ref()) {
            warn!(
                user_id = %user.id,
                client_id = %params.client_id,
                step = ?pending_step,
                "Refusing a direct grant: the account still owes an authentication step"
            );
            return Err(error);
        }

        let final_scope = self
            .resolve_scopes_for_client(client.id, params.scope)
            .await?;

        let lifetimes = self
            .resolve_token_lifetimes(params.realm_id, client.id)
            .await?;

        let user_session = self
            .create_user_session(user.id, params.realm_id, lifetimes.refresh_token)
            .await?;

        let (jwt, refresh_token, id_token) = self
            .create_jwt(GenerateTokenInput {
                base_url: params.base_url,
                client_id: params.client_id,
                client_uuid: client.id,
                email: user.email.clone().unwrap_or_default(),
                email_verified: user.email_verified,
                firstname: user.firstname.clone().unwrap_or_default(),
                lastname: user.lastname.clone().unwrap_or_default(),
                realm_id: params.realm_id,
                realm_name: params.realm_name,
                user_id: user.id,
                username: user.username,
                scope: Some(final_scope),
                access_token_lifetime: lifetimes.access_token,
                refresh_token_lifetime: lifetimes.refresh_token,
                id_token_lifetime: lifetimes.id_token,
                nonce: None,
                refresh_jti_override: None,
                session_id: Some(user_session.id),
            })
            .instrument(info_span!("auth.password.create_jwt"))
            .await?;

        let id_token_value = id_token.map(|t| t.token);

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            Self::expires_in_from(refresh_token.expires_at),
            None,
            id_token_value,
        ))
    }

    async fn refresh_token(&self, params: GrantTypeParams) -> Result<JwtToken, CoreError> {
        let token_str = params.refresh_token.ok_or(CoreError::InvalidRefreshToken)?;

        let (claims, stored) = self
            .verify_refresh_token(token_str, params.realm_id)
            .await
            // A refresh presented against a revoked or expired session is a
            // grant failure, not an authentication failure: RFC 6749 §5.2 asks
            // the token endpoint for `400 invalid_grant`.
            .map_err(revoked_session_is_an_invalid_grant)?;

        if claims.typ != ClaimsTyp::Refresh {
            return Err(CoreError::InvalidToken);
        }

        if claims.azp != params.client_id {
            tracing::warn!("invalid client id: {:?}", claims.azp);
            return Err(CoreError::InvalidToken);
        }

        // Reuse detection: rotated or revoked tokens trigger family revocation.
        if !stored.status.is_active() {
            warn!(
                family_id = %stored.family_id,
                status = stored.status.as_str(),
                "refresh token reuse detected — revoking family"
            );
            self.refresh_token_repository
                .revoke_family(stored.family_id)
                .await
                .map_err(|_| CoreError::InternalServerError)?;
            return Err(CoreError::InvalidRefreshToken);
        }

        let user = self
            .user_repository
            .get_by_id(claims.sub)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if !user.enabled {
            return Err(CoreError::UserDisabled);
        }

        let client = self
            .client_repository
            .get_by_client_id(params.client_id.clone(), params.realm_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        let lifetimes = self
            .resolve_token_lifetimes(params.realm_id, client.id)
            .await?;

        let new_refresh_jti = Uuid::new_v4();
        let new_refresh_expires_at = {
            let ts = Utc::now().timestamp() + lifetimes.refresh_token;
            Utc.timestamp_opt(ts, 0)
                .single()
                .ok_or(CoreError::InternalServerError)?
        };

        // Atomic conditional rotate: marks old token as 'rotated' and mints successor.
        let outcome = self
            .refresh_token_repository
            .rotate(
                stored.id,
                new_refresh_jti,
                user.id,
                stored.family_id,
                Some(new_refresh_expires_at),
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        match outcome {
            RotateOutcome::Conflict => {
                // A concurrent request already rotated this token — fail safe.
                warn!(
                    family_id = %stored.family_id,
                    "concurrent refresh token rotation detected — revoking family"
                );
                self.refresh_token_repository
                    .revoke_family(stored.family_id)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;
                Err(CoreError::InvalidRefreshToken)
            }
            RotateOutcome::Rotated(new_stored) => {
                // `rotate()` already committed the successor refresh token row.
                // Pass its jti as an override so `create_jwt` signs a matching JWT
                // without inserting a duplicate DB row.
                let (jwt, rotated_refresh_jwt, id_token) = self
                    .create_jwt(GenerateTokenInput {
                        base_url: params.base_url,
                        client_id: params.client_id,
                        client_uuid: client.id,
                        email: user.email.clone().unwrap_or_default(),
                        email_verified: user.email_verified,
                        firstname: user.firstname.clone().unwrap_or_default(),
                        lastname: user.lastname.clone().unwrap_or_default(),
                        realm_id: params.realm_id,
                        realm_name: params.realm_name,
                        user_id: user.id,
                        username: user.username,
                        scope: claims.scope.clone(),
                        access_token_lifetime: lifetimes.access_token,
                        refresh_token_lifetime: lifetimes.refresh_token,
                        id_token_lifetime: lifetimes.id_token,
                        nonce: None,
                        refresh_jti_override: Some(new_stored.jti),
                        // Carry the session binding across rotation, otherwise a
                        // single refresh would silently detach the token pair from
                        // its session and make it immune to revocation.
                        session_id: claims.sid,
                    })
                    .await?;

                let id_token_value = id_token.map(|t| t.token);
                let refresh_expires_in = Self::expires_in_from(
                    new_stored.expires_at.map(|dt| dt.timestamp()).unwrap_or(0),
                );

                Ok(JwtToken::new(
                    jwt.token,
                    "Bearer".to_string(),
                    rotated_refresh_jwt.token,
                    Self::expires_in_from(jwt.expires_at),
                    refresh_expires_in,
                    None,
                    id_token_value,
                ))
            }
        }
    }

    async fn authenticate_with_grant_type(
        &self,
        grant_type: GrantType,
        params: GrantTypeParams,
    ) -> Result<JwtToken, CoreError> {
        match grant_type {
            GrantType::Code => self.authorization_code(params).await,
            GrantType::Password => self.password(params).await,
            GrantType::Credentials => self.client_credential(params).await,
            GrantType::RefreshToken => self.refresh_token(params).await,
            // Device flow token exchange is not wired up yet (see #1020).
            GrantType::DeviceCode => Err(CoreError::InvalidRequest),
            // RFC 8693 token exchange is dispatched here once the exchange
            // service lands (see #1053/#1054); not wired up yet.
            GrantType::TokenExchange => Err(CoreError::InvalidRequest),
        }
    }

    async fn handle_user_credentials_authentication(
        &self,
        params: CredentialsAuthParams,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
        let flow_id = auth_session.compass_flow_id.map(FlowId);
        let step_start = Utc::now();

        let auth_result = self
            .using_session_code(
                params.realm_name,
                params.client_id,
                params.session_code,
                params.username,
                params.password,
                params.base_url,
            )
            .await
            .map_err(|e| {
                warn!("authentication using session code error: {:?}", e);
                if let Some(ref fid) = flow_id {
                    let duration = (Utc::now() - step_start).num_milliseconds();
                    self.flow_recorder.record_step(
                        fid.clone(),
                        FlowStepName::CredentialValidation,
                        StepStatus::Failure,
                        Some(duration),
                        Some(format!("{:?}", e)),
                        None,
                    );
                }
                e
            })?;

        if let Some(ref fid) = flow_id {
            let duration = (Utc::now() - step_start).num_milliseconds();
            self.flow_recorder.record_step(
                fid.clone(),
                FlowStepName::CredentialValidation,
                StepStatus::Success,
                Some(duration),
                None,
                None,
            );
        }

        self.determine_next_step(auth_result, params.session_code, auth_session)
            .await
    }

    async fn determine_next_step(
        &self,
        auth_result: AuthenticationResult,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
        let flow_id = auth_session.compass_flow_id.map(FlowId);

        if !auth_result.required_actions.is_empty() {
            return Ok(AuthenticateOutput::requires_actions(
                auth_result.user_id,
                auth_result.required_actions,
                auth_result.token.ok_or(CoreError::InternalServerError)?,
            ));
        }

        let has_otp_credentials = auth_result.credentials.iter().any(|cred| cred == "otp");
        let needs_configure_otp = auth_result
            .required_actions
            .contains(&RequiredAction::ConfigureOtp);

        if has_otp_credentials && !needs_configure_otp {
            if let Some(ref fid) = flow_id {
                self.flow_recorder.record_step(
                    fid.clone(),
                    FlowStepName::MfaChallenge,
                    StepStatus::Success,
                    None,
                    None,
                    None,
                );
            }
            let token = auth_result.token.ok_or(CoreError::InternalServerError)?;
            let email = self
                .user_repository
                .get_by_id(auth_result.user_id)
                .await
                .ok()
                .and_then(|user| user.email);
            return Ok(AuthenticateOutput::requires_otp_challenge(
                auth_result.user_id,
                token,
                email,
            ));
        }

        self.finalize_authentication(auth_result.user_id, session_code, auth_session)
            .await
    }

    async fn finalize_authentication(
        &self,
        user_id: Uuid,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
        let authorization_code = generate_random_string();

        self.auth_session_repository
            .update_code_and_user_id(session_code, authorization_code.clone(), user_id)
            .await
            .map_err(|e| {
                warn!(
                    "failed to update auth session with code and user id: {:?}",
                    e
                );
                CoreError::SessionNotFound
            })?;

        let completion = self.build_auth_completion(&auth_session, &authorization_code)?;

        Ok(AuthenticateOutput::complete(
            user_id,
            authorization_code,
            completion,
        ))
    }

    async fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> Result<AuthenticationResult, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(client_id.clone(), realm.id)
            .await
            .map_err(|e| {
                warn!("Client not found for client_id {}: {:?}", client_id, e);

                CoreError::InvalidClient
            })?;

        let realm_settings = self.realm_repository.get_realm_settings(realm.id).await?;
        let login_aliases = realm_settings
            .as_ref()
            .map(|s| s.login_aliases.clone())
            .unwrap_or_default();

        let user = crate::domain::authentication::login_resolver::resolve_user_by_identifier(
            self.user_repository.as_ref(),
            &username,
            realm.id,
            &login_aliases,
        )
        .await?
        .ok_or(CoreError::UserNotFound)?;

        if !user.enabled {
            return Err(CoreError::UserDisabled);
        }

        // Check maintenance mode
        if client.maintenance_enabled {
            let user_roles = self
                .user_role_repository
                .get_user_roles(user.id)
                .await
                .map_err(|_| CoreError::InternalServerError)?;
            let role_ids: Vec<Uuid> = user_roles.iter().map(|r| r.id).collect();

            let allowed_user_ids = self
                .maintenance_whitelist_repository
                .get_whitelisted_user_ids(client.id)
                .await?;
            let allowed_role_ids = self
                .maintenance_whitelist_repository
                .get_whitelisted_role_ids(client.id)
                .await?;
            let realm_allowed_user_ids = self
                .realm_maintenance_whitelist_repository
                .get_whitelisted_user_ids(realm.id)
                .await?;
            let realm_allowed_role_ids = self
                .realm_maintenance_whitelist_repository
                .get_whitelisted_role_ids(realm.id)
                .await?;

            let is_allowed = allowed_user_ids.contains(&user.id)
                || role_ids.iter().any(|r| allowed_role_ids.contains(r))
                || realm_allowed_user_ids.contains(&user.id)
                || role_ids.iter().any(|r| realm_allowed_role_ids.contains(r));

            if !is_allowed {
                let reason = client
                    .maintenance_reason
                    .unwrap_or_else(|| "This service is currently under maintenance".to_string());
                warn!(
                    "User {} denied access to client {} (maintenance mode)",
                    user.username, client.name
                );
                return Err(CoreError::ClientUnderMaintenance(reason));
            }
        }

        let realm_settings = self.realm_repository.get_realm_settings(realm.id).await?;
        let lockout_threshold = realm_settings
            .as_ref()
            .map(|s| s.lockout_threshold)
            .unwrap_or(10);
        let lockout_duration_seconds = realm_settings
            .as_ref()
            .map(|s| s.lockout_duration_seconds)
            .unwrap_or(900);

        let now = Utc::now();
        if user.is_locked(now) {
            return Err(CoreError::AccountLocked);
        }

        // Check if user has federation mapping (LDAP authentication) FIRST
        let federation_mapping = self
            .federation_repository
            .get_mapping_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        info!(
            "User {} (ID: {}): federation_mapping = {}",
            user.username,
            user.id,
            if federation_mapping.is_some() {
                "YES (LDAP user)"
            } else {
                "NO (local user)"
            }
        );

        let (has_valid_password, credentials, has_temporary_password) =
            if let Some(mapping) = federation_mapping {
                // User is federated - authenticate via LDAP
                info!(
                    "User {} is federated (provider_id: {}), authenticating via LDAP",
                    user.username, mapping.provider_id
                );

                let provider = self
                    .federation_repository
                    .get_by_id(mapping.provider_id)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?
                    .ok_or(CoreError::InternalServerError)?;

                if !provider.enabled {
                    error!("Federation provider {} is disabled", provider.name);
                    return Err(CoreError::InvalidPassword);
                }

                // Authenticate via LDAP
                let ldap_auth_result = match self
                    .ldap_client
                    .authenticate_user(&provider, &user.username, &password)
                    .await
                {
                    Ok(_) => {
                        info!("LDAP authentication successful for user {}", user.username);
                        true
                    }
                    Err(e) => {
                        error!(
                            "LDAP authentication failed for user {}: {}",
                            user.username, e
                        );
                        false
                    }
                };

                // Federated users don't have local credentials
                (ldap_auth_result, vec!["federated".to_string()], false)
            } else {
                // User is not federated - use local password hash
                info!(
                    "User {} is not federated, using local password hash",
                    user.username
                );

                let user_credentials = self
                    .credential_repository
                    .get_credentials_by_user_id(user.id)
                    .await
                    .map_err(|_| CoreError::GetUserCredentialsError)?;

                let has_temp_password = user_credentials.iter().any(|cred| cred.temporary);

                let creds: Vec<String> = user_credentials
                    .iter()
                    .map(|cred| cred.credential_type.clone().to_string())
                    .collect();

                let credential = self
                    .credential_repository
                    .get_password_credential(user.id)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;

                let salt = credential.salt.ok_or(CoreError::InternalServerError)?;

                let CredentialData::Hash {
                    hash_iterations,
                    algorithm,
                } = &credential.credential_data
                else {
                    tracing::error!(
                        "A password credential doesn't have Hash credential data.
This is a server error that should be investigated. Do not forward back this message to the client"
                    );
                    return Err(CoreError::InternalServerError);
                };

                let is_valid = self
                    .hasher_repository
                    .verify_password(
                        &password,
                        &credential.secret_data,
                        *hash_iterations,
                        algorithm,
                        &salt,
                    )
                    .await
                    .map_err(|_| CoreError::InvalidPassword)?;

                (is_valid, creds, has_temp_password)
            };

        if !has_valid_password {
            let locked_until = Self::compute_locked_until(
                user.failed_login_attempts + 1,
                lockout_threshold,
                lockout_duration_seconds,
                now,
            );
            let _ = self
                .user_repository
                .increment_failed_login_attempts(user.id, locked_until)
                .await;
            return Err(CoreError::InvalidPassword);
        }

        let _ = self
            .user_repository
            .reset_failed_login_attempts(user.id)
            .await;

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::SessionNotFound)?;

        let iss = format!("{}/realms/{}", base_url, realm.name);

        // A step token must not borrow the (possibly hours-long) access-token
        // lifetime — it only has to survive the next hop of the login flow.
        let temporary_lifetime = temporary_token_lifetime(realm_settings.as_ref());

        let auth_session_id = auth_session.id;
        let mut jwt_claim = JwtClaim::new(
            user.id,
            user.username.clone(),
            iss,
            vec![format!("{}-realm", realm.name), "account".to_string()],
            ClaimsTyp::Temporary,
            client_id.clone(),
            user.email.clone(),
            auth_session.scope,
            temporary_lifetime,
        );
        jwt_claim.additional_claims.insert(
            LOGIN_ACTION_SESSION_CLAIM.to_string(),
            serde_json::Value::String(auth_session_id.to_string()),
        );

        self.login_action_token_repository
            .create(LoginActionToken {
                jti: jwt_claim.jti,
                user_id: user.id,
                realm_id: realm.id.into(),
                auth_session_id,
                expires_at: Utc::now() + Duration::seconds(temporary_lifetime),
                consumed_at: None,
            })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        // Resolve MFA enforcement: realm-level or role-level require_mfa.
        let has_otp_credentials = credentials.iter().any(|cred| cred == "otp");
        let user_roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let mut effective_required_actions = user.required_actions.clone();

        let mfa_required_action = (!has_temporary_password
            && mfa_policy::user_requires_mfa(realm_settings.as_ref(), &user_roles))
        .then(|| {
            mfa_policy::required_action_for_mfa(has_otp_credentials)
                .filter(|a| !effective_required_actions.contains(a))
        })
        .flatten();

        if let Some(action) = mfa_required_action {
            effective_required_actions.push(action);
        }

        if !effective_required_actions.is_empty() || has_temporary_password {
            let jwt_token = self.generate_token(jwt_claim, realm.id).await?;

            let required_actions = if has_temporary_password {
                vec![RequiredAction::UpdatePassword]
            } else {
                effective_required_actions
            };

            return Ok(AuthenticationResult {
                code: None,
                required_actions,
                user_id: user.id,
                token: Some(jwt_token.token),
                credentials,
            });
        }

        if has_otp_credentials {
            let jwt_token = self.generate_token(jwt_claim, realm.id).await?;

            return Ok(AuthenticationResult {
                code: None,
                required_actions: Vec::new(),
                user_id: user.id,
                token: Some(jwt_token.token),
                credentials,
            });
        }

        Ok(AuthenticationResult {
            code: Some(generate_random_string()),
            required_actions: Vec::new(),
            user_id: user.id,
            token: None,
            credentials,
        })
    }

    async fn handle_token_refresh(
        &self,
        token: String,
        realm_id: RealmId,
        auth_session: AuthSession,
        session_code: Uuid,
    ) -> Result<AuthenticateOutput, CoreError> {
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let token_fingerprint = token_hash.chars().take(12).collect::<String>();
        let token_segments = token.split('.').count();

        let claims = self
            .verify_token(token.clone(), realm_id)
            .await
            .map_err(|e| {
                match &e {
                    CoreError::InvalidToken
                    | CoreError::ExpiredToken
                    | CoreError::TokenValidationError(_) => {
                        warn!(
                            token_fingerprint = %token_fingerprint,
                            token_segments = token_segments,
                            realm_id = %Uuid::from(realm_id),
                            session_code = %session_code,
                            error = ?e,
                            "Identity token cookie rejected, falling back to interactive login"
                        );
                    }
                    _ => {
                        error!("Failed to verify token: {:?}", e);
                    }
                }
                e
            })?;

        // FK-003: only a `Bearer` access token stands for a completed login.
        // Checked here — the earliest point at which `typ` is known, and before
        // any repository lookup — so a replayed `Temporary` step token cannot
        // skip the OTP challenge it was minted in front of.
        validate_token_refresh_request(&claims.typ, &auth_session, Utc::now()).inspect_err(
            |e| {
                warn!(
                    token_fingerprint = %token_fingerprint,
                    claims_typ = ?claims.typ,
                    realm_id = %Uuid::from(realm_id),
                    session_code = %session_code,
                    error = ?e,
                    "Rejected token-refresh authentication attempt"
                );
            },
        )?;

        let user = self
            .user_repository
            .get_by_id(claims.sub)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        // `ConfigureOtp` is never persisted (see `resolve_refresh_required_actions`),
        // so the MFA policy has to be re-evaluated here instead of trusting the
        // stored `user.required_actions` alone.
        let realm_settings = self.realm_repository.get_realm_settings(realm_id).await?;

        let user_roles = self
            .user_role_repository
            .get_user_roles(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let has_otp_credential = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?
            .iter()
            .any(|cred| cred.credential_type == CredentialType::Otp);

        let required_actions = resolve_refresh_required_actions(
            &user.required_actions,
            realm_settings.as_ref(),
            &user_roles,
            has_otp_credential,
        );

        if !required_actions.is_empty() {
            // Re-sign as an explicitly `Temporary` step token: re-signing the
            // incoming claims verbatim would carry the caller's `typ` through,
            // so the field named `temporary_token` could hand back a full
            // `Bearer` token.
            let temporary_claims = JwtClaim::new_temporary_token(
                claims,
                temporary_token_lifetime(realm_settings.as_ref()),
            );
            let jwt_token = self.generate_token(temporary_claims, realm_id).await?;

            return Ok(AuthenticateOutput::requires_actions(
                user.id,
                required_actions,
                jwt_token.token,
            ));
        }

        self.finalize_authentication(claims.sub, session_code, auth_session)
            .await
    }

    fn build_auth_completion(
        &self,
        auth_session: &AuthSession,
        authorization_code: &str,
    ) -> Result<AuthCompletion, CoreError> {
        format_auth_completion(auth_session, authorization_code)
    }

    fn claims_to_introspection_response(
        claims: JwtClaim,
        realm_name: String,
    ) -> TokenIntrospectionResponse {
        TokenIntrospectionResponse {
            active: true,
            scope: claims.scope,
            client_id: Some(claims.azp),
            username: claims.preferred_username,
            sub: Some(claims.sub.to_string()),
            token_type: Some(match claims.typ {
                ClaimsTyp::Bearer => "Bearer".to_string(),
                ClaimsTyp::Refresh => "Refresh".to_string(),
                ClaimsTyp::Temporary => "Temporary".to_string(),
                ClaimsTyp::Id => "ID".to_string(),
            }),
            exp: claims.exp,
            iat: Some(claims.iat),
            nbf: Some(claims.iat),
            aud: Some(claims.aud.join(" ")),
            iss: Some(claims.iss),
            jti: Some(claims.jti.to_string()),
            realm: Some(realm_name),
        }
    }

    fn verify_client_secret(stored: Option<&str>, provided: Option<&str>) -> bool {
        client_secret_matches(stored, provided)
    }

    async fn verify_id_token_hint(
        &self,
        id_token_hint: &str,
        realm_id: RealmId,
        expected_issuer: &str,
    ) -> Result<IdTokenClaims, CoreError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.validate_aud = false;

        let jwt_key_pair = self
            .keystore_repository
            .get_or_generate_key(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let token_data = jsonwebtoken::decode::<IdTokenClaims>(
            id_token_hint,
            &jwt_key_pair.decoding_key,
            &validation,
        )
        .map_err(|_| CoreError::InvalidToken)?;

        if token_data.claims.exp < Utc::now().timestamp() {
            return Err(CoreError::ExpiredToken);
        }

        if token_data.claims.iss != expected_issuer {
            return Err(CoreError::InvalidToken);
        }

        Ok(token_data.claims)
    }

    fn append_state_to_redirect_uri(redirect_uri: &str, state: Option<&str>) -> String {
        match state {
            Some(state) if !state.is_empty() => {
                let separator = if redirect_uri.contains('?') { '&' } else { '?' };
                format!(
                    "{redirect_uri}{separator}state={}",
                    urlencoding::encode(state)
                )
            }
            _ => redirect_uri.to_string(),
        }
    }
}

impl<
    R,
    C,
    RU,
    PLRU,
    U,
    UR,
    CR,
    H,
    AS,
    KS,
    RT,
    AT,
    F,
    CSM,
    PM,
    OM,
    OR,
    OAR,
    GT,
    URA,
    MW,
    RMW,
    UAR,
    EV,
    WR,
    SER,
    USR,
    LAT,
> AuthService
    for AuthServiceImpl<
        R,
        C,
        RU,
        PLRU,
        U,
        UR,
        CR,
        H,
        AS,
        KS,
        RT,
        AT,
        F,
        CSM,
        PM,
        OM,
        OR,
        OAR,
        GT,
        URA,
        MW,
        RMW,
        UAR,
        EV,
        WR,
        SER,
        USR,
        LAT,
    >
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    PLRU: PostLogoutRedirectUriRepository,
    U: UserRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    KS: KeyStoreRepository,
    RT: RefreshTokenRepository,
    AT: AccessTokenRepository,
    F: FederationRepository,
    CSM: ClientScopeMappingRepository,
    PM: ProtocolMapperRepository,
    OM: OrganizationMemberRepository,
    OR: OrganizationRepository,
    OAR: OrganizationAttributeRepository,
    GT: GroupTokenRepository,
    URA: UserRequiredActionRepository,
    MW: MaintenanceWhitelistRepository,
    RMW: RealmMaintenanceWhitelistRepository,
    UAR: UserAttributeRepository,
    EV: EmailVerificationService,
    WR: WebhookRepository,
    SER: SecurityEventRepository,
    USR: UserSessionRepository,
    LAT: LoginActionTokenRepository,
{
    async fn auth(&self, input: AuthInput) -> Result<AuthOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await?;

        let protocol = client.protocol.parse::<AuthProtocol>().map_err(|reason| {
            warn!(
                client_id = %input.client_id,
                %reason,
                "rejecting an authorization request for a client whose protocol is unknown"
            );

            CoreError::InvalidClient
        })?;

        if protocol != AuthProtocol::OpenIdConnect {
            warn!(
                client_id = %input.client_id,
                %protocol,
                "rejecting an authorization request: this endpoint only serves openid-connect clients"
            );

            return Err(CoreError::InvalidClient);
        }

        let redirect_uri = input.redirect_uri.clone();

        let client_redirect_uris = self
            .redirect_uri_repository
            .get_enabled_by_client_id(client.id)
            .await?;

        if !redirect_uri_matches_any(
            client_redirect_uris.iter().map(|uri| uri.value.as_str()),
            &redirect_uri,
        ) {
            return Err(CoreError::InvalidRedirectUri);
        }

        if !client.enabled {
            return Err(CoreError::InvalidClient);
        }

        // Enforce per-client PKCE policy (RFC 7636 §4.3).
        if client.require_pkce {
            if input.code_challenge.is_none() {
                return Err(CoreError::PkceRequired);
            }
            // Only S256 is accepted; an omitted method would default to `plain`.
            if !matches!(input.code_challenge_method, Some(CodeChallengeMethod::S256)) {
                return Err(CoreError::PkceRequired);
            }
        }

        let flow_id = self
            .flow_recorder
            .start_flow(
                realm.id,
                Some(input.client_id.clone()),
                "authorization_code".to_string(),
                None,
                None,
            )
            .await;

        let params = AuthSessionParams {
            realm_id: realm.id,
            client_id: client.id,
            protocol,
            redirect_uri,
            response_type: Some(input.response_type),
            scope: Some(input.scope.unwrap_or_default()),
            state: input.state.clone(),
            nonce: input.nonce,
            user_id: None,
            code: None,
            authenticated: false,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: Some(flow_id.0),
            code_challenge: input.code_challenge,
            code_challenge_method: input.code_challenge_method,
        };
        let session = self
            .auth_session_repository
            .create(&AuthSession::new(params))
            .await
            .map_err(|_| CoreError::SessionCreateError)?;

        self.flow_recorder.record_step(
            flow_id,
            FlowStepName::Authorize,
            StepStatus::Success,
            None,
            None,
            None,
        );

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
            .get_by_name(&realm_name)
            .await?
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

    #[instrument(
        skip(self, input),
        fields(
            realm_name = %input.realm_name,
            client_id = %input.client_id,
            grant_type = ?input.grant_type,
            has_client_secret = input.client_secret.is_some(),
            has_username = input.username.is_some(),
            has_password = input.password.is_some(),
            has_code = input.code.is_some(),
            has_refresh_token = input.refresh_token.is_some()
        )
    )]
    async fn exchange_token(&self, input: ExchangeTokenInput) -> Result<JwtToken, CoreError> {
        let exchange_start = Utc::now();
        let grant_type = input.grant_type.clone();
        let is_code_grant = grant_type == GrantType::Code;

        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .instrument(info_span!("auth.exchange_token.realm_lookup"))
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .instrument(info_span!("auth.exchange_token.client_lookup"))
            .await
            .map_err(|e| {
                warn!(
                    "Client not found for client_id {}: {:?}",
                    input.client_id, e
                );
                e
            })?;

        // For non-code grants, start a new compass flow (code grant uses existing flow from auth session)
        let is_refresh_grant = grant_type == GrantType::RefreshToken;
        let standalone_flow_id = if !is_code_grant && !is_refresh_grant {
            Some(
                self.flow_recorder
                    .start_flow(
                        realm.id,
                        Some(input.client_id.clone()),
                        grant_type.to_string(),
                        None,
                        None,
                    )
                    .await,
            )
        } else {
            None
        };

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
            redirect_uri: input.redirect_uri,
            scope: input.scope,
            code_verifier: input.code_verifier,
        };

        let result = self
            .authenticate_with_grant_type(grant_type, params)
            .instrument(info_span!(
                "auth.exchange_token.authenticate_with_grant_type"
            ))
            .await;

        if let Some(ref fid) = standalone_flow_id {
            let duration = (Utc::now() - exchange_start).num_milliseconds();
            match &result {
                Ok(_) => {
                    self.flow_recorder.record_step(
                        fid.clone(),
                        FlowStepName::TokenExchange,
                        StepStatus::Success,
                        Some(duration),
                        None,
                        None,
                    );
                    self.flow_recorder.complete_flow(
                        fid.clone(),
                        FlowStatus::Success,
                        duration,
                        None,
                    );
                }
                Err(error) => {
                    self.flow_recorder.record_step(
                        fid.clone(),
                        FlowStepName::TokenExchange,
                        StepStatus::Failure,
                        Some(duration),
                        Some(format!("{:?}", error)),
                        None,
                    );
                    self.flow_recorder.complete_flow(
                        fid.clone(),
                        FlowStatus::Failure,
                        duration,
                        None,
                    );
                }
            }
        }

        if let Err(error) = &result {
            warn!(
                error = ?error,
                "Token exchange failed"
            )
        }

        result
    }

    #[instrument(skip(self, input), fields(claims.sub = %input.claims.sub))]
    async fn authorize_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> Result<AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Bearer {
            return Err(CoreError::InvalidToken);
        }

        let user = self.user_repository.get_by_id(input.claims.sub).await?;

        self.verify_token(input.token, user.realm_id).await?;

        let identity: Identity = match input.claims.is_service_account() {
            true => {
                let client_id = input.claims.client_id.ok_or(CoreError::InvalidClient)?;
                let client_id = Uuid::parse_str(&client_id).map_err(|e| {
                    tracing::error!("failed to parse client id: {:?}", e);
                    CoreError::InvalidClient
                })?;

                let client = self
                    .client_repository
                    .get_by_id(user.realm_id, client_id)
                    .await?;

                Identity::Client(client)
            }
            false => Identity::User(user),
        };

        Ok(AuthorizeRequestOutput { identity })
    }

    async fn authorize_login_action_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> Result<AuthorizeRequestOutput, CoreError> {
        if input.claims.typ != ClaimsTyp::Temporary {
            return Err(CoreError::InvalidToken);
        }

        let user = self.user_repository.get_by_id(input.claims.sub).await?;

        if let Some(realm_name) = input.realm_name.as_deref() {
            let realm = self
                .realm_repository
                .get_by_name(realm_name)
                .await?
                .ok_or(CoreError::InvalidRealm)?;

            if realm.id != user.realm_id {
                return Err(CoreError::InvalidToken);
            }
        }

        let session_id = input
            .claims
            .additional_claims
            .get(LOGIN_ACTION_SESSION_CLAIM)
            .and_then(|value| value.as_str())
            .and_then(|raw| Uuid::parse_str(raw).ok())
            .ok_or(CoreError::InvalidToken)?;

        let auth_session = self
            .auth_session_repository
            .get_by_session_code(session_id)
            .await
            .map_err(|_| CoreError::InvalidToken)?;

        if auth_session.user_id.is_some_and(|owner| owner != user.id) {
            return Err(CoreError::InvalidToken);
        }

        if auth_session.expires_at <= Utc::now() {
            return Err(CoreError::InvalidToken);
        }

        let jti = input.claims.jti;
        let persisted = self
            .login_action_token_repository
            .get_by_jti(jti)
            .await
            .map_err(|_| CoreError::InvalidToken)?
            .ok_or(CoreError::InvalidToken)?;

        if persisted.consumed_at.is_some() || persisted.user_id != user.id {
            return Err(CoreError::InvalidToken);
        }

        self.verify_token(input.token, user.realm_id).await?;

        Ok(AuthorizeRequestOutput {
            identity: Identity::User(user),
        })
    }

    async fn authenticate(
        &self,
        input: super::entities::AuthenticateInput,
    ) -> Result<super::entities::AuthenticateOutput, CoreError> {
        let auth_session = self
            .auth_session_repository
            .get_by_session_code(input.session_code)
            .await
            .map_err(|e| {
                warn!("Failed to get auth session by session code: {:?}", e);
                CoreError::SessionNotFound
            })?;

        if auth_session.expires_at < Utc::now() {
            return Err(CoreError::SessionExpired);
        }

        if auth_session.user_id.is_some() && auth_session.authenticated {
            return Err(CoreError::InvalidSession);
        }

        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        match input.auth_method {
            AuthenticationMethod::ExistingToken { token } => {
                self.handle_token_refresh(token, realm.id, auth_session, input.session_code)
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

                self.handle_user_credentials_authentication(params, auth_session)
                    .await
                    .map_err(|_| CoreError::InvalidCredentials)
            }
        }
    }

    async fn register_user(
        &self,
        url_context: RegisterUserUrlContext,
        input: RegisterUserInput,
    ) -> Result<RegisterUserOutput, CoreError> {
        let RegisterUserUrlContext {
            issuer_base_url,
            verification_base_url,
        } = url_context;

        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let firstname = input.first_name;
        let lastname = input.last_name;

        let email = normalize_login_email(&input.email);
        let username = input.username.trim().to_string();

        if email.is_empty() || username.is_empty() {
            return Err(CoreError::InvalidRequest);
        }

        let email_verification_enabled = realm
            .settings
            .as_ref()
            .map(|s| s.email_verification_enabled)
            .unwrap_or(false);

        self.refuse_login_identifier_collision(&realm, &username, &email)
            .await?;

        let user = self
            .user_repository
            .create_user(CreateUserRequest {
                client_id: None,
                email: Some(email),
                email_verified: false,
                enabled: true,
                firstname,
                lastname,
                realm_id: realm.id,
                username,
            })
            .await?;

        // create user credentials
        let hash_result = self
            .hasher_repository
            .hash_password(&input.password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(user.id, "password".into(), hash_result, "".into(), false)
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        if email_verification_enabled {
            // Add verify_email required action
            self.user_required_action_repository
                .add_required_action(user.id, RequiredAction::VerifyEmail)
                .await
                .map_err(|e| {
                    tracing::error!(user_id = %user.id, error = %e, "Failed to add VerifyEmail required action");
                    CoreError::InternalServerError
                })?;

            if let Err(err) = self
                .email_verification_service
                .send_verification_email(user.id, input.realm_name, verification_base_url)
                .await
            {
                // Avoid leaving behind an unverified user that can no longer re-register.
                if let Err(cleanup_err) = self.user_repository.delete_user(user.id).await {
                    warn!(
                        user_id = %user.id,
                        error = %cleanup_err,
                        "Failed to roll back user after verification email delivery error"
                    );
                }

                return Err(err);
            }

            self.security_event_repository
                .store_event(
                    SecurityEvent::new(
                        realm.id,
                        SecurityEventType::UserCreated,
                        EventStatus::Success,
                        user.id,
                    )
                    .with_target("user".to_string(), user.id, None),
                )
                .await
                .map_err(|err| warn!("Failed to store UserCreated security event: {}", err))
                .ok();

            self.webhook_repository
                .notify(
                    realm.id,
                    WebhookPayload::new(
                        WebhookTrigger::UserCreated,
                        realm.id.into(),
                        Some(user.clone()),
                    ),
                )
                .await
                .map_err(|err| warn!("Failed to notify UserCreated webhook: {}", err))
                .ok();

            return Ok(RegisterUserOutput::PendingAction {
                message: "Please check your email to verify your account.".to_string(),
                user_id: user.id,
            });
        }

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::UserCreated,
                    EventStatus::Success,
                    user.id,
                )
                .with_target("user".to_string(), user.id, None),
            )
            .await
            .map_err(|err| warn!("Failed to store UserCreated security event: {}", err))
            .ok();

        self.webhook_repository
            .notify(
                realm.id,
                WebhookPayload::new(
                    WebhookTrigger::UserCreated,
                    realm.id.into(),
                    Some(user.clone()),
                ),
            )
            .await
            .map_err(|err| warn!("Failed to notify UserCreated webhook: {}", err))
            .ok();

        // If the registration happened inside an active OIDC authorization flow
        // (a FERRISKEY_SESSION cookie was present), resume that flow by
        // finalizing the auth session and returning the redirect URL back to
        // the original client, mirroring the behavior of the login handler.
        if let Some(session_code) = input.session_code
            && let Ok(auth_session) = self
                .auth_session_repository
                .get_by_session_code(session_code)
                .await
            && auth_session_can_resume(&auth_session, Utc::now())
        {
            let output = self
                .finalize_authentication(user.id, session_code, auth_session)
                .await?;
            let redirect_url = output.redirect_url.ok_or(CoreError::InternalServerError)?;
            return Ok(RegisterUserOutput::Redirect { url: redirect_url });
        }

        if let Some(step) = self.resolve_pending_auth_step(user.id, realm.id).await? {
            return Ok(RegisterUserOutput::PendingAction {
                message: pending_step_message(&step),
                user_id: user.id,
            });
        }

        let token = self
            .generate_tokens_for_user(GenerateTokensForUserInput {
                user_id: user.id,
                realm_id: realm.id.into(),
                base_url: issuer_base_url,
                client_id: None,
                scope: None,
            })
            .await?;

        Ok(RegisterUserOutput::Authenticated(token))
    }

    async fn get_userinfo(
        &self,
        identity: Identity,
        input: GetUserInfoInput,
    ) -> Result<UserInfoResponse, CoreError> {
        let user = self.user_repository.get_by_id(identity.id()).await?;

        let scopes = input
            .claims
            .scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect::<Vec<_>>())
            .unwrap_or_default();

        let contains_openid = scopes.contains(&"openid".to_string());
        if scopes.is_empty() || !contains_openid {
            return Err(CoreError::InvalidToken);
        }

        let mut response = UserInfoResponse {
            sub: user.id.to_string(),
            ..Default::default()
        };

        if scopes.contains(&"profile".to_string()) {
            response.name = Some(format!(
                "{} {}",
                user.firstname.as_deref().unwrap_or(""),
                user.lastname.as_deref().unwrap_or("")
            ));
            response.given_name = user.firstname.clone();
            response.family_name = user.lastname.clone();
            response.preferred_username = Some(user.username.clone());
        }

        if scopes.contains(&"email".to_string()) {
            response.email = user.email.clone();
            response.email_verified = Some(user.email_verified);
        }

        Ok(response)
    }

    async fn introspect_token(
        &self,
        input: IntrospectTokenInput,
    ) -> Result<TokenIntrospectionResponse, CoreError> {
        let inactive = TokenIntrospectionResponse {
            active: false,
            ..Default::default()
        };

        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if !client.enabled || client.public_client {
            return Err(CoreError::InvalidClient);
        }

        if !Self::verify_client_secret(client.secret_str(), Some(&input.client_secret)) {
            return Err(CoreError::InvalidClientSecret);
        }

        let token = input.token;
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

        // Opaque token support: prefer DB lookup by hash. This also enables immediate revocation.
        if let Some(stored) = self
            .access_token_repository
            .get_by_token_hash(token_hash.clone())
            .await
            .map_err(|_| CoreError::InternalServerError)?
        {
            if stored.revoked {
                return Ok(inactive);
            }

            if let Some(expires_at) = stored.expires_at
                && expires_at < Utc::now()
            {
                return Ok(inactive);
            }

            let claims: JwtClaim = serde_json::from_value(stored.claims)
                .map_err(|_| CoreError::InternalServerError)?;

            return Ok(Self::claims_to_introspection_response(claims, realm.name));
        }

        // Backward-compatible JWT introspection: validate signature + expiry even if not persisted.
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Ok(inactive);
        }

        let mut claims = match self.verify_token(token.clone(), realm.id).await {
            Ok(c) => c,
            Err(_) => return Ok(inactive),
        };

        // If the token is a refresh token (or hinted as such), enforce refresh token repository checks.
        if input.token_type_hint.as_deref() == Some("refresh_token")
            || claims.typ == ClaimsTyp::Refresh
        {
            claims = match self.verify_refresh_token(token, realm.id).await {
                Ok((c, _stored)) => c,
                Err(_) => return Ok(inactive),
            };
        }

        Ok(Self::claims_to_introspection_response(claims, realm.name))
    }

    async fn revoke_token(&self, input: RevokeTokenInput) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let hinted_refresh = input.token_type_hint.as_deref() == Some("refresh_token");
        let hinted_access = input.token_type_hint.as_deref() == Some("access_token");

        let claims = match self.verify_token(input.token.clone(), realm.id).await {
            Ok(claims) => claims,
            // RFC 7009 behavior: revocation is idempotent and should not reveal token validity.
            Err(CoreError::InvalidToken)
            | Err(CoreError::ExpiredToken)
            | Err(CoreError::TokenValidationError(_))
            | Err(CoreError::TokenParsingError(_)) => return Ok(()),
            Err(e) => return Err(e),
        };

        // Avoid cross-client revocation: only allow a client to revoke its own tokens.
        if claims.azp != input.client_id {
            return Ok(());
        }

        match claims.typ {
            ClaimsTyp::Refresh => {
                if hinted_access {
                    return Ok(());
                }

                self.refresh_token_repository
                    .revoke_by_jti(claims.jti)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;
            }
            ClaimsTyp::Bearer => {
                if hinted_refresh {
                    return Ok(());
                }

                let token_hash = format!("{:x}", Sha256::digest(input.token.as_bytes()));
                self.access_token_repository
                    .revoke_by_token_hash(token_hash)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;
            }
            ClaimsTyp::Temporary => {}
            ClaimsTyp::Id => {}
        }

        Ok(())
    }

    async fn end_session(&self, input: EndSessionInput) -> Result<EndSessionOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let id_token_claims = if let Some(id_token_hint) = input.id_token_hint.as_deref() {
            match self
                .verify_id_token_hint(id_token_hint, realm.id, &input.expected_issuer)
                .await
            {
                Ok(claims) => Some(claims),
                Err(e) if input.post_logout_redirect_uri.is_none() => {
                    // Keep logout robust for local-session logout when a stale/invalid id_token_hint is sent.
                    tracing::warn!(
                        "Ignoring invalid id_token_hint for non-redirect logout: {:?}",
                        e
                    );
                    None
                }
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        let token_client_id = id_token_claims
            .as_ref()
            .and_then(|claims| claims.azp.clone())
            .or_else(|| {
                id_token_claims.as_ref().and_then(|claims| {
                    if claims.aud.contains(' ') {
                        None
                    } else {
                        Some(claims.aud.clone())
                    }
                })
            });

        if let (Some(client_id), Some(token_client_id)) = (&input.client_id, &token_client_id)
            && client_id != token_client_id
        {
            warn!(
                "Logout rejected: client_id does not match id_token_hint (client_id={}, token_client_id={})",
                client_id, token_client_id
            );
            return Err(CoreError::InvalidRequest);
        }

        if let Some(claims) = id_token_claims.as_ref()
            && let Some(session_id) = claims
                .sid
                .as_deref()
                .and_then(|sid| Uuid::parse_str(sid).ok())
        {
            self.revoke_session_cascade(session_id, realm.id, claims.sub)
                .await?;
        }

        if let Some(post_logout_redirect_uri) = input.post_logout_redirect_uri {
            let resolved_client_id = input
                .client_id
                .or(token_client_id)
                .ok_or(CoreError::InvalidRequest)?;

            let client = self
                .client_repository
                .get_by_client_id(resolved_client_id, realm.id)
                .await?;

            let enabled_redirect_uris = self
                .post_logout_redirect_uri_repository
                .get_enabled_by_client_id(client.id)
                .await?;

            if !enabled_redirect_uris
                .iter()
                .any(|uri| uri.value == post_logout_redirect_uri)
            {
                warn!(
                    "Logout rejected: post_logout_redirect_uri is not registered for client (client_id={}, uri={}, registered_enabled_count={})",
                    client.client_id,
                    post_logout_redirect_uri,
                    enabled_redirect_uris.len()
                );
                return Err(CoreError::InvalidRedirectUri);
            }

            return Ok(EndSessionOutput {
                redirect_uri: Some(Self::append_state_to_redirect_uri(
                    &post_logout_redirect_uri,
                    input.state.as_deref(),
                )),
            });
        }

        Ok(EndSessionOutput { redirect_uri: None })
    }

    async fn generate_tokens_for_user(
        &self,
        input: GenerateTokensForUserInput,
    ) -> Result<JwtToken, CoreError> {
        let realm = self
            .realm_repository
            .get_by_id(input.realm_id.into())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let lifetimes = match input.client_id {
            Some(client_uuid) => self.resolve_token_lifetimes(realm.id, client_uuid).await?,
            None => {
                let realm_settings = self
                    .realm_repository
                    .get_realm_settings(realm.id)
                    .await?
                    .ok_or(CoreError::InvalidRealm)?;
                TokenLifetimes::from_realm(&realm_settings)
            }
        };

        let user = self.user_repository.get_by_id(input.user_id).await?;

        if !user.enabled {
            warn!(
                user_id = %user.id,
                "Refusing to mint tokens: the account is disabled"
            );
            return Err(CoreError::Forbidden("account is disabled".to_string()));
        }

        let pending_step = self.resolve_pending_auth_step(user.id, realm.id).await?;

        if let Err(error) = refuse_token_issuance_when_step_pending(pending_step.as_ref()) {
            warn!(
                user_id = %user.id,
                step = ?pending_step,
                "Refusing to mint tokens: the account still owes an authentication step"
            );
            return Err(error);
        }

        let user_session = self
            .create_user_session(user.id, realm.id, lifetimes.refresh_token)
            .await?;

        if let Some(client_uuid) = input.client_id {
            let client = self
                .client_repository
                .get_by_id(realm.id, client_uuid)
                .await?;
            let scope = self
                .resolve_scopes_for_client(client_uuid, input.scope.clone())
                .await?;

            let (jwt, refresh_token, id_token) = self
                .create_jwt(GenerateTokenInput {
                    base_url: input.base_url.clone(),
                    realm_name: realm.name.clone(),
                    user_id: user.id,
                    username: user.username.clone(),
                    firstname: user.firstname.clone().unwrap_or_default(),
                    lastname: user.lastname.clone().unwrap_or_default(),
                    email_verified: user.email_verified,
                    client_id: client.client_id,
                    client_uuid,
                    email: user.email.clone().unwrap_or_default(),
                    realm_id: realm.id,
                    scope: Some(scope),
                    access_token_lifetime: lifetimes.access_token,
                    refresh_token_lifetime: lifetimes.refresh_token,
                    id_token_lifetime: lifetimes.id_token,
                    nonce: None,
                    refresh_jti_override: None,
                    session_id: Some(user_session.id),
                })
                .await?;

            return Ok(JwtToken::new(
                jwt.token,
                "Bearer".to_string(),
                refresh_token.token,
                lifetimes.access_token as u32,
                lifetimes.refresh_token as u32,
                None,
                id_token.map(|token| token.token),
            ));
        }

        let azp = String::new();
        let scope = None;

        let iss = format!("{}/realms/{}", input.base_url, realm.name);
        let mut claims = JwtClaim::new(
            user.id,
            user.username.clone(),
            iss.clone(),
            vec![format!("{}-realm", realm.name), "account".to_string()],
            ClaimsTyp::Bearer,
            azp,
            user.email.clone(),
            scope.clone(),
            lifetimes.access_token,
        );
        claims.sid = Some(user_session.id);

        let jwt = self.generate_token(claims.clone(), realm.id).await?;

        let mut refresh_claims = JwtClaim::new_refresh_token(
            claims.sub,
            claims.iss.clone(),
            claims.aud.clone(),
            claims.azp.clone(),
            scope,
            lifetimes.refresh_token,
        );
        refresh_claims.sid = Some(user_session.id);

        let refresh_token = self
            .generate_token(refresh_claims.clone(), realm.id)
            .await?;

        let access_token_hash = format!("{:x}", Sha256::digest(jwt.token.as_bytes()));
        let access_token_claims =
            serde_json::to_value(&claims).map_err(|_| CoreError::InternalServerError)?;
        let access_token_expires_at = claims
            .exp
            .and_then(|exp| Utc.timestamp_opt(exp, 0).single());
        let refresh_token_expires_at = Utc
            .timestamp_opt(refresh_token.expires_at, 0)
            .single()
            .ok_or(CoreError::InternalServerError)?;

        tokio::try_join!(
            self.access_token_repository.create(
                access_token_hash,
                Some(claims.jti),
                claims.sub,
                realm.id,
                access_token_expires_at,
                access_token_claims,
            ),
            self.refresh_token_repository.create(
                refresh_claims.jti,
                user.id,
                Some(refresh_token_expires_at),
                Some(user_session.id),
            )
        )
        .map_err(|e| {
            warn!(
                user_id = %user.id,
                error = ?e,
                "Failed to persist tokens generated for a user"
            );
            CoreError::InternalServerError
        })?;

        Ok(JwtToken::new(
            jwt.token,
            "Bearer".to_string(),
            refresh_token.token,
            Self::expires_in_from(jwt.expires_at),
            Self::expires_in_from(refresh_token.expires_at),
            None,
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        auth_session_can_resume, format_auth_completion, format_authorization_redirect_url,
        lockout_compute_locked_until, validate_authorization_code_request,
    };
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::authentication::entities::{AuthCompletion, AuthProtocol, AuthSession};
    use crate::domain::client::entities::{Client, ClientType, MaintenanceSessionStrategy};
    use crate::domain::common::entities::app_errors::CoreError;
    use crate::domain::realm::entities::RealmId;

    /// Build an [`AuthSession`] with only the fields these helpers read, so the
    /// tests stay focused on redirect formatting / resume eligibility.
    fn auth_session(
        state: Option<&str>,
        redirect_uri: &str,
        expires_at: chrono::DateTime<Utc>,
        user_id: Option<Uuid>,
        authenticated: bool,
    ) -> AuthSession {
        AuthSession {
            id: Uuid::new_v4(),
            realm_id: RealmId::from(Uuid::new_v4()),
            client_id: Uuid::new_v4(),
            protocol: AuthProtocol::OpenIdConnect,
            redirect_uri: redirect_uri.to_string(),
            response_type: Some("code".to_string()),
            scope: Some("openid".to_string()),
            state: state.map(str::to_string),
            nonce: None,
            user_id,
            code: None,
            authenticated,
            created_at: Utc::now(),
            expires_at,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
            code_challenge: None,
            code_challenge_method: None,
        }
    }

    // ---- format_authorization_redirect_url -------------------------------

    #[test]
    fn redirect_url_includes_state_when_present() {
        let session = auth_session(
            Some("xyz-state"),
            "https://client.example/callback",
            Utc::now(),
            None,
            false,
        );

        assert_eq!(
            format_authorization_redirect_url(&session, "AUTH_CODE"),
            "https://client.example/callback?code=AUTH_CODE&state=xyz-state"
        );
    }

    #[test]
    fn redirect_url_omits_state_when_absent() {
        // RFC 6749 §4.1.2: `state` is echoed back only when the client supplied
        // it. A missing state must neither fail the flow nor emit `&state=`.
        let session = auth_session(
            None,
            "https://client.example/callback",
            Utc::now(),
            None,
            false,
        );

        assert_eq!(
            format_authorization_redirect_url(&session, "AUTH_CODE"),
            "https://client.example/callback?code=AUTH_CODE"
        );
    }

    #[test]
    fn redirect_url_treats_empty_state_as_absent() {
        let session = auth_session(
            Some(""),
            "https://client.example/callback",
            Utc::now(),
            None,
            false,
        );

        assert_eq!(
            format_authorization_redirect_url(&session, "AUTH_CODE"),
            "https://client.example/callback?code=AUTH_CODE"
        );
    }

    #[test]
    fn redirect_url_preserves_redirect_uri_verbatim() {
        let session = auth_session(
            Some("s"),
            "https://client.example/app/oidc",
            Utc::now(),
            None,
            false,
        );

        assert_eq!(
            format_authorization_redirect_url(&session, "C"),
            "https://client.example/app/oidc?code=C&state=s"
        );
    }

    #[test]
    fn redirect_url_appends_to_an_existing_query_string() {
        // A registered redirect URI may legitimately carry query parameters. Joining
        // with `?` unconditionally produced `...?tenant=acme?code=...`, which no
        // client can parse.
        let session = auth_session(
            Some("s"),
            "https://client.example/callback?tenant=acme",
            Utc::now(),
            None,
            false,
        );

        assert_eq!(
            format_authorization_redirect_url(&session, "C"),
            "https://client.example/callback?tenant=acme&code=C&state=s"
        );
    }

    #[test]
    fn redirect_url_percent_encodes_state() {
        // `state` is echoed back verbatim from the client, so it has to be encoded
        // or it can inject extra query parameters into the callback.
        let session = auth_session(
            Some("a b&next=https://evil.example"),
            "https://client.example/callback",
            Utc::now(),
            None,
            false,
        );

        assert_eq!(
            format_authorization_redirect_url(&session, "C"),
            "https://client.example/callback?code=C&state=a%20b%26next%3Dhttps%3A%2F%2Fevil.example"
        );
    }

    #[test]
    fn an_openid_connect_session_completes_with_the_untouched_redirect_url() {
        let session = auth_session(
            Some("xyz-state"),
            "https://client.example/callback",
            Utc::now(),
            None,
            false,
        );

        assert_eq!(
            format_auth_completion(&session, "AUTH_CODE").expect("openid-connect must complete"),
            AuthCompletion::Redirect {
                url: format_authorization_redirect_url(&session, "AUTH_CODE"),
            }
        );
    }

    #[test]
    fn a_saml_session_completes_on_the_continue_endpoint_that_will_issue_the_assertion() {
        let session = AuthSession {
            protocol: AuthProtocol::Saml,
            ..auth_session(
                Some("relay"),
                "https://auth.example.com/realms/master/protocol/saml/continue",
                Utc::now(),
                None,
                false,
            )
        };

        assert_eq!(
            format_auth_completion(&session, "AUTH_CODE").expect("a saml session must complete"),
            AuthCompletion::Redirect {
                url:
                    "https://auth.example.com/realms/master/protocol/saml/continue?code=AUTH_CODE&state=relay"
                        .to_string(),
            }
        );
    }

    #[test]
    fn a_saml_session_never_completes_on_an_address_the_service_provider_chose() {
        let session = AuthSession {
            protocol: AuthProtocol::Saml,
            ..auth_session(
                None,
                "https://auth.example.com/realms/master/protocol/saml/continue",
                Utc::now(),
                None,
                false,
            )
        };

        let completion =
            format_auth_completion(&session, "AUTH_CODE").expect("a saml session must complete");

        assert_eq!(
            completion.redirect_url(),
            Some("https://auth.example.com/realms/master/protocol/saml/continue?code=AUTH_CODE"),
            "the browser is sent back to us, never straight to the assertion consumer service"
        );
    }

    // ---- auth_session_can_resume -----------------------------------------

    #[test]
    fn resume_allowed_for_fresh_unconsumed_session() {
        let now = Utc::now();
        let session = auth_session(
            Some("s"),
            "https://c/cb",
            now + Duration::minutes(5),
            None,
            false,
        );

        assert!(auth_session_can_resume(&session, now));
    }

    #[test]
    fn resume_allowed_at_exact_expiry_boundary() {
        // `expires_at == now` is still live because the check is `>=`.
        let now = Utc::now();
        let session = auth_session(Some("s"), "https://c/cb", now, None, false);

        assert!(auth_session_can_resume(&session, now));
    }

    #[test]
    fn resume_rejected_for_expired_session() {
        let now = Utc::now();
        let session = auth_session(
            Some("s"),
            "https://c/cb",
            now - Duration::seconds(1),
            None,
            false,
        );

        assert!(!auth_session_can_resume(&session, now));
    }

    #[test]
    fn resume_rejected_for_spent_session() {
        // Already bound to a user *and* authenticated → an authorization code was
        // already minted for this request; replaying it is forbidden.
        let now = Utc::now();
        let session = auth_session(
            Some("s"),
            "https://c/cb",
            now + Duration::minutes(5),
            Some(Uuid::new_v4()),
            true,
        );

        assert!(!auth_session_can_resume(&session, now));
    }

    #[test]
    fn resume_allowed_when_user_bound_but_not_yet_authenticated() {
        // Mid-flow (user_id set, authenticated still false) is not yet spent.
        let now = Utc::now();
        let session = auth_session(
            Some("s"),
            "https://c/cb",
            now + Duration::minutes(5),
            Some(Uuid::new_v4()),
            false,
        );

        assert!(auth_session_can_resume(&session, now));
    }

    // ---- PKCE (RFC 7636) unit tests --------------------------------------

    use super::{pkce_validate_verifier, pkce_verify};
    use crate::domain::authentication::value_objects::CodeChallengeMethod;

    /// RFC 7636 Appendix B: known S256 test vector.
    ///
    /// verifier  = dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
    /// challenge = E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM
    #[test]
    fn pkce_s256_rfc7636_appendix_b_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        assert!(
            pkce_verify(verifier, challenge, &CodeChallengeMethod::S256),
            "RFC 7636 Appendix B S256 vector must verify"
        );
    }

    #[test]
    fn pkce_s256_wrong_verifier_rejected() {
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert!(
            !pkce_verify(
                "wrong-verifier-123456789012345678901234567890",
                challenge,
                &CodeChallengeMethod::S256
            ),
            "Wrong verifier must not match"
        );
    }

    #[test]
    fn pkce_plain_matches_when_equal() {
        let secret = "abcdefghijklmnopqrstuvwxyz0123456789ABCDE67";
        assert!(pkce_verify(secret, secret, &CodeChallengeMethod::Plain));
    }

    #[test]
    fn pkce_plain_rejects_when_different() {
        let verifier = "abcdefghijklmnopqrstuvwxyz0123456789ABCDE67";
        let challenge = "different-challenge-abcdefghijklmnopqrstuv";
        assert!(!pkce_verify(
            verifier,
            challenge,
            &CodeChallengeMethod::Plain
        ));
    }

    #[test]
    fn validate_code_verifier_accepts_valid() {
        let min_len = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(min_len.len(), 43);
        assert!(pkce_validate_verifier(min_len));
    }

    #[test]
    fn validate_code_verifier_rejects_too_short() {
        assert!(!pkce_validate_verifier("tooshort"));
    }

    #[test]
    fn validate_code_verifier_rejects_too_long() {
        let long = "a".repeat(129);
        assert!(!pkce_validate_verifier(&long));
    }

    #[test]
    fn validate_code_verifier_rejects_invalid_char() {
        // '+' is not in the unreserved set (RFC 7636 §4.1)
        assert!(!pkce_validate_verifier(
            "abcdefghijklmnopqrstuvwxyz0123456789ABCDE+7"
        ));
    }

    #[test]
    fn validate_code_verifier_accepts_all_unreserved_chars() {
        // All four special unreserved characters: - . _ ~
        // 43 chars: 26 uppercase + 4 special + 13 lowercase
        let verifier = "ABCDEFGHIJKLMNOPQRSTUVWXYZ-._~abcdefghijklm";
        assert_eq!(verifier.len(), 43);
        assert!(pkce_validate_verifier(verifier));
    }

    #[test]
    fn validate_code_verifier_accepts_128_chars() {
        let max_len = "a".repeat(128);
        assert!(pkce_validate_verifier(&max_len));
    }

    // ---- compute_locked_until -------------------------------------------

    #[test]
    fn compute_locked_until_returns_none_below_threshold() {
        let now = Utc::now();
        // 3 attempts with threshold 5 → not yet locked
        let result = lockout_compute_locked_until(3, 5, 900, now);
        assert!(result.is_none());
    }

    #[test]
    fn compute_locked_until_returns_some_at_threshold() {
        let now = Utc::now();
        let result = lockout_compute_locked_until(5, 5, 900, now);
        assert!(result.is_some());
        let locked = result.unwrap();
        let diff = (locked - now).num_seconds();
        assert_eq!(diff, 900);
    }

    #[test]
    fn compute_locked_until_returns_some_above_threshold() {
        let now = Utc::now();
        let result = lockout_compute_locked_until(10, 5, 300, now);
        assert!(result.is_some());
    }

    // ---- validate_authorization_code_request -----------------------------

    const REDIRECT_URI: &str = "https://client.example/callback";

    fn confidential_client(id: Uuid, realm_id: RealmId) -> Client {
        Client {
            id,
            enabled: true,
            client_id: "app".to_string(),
            secret: Some(maskass::Masked::new("s3cr3t".to_string())),
            realm_id,
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: false,
            direct_access_grants_enabled: false,
            oauth_device_code_grant_enabled: false,
            require_pkce: false,
            client_type: ClientType::Confidential,
            name: "app".to_string(),
            redirect_uris: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// A session and a client that agree on realm, client and redirect_uri, so
    /// each test can break exactly one binding.
    fn matching_pair() -> (AuthSession, Client) {
        let realm_id = RealmId::from(Uuid::new_v4());
        let client_uuid = Uuid::new_v4();

        let mut session = auth_session(
            Some("state"),
            REDIRECT_URI,
            Utc::now() + Duration::minutes(5),
            Some(Uuid::new_v4()),
            false,
        );
        session.realm_id = realm_id;
        session.client_id = client_uuid;

        (session, confidential_client(client_uuid, realm_id))
    }

    #[test]
    fn code_request_accepted_when_every_binding_matches() {
        let (session, client) = matching_pair();

        assert!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                Some("s3cr3t"),
                Utc::now(),
            )
            .is_ok()
        );
    }

    #[test]
    fn code_request_rejected_without_client_secret() {
        // The core of the CVE: a confidential client's code was redeemable with
        // no client authentication at all.
        let (session, client) = matching_pair();

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                None,
                Utc::now(),
            ),
            Err(CoreError::InvalidClientSecret)
        ));
    }

    #[test]
    fn code_request_rejected_with_wrong_client_secret() {
        let (session, client) = matching_pair();

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                Some("wrong"),
                Utc::now(),
            ),
            Err(CoreError::InvalidClientSecret)
        ));
    }

    #[test]
    fn public_client_needs_no_secret() {
        let (session, mut client) = matching_pair();
        client.public_client = true;
        client.secret = None;
        client.client_type = ClientType::Public;

        assert!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                None,
                Utc::now(),
            )
            .is_ok()
        );
    }

    #[test]
    fn code_request_rejected_for_a_different_client() {
        // Another client in the same realm presenting someone else's code.
        let (session, mut client) = matching_pair();
        client.id = Uuid::new_v4();

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                Some("s3cr3t"),
                Utc::now(),
            ),
            Err(CoreError::InvalidAuthorizationCode)
        ));
    }

    #[test]
    fn a_saml_authorization_code_is_not_redeemable_at_the_token_endpoint() {
        let (mut session, client) = matching_pair();
        session.protocol = AuthProtocol::Saml;

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                Some("s3cr3t"),
                Utc::now(),
            ),
            Err(CoreError::InvalidAuthorizationCode)
        ));
    }

    #[test]
    fn code_request_rejected_for_a_different_realm() {
        // Cross-tenant redemption: the code would come back signed with the
        // target realm's key.
        let (session, client) = matching_pair();
        let other_realm = RealmId::from(Uuid::new_v4());

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                other_realm,
                Some(REDIRECT_URI),
                Some("s3cr3t"),
                Utc::now(),
            ),
            Err(CoreError::InvalidAuthorizationCode)
        ));
    }

    #[test]
    fn code_request_rejected_on_redirect_uri_mismatch() {
        let (session, client) = matching_pair();

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some("https://attacker.example/callback"),
                Some("s3cr3t"),
                Utc::now(),
            ),
            Err(CoreError::InvalidAuthorizationCode)
        ));
    }

    #[test]
    fn code_request_rejected_when_redirect_uri_is_absent() {
        // RFC 6749 §4.1.3: the authorization request always carries a
        // redirect_uri here, so omitting it at the token endpoint is a mismatch.
        let (session, client) = matching_pair();

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                None,
                Some("s3cr3t"),
                Utc::now(),
            ),
            Err(CoreError::InvalidAuthorizationCode)
        ));
    }

    #[test]
    fn code_request_rejected_once_expired() {
        let (mut session, client) = matching_pair();
        session.expires_at = Utc::now() - Duration::seconds(1);

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                Some("s3cr3t"),
                Utc::now(),
            ),
            Err(CoreError::InvalidAuthorizationCode)
        ));
    }

    #[test]
    fn code_request_rejected_for_disabled_client() {
        let (session, mut client) = matching_pair();
        client.enabled = false;

        assert!(matches!(
            validate_authorization_code_request(
                &session,
                &client,
                session.realm_id,
                Some(REDIRECT_URI),
                Some("s3cr3t"),
                Utc::now(),
            ),
            Err(CoreError::InvalidClient)
        ));
    }

    // ---- FK-003: token-refresh guards ------------------------------------
    //
    // `POST /login-actions/authenticate` accepts an `Authorization: Bearer`
    // token and short-circuits the interactive login. The mid-flow `Temporary`
    // token minted by `using_session_code` (right before the OTP challenge)
    // must never be usable there: replaying it would complete authentication
    // with the password alone, skipping the second factor entirely.

    use super::{
        resolve_refresh_required_actions, temporary_token_lifetime, validate_token_refresh_request,
    };
    use crate::domain::authentication::entities::{AuthenticateOutput, AuthenticationStepStatus};
    use crate::domain::jwt::entities::ClaimsTyp;
    use crate::domain::realm::entities::RealmSetting;
    use crate::domain::role::entities::Role;
    use crate::domain::user::entities::RequiredAction;
    use ferriskey_security::jwt::entities::DEFAULT_TEMPORARY_TOKEN_LIFETIME;

    /// A session that is still live, so only the token `typ` can fail a test.
    fn live_session() -> AuthSession {
        auth_session(
            Some("s"),
            REDIRECT_URI,
            Utc::now() + Duration::minutes(5),
            None,
            false,
        )
    }

    fn realm_setting(require_mfa: bool) -> RealmSetting {
        let mut s = RealmSetting::new(RealmId::from(Uuid::new_v4()), None);
        s.require_mfa = require_mfa;
        s
    }

    fn role(require_mfa: bool) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "r".to_string(),
            description: None,
            permissions: vec![],
            realm_id: RealmId::from(Uuid::new_v4()),
            client_id: None,
            client: None,
            require_mfa,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn token_refresh_rejects_a_replayed_temporary_token() {
        // FK-003 path A: the step token handed to the client just before the
        // OTP challenge, replayed on /login-actions/authenticate.
        assert!(matches!(
            validate_token_refresh_request(&ClaimsTyp::Temporary, &live_session(), Utc::now()),
            Err(CoreError::InvalidToken)
        ));
    }

    #[test]
    fn token_refresh_rejects_refresh_and_id_tokens() {
        // Only a fully-minted access token stands for a completed login.
        for typ in [ClaimsTyp::Refresh, ClaimsTyp::Id] {
            assert!(
                matches!(
                    validate_token_refresh_request(&typ, &live_session(), Utc::now()),
                    Err(CoreError::InvalidToken)
                ),
                "{typ:?} must not short-circuit an interactive login"
            );
        }
    }

    #[test]
    fn token_refresh_accepts_a_bearer_token() {
        assert!(
            validate_token_refresh_request(&ClaimsTyp::Bearer, &live_session(), Utc::now()).is_ok()
        );
    }

    #[test]
    fn token_refresh_rejects_an_expired_auth_session() {
        // `authenticate` guards this, `handle_token_refresh` did not.
        let now = Utc::now();
        let session = auth_session(
            Some("s"),
            REDIRECT_URI,
            now - Duration::seconds(1),
            None,
            false,
        );

        assert!(matches!(
            validate_token_refresh_request(&ClaimsTyp::Bearer, &session, now),
            Err(CoreError::SessionExpired)
        ));
    }

    #[test]
    fn token_refresh_expired_session_beats_a_valid_bearer_token() {
        // An expired session is fatal regardless of how good the token is.
        let now = Utc::now();
        let session = auth_session(
            Some("s"),
            REDIRECT_URI,
            now - Duration::hours(1),
            None,
            false,
        );

        assert!(validate_token_refresh_request(&ClaimsTyp::Bearer, &session, now).is_err());
    }

    // ---- FK-003: MFA policy re-evaluated on the refresh path --------------

    #[test]
    fn refresh_injects_configure_otp_when_realm_requires_mfa() {
        // `ConfigureOtp` is computed, never persisted, so reading
        // `user.required_actions` alone lets mandatory enrolment be skipped.
        let actions = resolve_refresh_required_actions(&[], Some(&realm_setting(true)), &[], false);

        assert_eq!(actions, vec![RequiredAction::ConfigureOtp]);
    }

    #[test]
    fn refresh_injects_configure_otp_when_a_role_requires_mfa() {
        let actions = resolve_refresh_required_actions(
            &[],
            Some(&realm_setting(false)),
            &[role(true)],
            false,
        );

        assert_eq!(actions, vec![RequiredAction::ConfigureOtp]);
    }

    #[test]
    fn refresh_requires_actions_output_carries_no_authorization_code() {
        // The security property: an enrolment-pending user gets a step token,
        // never an authorization code.
        let actions = resolve_refresh_required_actions(&[], Some(&realm_setting(true)), &[], false);
        assert!(!actions.is_empty());

        let output =
            AuthenticateOutput::requires_actions(Uuid::new_v4(), actions, "step-token".to_string());

        assert_eq!(output.status, AuthenticationStepStatus::RequiresActions);
        assert!(output.authorization_code.is_none());
        assert!(output.redirect_url.is_none());
        assert_eq!(output.required_actions, vec![RequiredAction::ConfigureOtp]);
    }

    #[test]
    fn refresh_keeps_persisted_actions_when_mfa_is_not_enforced() {
        let actions = resolve_refresh_required_actions(
            &[RequiredAction::VerifyEmail],
            Some(&realm_setting(false)),
            &[role(false)],
            false,
        );

        assert_eq!(actions, vec![RequiredAction::VerifyEmail]);
    }

    #[test]
    fn refresh_does_not_duplicate_an_already_persisted_configure_otp() {
        let actions = resolve_refresh_required_actions(
            &[RequiredAction::ConfigureOtp],
            Some(&realm_setting(true)),
            &[],
            false,
        );

        assert_eq!(actions, vec![RequiredAction::ConfigureOtp]);
    }

    #[test]
    fn refresh_adds_no_action_when_the_user_already_enrolled_an_authenticator() {
        // Enrolment is done; the OTP-challenge gate owns the prompt from here.
        let actions = resolve_refresh_required_actions(&[], Some(&realm_setting(true)), &[], true);

        assert!(actions.is_empty());
    }

    #[test]
    fn refresh_enforces_mfa_even_without_realm_settings() {
        let actions = resolve_refresh_required_actions(&[], None, &[role(true)], false);

        assert_eq!(actions, vec![RequiredAction::ConfigureOtp]);
    }

    // ---- FK-003: step tokens are short-lived ------------------------------

    #[test]
    fn temporary_lifetime_uses_the_realm_temporary_setting_not_the_access_one() {
        let mut settings = realm_setting(false);
        settings.access_token_lifetime = 3600;
        settings.temporary_token_lifetime = 120;

        assert_eq!(temporary_token_lifetime(Some(&settings)), 120);
    }

    #[test]
    fn temporary_lifetime_falls_back_to_the_temporary_default() {
        assert_eq!(
            temporary_token_lifetime(None),
            DEFAULT_TEMPORARY_TOKEN_LIFETIME
        );
    }

    use super::validate_session_binding;
    use ferriskey_domain::session::entities::UserSession;

    fn user_session(expires_at: chrono::DateTime<Utc>) -> UserSession {
        UserSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            user_agent: None,
            ip_address: None,
            created_at: Utc::now(),
            expires_at,
            last_seen_at: None,
            soft_expiry_duration: None,
        }
    }

    #[test]
    fn token_rejected_when_its_session_was_revoked() {
        let now = Utc::now();

        assert!(
            matches!(
                validate_session_binding(Some(Uuid::new_v4()), None, now),
                Err(CoreError::SessionRevoked)
            ),
            "a token naming a session that no longer exists must not validate"
        );
    }

    #[test]
    fn token_rejected_when_its_session_expired() {
        let now = Utc::now();
        let session = user_session(now - Duration::seconds(1));

        assert!(
            matches!(
                validate_session_binding(Some(session.id), Some(&session), now),
                Err(CoreError::SessionRevoked)
            ),
            "a token naming an expired session must not validate"
        );
    }

    #[test]
    fn a_revoked_session_becomes_invalid_grant_on_the_token_endpoint() {
        assert!(
            matches!(
                super::revoked_session_is_an_invalid_grant(CoreError::SessionRevoked),
                CoreError::InvalidGrant(_)
            ),
            "refreshing against a revoked session must answer 400 invalid_grant"
        );
    }

    #[test]
    fn other_refresh_failures_keep_their_own_error() {
        assert!(
            matches!(
                super::revoked_session_is_an_invalid_grant(CoreError::ExpiredToken),
                CoreError::ExpiredToken
            ),
            "only the revoked-session case is rewritten"
        );
    }

    #[test]
    fn token_without_a_sid_is_still_accepted() {
        assert!(
            validate_session_binding(None, None, Utc::now()).is_ok(),
            "a token that never claimed a session must keep working"
        );
    }

    #[test]
    fn token_accepted_while_its_session_lives() {
        let now = Utc::now();
        let session = user_session(now + Duration::hours(1));

        assert!(
            validate_session_binding(Some(session.id), Some(&session), now).is_ok(),
            "the normal path must not regress"
        );
    }

    #[test]
    fn session_binding_holds_at_the_exact_expiry_boundary() {
        let now = Utc::now();
        let session = user_session(now);

        assert!(validate_session_binding(Some(session.id), Some(&session), now).is_ok());
    }

    use super::{
        refuse_token_issuance_when_actions_pending, refuse_token_issuance_when_step_pending,
    };
    use crate::domain::trident::mfa_policy::{PendingAuthStep, pending_auth_step};

    fn step(
        persisted: &[RequiredAction],
        require_mfa: bool,
        has_otp_credential: bool,
        has_temporary_password: bool,
    ) -> Option<PendingAuthStep> {
        let settings = realm_setting(require_mfa);
        pending_auth_step(
            persisted,
            Some(&settings),
            &[role(false)],
            has_otp_credential,
            has_temporary_password,
        )
    }

    #[test]
    fn an_admin_owing_nothing_still_gets_a_direct_grant() {
        let pending = step(&[], false, false, false);

        assert_eq!(pending, None);
        assert!(refuse_token_issuance_when_step_pending(pending.as_ref()).is_ok());
    }

    #[test]
    fn a_direct_grant_is_refused_for_a_temporary_password() {
        assert!(matches!(
            refuse_token_issuance_when_step_pending(step(&[], false, false, true).as_ref()),
            Err(CoreError::Forbidden(_))
        ));
    }

    #[test]
    fn a_direct_grant_is_refused_for_an_enrolled_authenticator() {
        assert!(matches!(
            refuse_token_issuance_when_step_pending(step(&[], false, true, false).as_ref()),
            Err(CoreError::Forbidden(_))
        ));
    }

    #[test]
    fn a_direct_grant_is_refused_when_the_realm_mandates_mfa_enrolment() {
        assert!(matches!(
            refuse_token_issuance_when_step_pending(step(&[], true, false, false).as_ref()),
            Err(CoreError::Forbidden(_))
        ));
    }

    #[test]
    fn a_direct_grant_is_refused_when_a_role_mandates_mfa_enrolment() {
        let pending = pending_auth_step(&[], None, &[role(true)], false, false);

        assert!(matches!(
            refuse_token_issuance_when_step_pending(pending.as_ref()),
            Err(CoreError::Forbidden(_))
        ));
    }

    #[test]
    fn a_direct_grant_is_refused_for_a_persisted_action() {
        assert!(matches!(
            refuse_token_issuance_when_step_pending(
                step(&[RequiredAction::VerifyEmail], false, false, false).as_ref()
            ),
            Err(CoreError::Forbidden(_))
        ));
    }

    #[test]
    fn a_refusal_names_the_step_and_sends_the_client_to_the_browser() {
        let Err(CoreError::Forbidden(message)) =
            refuse_token_issuance_when_step_pending(step(&[], false, false, true).as_ref())
        else {
            panic!("a temporary password must be refused");
        };

        assert!(
            message.contains("update_password"),
            "the client must learn which step is owed: {message}"
        );
        assert!(
            message.contains("authorization_code"),
            "the client must be told where to continue: {message}"
        );

        let Err(CoreError::Forbidden(otp_message)) =
            refuse_token_issuance_when_step_pending(step(&[], false, true, false).as_ref())
        else {
            panic!("an enrolled authenticator must be refused");
        };

        assert!(
            otp_message.contains("authorization_code"),
            "the client must be told where to continue: {otp_message}"
        );
    }

    #[test]
    fn auto_login_survives_a_reset_that_leaves_nothing_pending() {
        assert!(
            refuse_token_issuance_when_step_pending(step(&[], false, false, false).as_ref())
                .is_ok()
        );
    }

    #[test]
    fn auto_login_is_refused_when_a_second_factor_is_enrolled() {
        assert!(matches!(
            refuse_token_issuance_when_step_pending(step(&[], false, true, false).as_ref()),
            Err(CoreError::Forbidden(_))
        ));
    }

    #[test]
    fn a_code_grant_survives_an_otp_challenge() {
        assert!(
            refuse_token_issuance_when_actions_pending(step(&[], false, true, false).as_ref())
                .is_ok()
        );
    }

    #[test]
    fn a_code_grant_is_refused_while_actions_are_owed() {
        assert!(matches!(
            refuse_token_issuance_when_actions_pending(
                step(&[RequiredAction::VerifyEmail], false, false, false).as_ref()
            ),
            Err(CoreError::Forbidden(_))
        ));
    }

    #[test]
    fn a_code_grant_survives_an_empty_slate() {
        assert!(
            refuse_token_issuance_when_actions_pending(step(&[], false, false, false).as_ref())
                .is_ok()
        );
    }
}
