use crate::application::http::auth::router::auth_router;
use crate::application::http::authentication::router::authentication_routes;
use crate::application::http::client::router::client_routes;
use crate::application::http::realm::router::realm_routes;
use crate::application::http::server::app_state::AppState;
use crate::application::http::server::openapi::ApiDoc;
use crate::application::http::user::router::user_routes;
use crate::domain::authentication::ports::AuthenticationService;
use crate::domain::client::ports::ClientService;
use crate::domain::credential::ports::CredentialService;
use crate::domain::realm::ports::RealmService;
use anyhow::Context;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::{Extension, Router};
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
    pub async fn new<R, C, CR, A>(
        config: HttpServerConfig,
        realm_service: Arc<R>,
        client_service: Arc<C>,
        credential_service: Arc<CR>,
        authentication_service: Arc<A>,
    ) -> Result<Self, anyhow::Error>
    where
        R: RealmService,
        C: ClientService,
        CR: CredentialService,
        A: AuthenticationService,
    {
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
        );

        let allowed_origins: Vec<HeaderValue> = vec![
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("http://localhost:5173"),
        ];

        let cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::DELETE,
                Method::PUT,
                Method::PATCH,
            ])
            .allow_origin(allowed_origins)
            .allow_headers([AUTHORIZATION, CONTENT_TYPE, CONTENT_LENGTH, ACCEPT])
            .allow_credentials(true);

        let router = axum::Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(realm_routes::<R>())
            .merge(client_routes::<C>())
            .merge(user_routes::<CR>())
            .merge(auth_router::<R, C>())
            .merge(authentication_routes::<A>())
            .layer(trace_layer)
            .layer(cors)
            .layer(Extension(Arc::clone(&state.realm_service)))
            .layer(Extension(Arc::clone(&state.client_service)))
            .layer(Extension(Arc::clone(&state.authentication_service)))
            .layer(Extension(Arc::clone(&state.credential_service)));

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
