use crate::application::http::client::router::client_routes;
use crate::application::http::realm::router::realm_routes;
use crate::application::http::server::app_state::AppState;
use crate::application::http::server::openapi::ApiDoc;
use crate::domain::client::ports::ClientService;
use crate::domain::realm::ports::RealmService;
use anyhow::Context;
use axum::{Extension, Router};
use std::sync::Arc;
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
    pub async fn new<R, C>(
        config: HttpServerConfig,
        realm_service: Arc<R>,
        client_service: Arc<C>,
    ) -> Result<Self, anyhow::Error>
    where
        R: RealmService,
        C: ClientService,
    {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request| {
                let uri: String = request.uri().to_string();
                info_span!("http_request", method = ?request.method(), uri)
            },
        );

        let state = AppState::new(realm_service, client_service);

        let router = axum::Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(realm_routes::<R>())
            .merge(client_routes::<C>())
            .layer(trace_layer)
            .layer(Extension(Arc::clone(&state.realm_service)))
            .layer(Extension(Arc::clone(&state.client_service)));

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
