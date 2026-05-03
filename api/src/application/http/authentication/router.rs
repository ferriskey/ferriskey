use axum::{
    Router, middleware,
    routing::{get, post},
};

use utoipa::OpenApi;

use super::handlers::{
    auth::{__path_auth_handler, auth_handler},
    authentificate::{__path_authenticate, authenticate},
    get_certs::{__path_get_certs, __path_get_jwks_json, get_certs, get_jwks_json},
    introspect::{__path_introspect_token, introspect_token},
    logout::{__path_logout_get, __path_logout_post, logout_get, logout_post},
    openid_configuration::{__path_get_openid_configuration, get_openid_configuration},
    registration::{__path_registration_handler, registration_handler},
    revoke::{__path_revoke_token, revoke_token},
    token::{__path_exchange_token, exchange_token},
    userinfo::{__path_get_userinfo, get_userinfo},
};
use crate::application::{auth::auth, http::server::app_state::AppState};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

#[derive(OpenApi)]
#[openapi(paths(
    exchange_token,
    introspect_token,
    authenticate,
    get_certs,
    get_jwks_json,
    auth_handler,
    logout_get,
    logout_post,
    revoke_token,
    get_openid_configuration,
    registration_handler,
    get_userinfo,
))]
pub struct AuthenticationApiDoc;

/// Build the authentication router with optional per-IP rate limiting on
/// sensitive endpoints (token, introspect, auth, authenticate).
///
/// **Proxy note:** The default key extractor uses the TCP peer IP
/// (`ConnectInfo<SocketAddr>`). Behind a reverse proxy all clients appear as
/// the proxy's IP, effectively creating a single global bucket. For
/// proxy-aware rate limiting, consider switching to `SmartIpKeyExtractor`
/// which reads `X-Forwarded-For` / `X-Real-Ip` headers.
pub fn authentication_routes(state: AppState, root_path: &str) -> Router<AppState> {
    let rate_limit_args = &state.args.rate_limit;

    // Non-rate-limited routes: userinfo (behind auth middleware), certs, jwks, well-known, logout, revoke, registrations
    let non_rate_limited = Router::new()
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/userinfo"),
            get(get_userinfo),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/userinfo"),
            post(get_userinfo),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/revoke"),
            post(revoke_token),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/logout"),
            get(logout_get),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/logout"),
            post(logout_post),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/registrations"),
            post(registration_handler),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/certs"),
            get(get_certs),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/jwks.json"),
            get(get_jwks_json),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/jwks"),
            get(get_jwks_json),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/.well-known/openid-configuration"),
            get(get_openid_configuration),
        );

    // Rate-limited routes: token, introspect, auth, authenticate
    let rate_limited = Router::new()
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/token"),
            post(exchange_token),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/token/introspect"),
            post(introspect_token),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/protocol/openid-connect/auth"),
            get(auth_handler),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/login-actions/authenticate"),
            post(authenticate),
        );

    // Apply governor layer only if rate limiting is enabled (per_minute > 0)
    let rate_limited = if rate_limit_args.auth_rate_limit_per_minute > 0 {
        let replenish_interval_ms =
            (60_000u64 / rate_limit_args.auth_rate_limit_per_minute).max(1);
        let config = GovernorConfigBuilder::default()
            .per_millisecond(replenish_interval_ms)
            .burst_size(rate_limit_args.auth_rate_limit_burst)
            .finish()
            .expect("invariant: replenish_interval_ms >= 1 and burst_size > 0");
        rate_limited.layer(GovernorLayer::new(config))
    } else {
        rate_limited
    };

    non_rate_limited.merge(rate_limited)
}
