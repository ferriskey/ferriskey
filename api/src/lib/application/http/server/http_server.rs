use crate::application::http::authentication::router::authentication_routes;
use crate::application::http::client::router::client_routes;
use crate::application::http::realm::router::realm_routes;
use crate::application::http::role::router::role_routes;
use crate::application::http::server::app_state::AppState;
use crate::application::http::server::openapi::ApiDoc;
use crate::application::http::user::router::user_routes;

use crate::domain::authentication::service::auth_session::DefaultAuthSessionService;
use crate::domain::authentication::service::authentication::DefaultAuthenticationService;
use crate::domain::client::services::client_service::DefaultClientService;
use crate::domain::client::services::redirect_uri_service::DefaultRedirectUriService;
use crate::domain::credential::services::credential_service::DefaultCredentialService;
use crate::domain::jwt::services::jwt_service::DefaultJwtService;
use crate::domain::realm::services::realm_service::DefaultRealmService;
use crate::domain::role::services::DefaultRoleService;
use crate::domain::user::services::user_service::DefaultUserService;
use anyhow::Context;
use axum::Router;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION};
use axum::http::{HeaderValue, Method};
use axum_cookie::prelude::*;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info_span;
use tracing::log::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct HttpServerConfig {
    port: String,
}

impl HttpServerConfig {
    pub fn new(port: String) -> Self {
        Self { port }
    }
}

pub struct HttpServer {
    router: Router,
    listener: tokio::net::TcpListener,
}

impl HttpServer {
    pub async fn new(
        config: HttpServerConfig,
        realm_service: Arc<DefaultRealmService>,
        client_service: Arc<DefaultClientService>,
        credential_service: Arc<DefaultCredentialService>,
        authentication_service: Arc<DefaultAuthenticationService>,
        auth_session_service: Arc<DefaultAuthSessionService>,
        user_service: Arc<DefaultUserService>,
        jwt_service: Arc<DefaultJwtService>,
        redirect_uri_service: DefaultRedirectUriService,
        role_service: DefaultRoleService,
    ) -> Result<Self, anyhow::Error> {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request| {
                let uri: String = request.uri().to_string();
                info_span!("http_request", method = ?request.method(), uri)
            },
        );

        let state = AppState::new(
            realm_service,
            client_service,
            credential_service,
            authentication_service,
            auth_session_service,
            user_service,
            jwt_service,
            redirect_uri_service,
            role_service,
        );

        let allowed_origins: Vec<HeaderValue> = vec![
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://localhost:4321"),
        ];

        let cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::DELETE,
                Method::PUT,
                Method::PATCH,
                Method::OPTIONS,
            ])
            .allow_origin(allowed_origins)
            .allow_headers([
                AUTHORIZATION,
                CONTENT_TYPE,
                CONTENT_LENGTH,
                ACCEPT,
                LOCATION,
            ])
            .allow_credentials(true);

        let router = axum::Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(realm_routes(state.clone()))
            .merge(client_routes())
            .merge(user_routes())
            .merge(authentication_routes())
            .merge(role_routes(state.clone()))
            .layer(trace_layer)
            .layer(cors)
            .layer(CookieLayer::default())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .with_context(|| format!("Failed to bind to port {}", config.port))?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        info!(
            "Server is running on http://{}",
            self.listener.local_addr()?
        );

        axum::serve(self.listener, self.router)
            .await
            .context("received error while running server")?;

        Ok(())
    }
}
