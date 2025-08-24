use std::sync::Arc;

use crate::application::http::authentication::router::authentication_routes;
use crate::application::http::client::router::client_routes;
use crate::application::http::realm::router::realm_routes;
use crate::application::http::role::router::role_routes;
use crate::application::http::server::app_state::AppState;
use crate::application::http::server::openapi::ApiDoc;
use crate::application::http::trident::router::trident_routes;
use crate::application::http::user::router::user_routes;
use crate::application::http::webhook::router::webhook_routes;
use crate::args::Args;

use super::config::get_config;
use crate::application::http::health::health_routes;
use axum::Router;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION};
use axum::http::{HeaderValue, Method};
use axum::routing::get;
use axum_cookie::prelude::*;
use axum_prometheus::PrometheusMetricLayer;
use tower_http::cors::CorsLayer;
use tracing::info_span;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use ferriskey_core::application::common::{
    factories::UseCaseBundle,
    services::{ServiceFactory, ServiceFactoryConfig},
};

pub async fn state(args: Arc<Args>) -> Result<AppState, anyhow::Error> {
    let service_bundle = ServiceFactory::create_all_services(ServiceFactoryConfig {
        database_url: format!(
            "postgresql://{}:{}@{}:{}/{}",
            args.db.user, args.db.password, args.db.host, args.db.port, args.db.name
        ),
    })
    .await?;

    let use_case = UseCaseBundle::new(&service_bundle);

    Ok(AppState::new(args, service_bundle, use_case))
}

///  Returns the [`Router`] of this application.
pub fn router(state: AppState) -> Result<Router, anyhow::Error> {
    let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
        |request: &axum::extract::Request| {
            let uri: String = request.uri().to_string();
            info_span!("http_request", method = ?request.method(), uri)
        },
    );

    let allowed_origins = state
        .args
        .server
        .allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin).unwrap())
        .collect::<Vec<HeaderValue>>();

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

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let mut openapi = ApiDoc::openapi();
    let mut paths = openapi.paths.clone();
    paths.paths = openapi
        .paths
        .paths
        .into_iter()
        .map(|(path, item)| (format!("{}{path}", state.args.server.root_path), item))
        .collect();
    openapi.paths = paths;

    let router = axum::Router::new()
        .merge(Scalar::with_url(
            format!("{}/swagger-ui", state.args.server.root_path),
            openapi,
        ))
        .route(
            &format!("{}/config", state.args.server.root_path),
            get(get_config),
        )
        .merge(realm_routes(state.clone()))
        .merge(client_routes(state.clone()))
        .merge(user_routes(state.clone()))
        .merge(authentication_routes(&state.args.server.root_path))
        .merge(role_routes(state.clone()))
        .merge(webhook_routes(state.clone()))
        .merge(trident_routes(state.clone()))
        .merge(health_routes(&state.args.server.root_path))
        .route(
            &format!("{}/metrics", state.args.server.root_path),
            get(|| async move { metric_handle.render() }),
        )
        .layer(trace_layer)
        .layer(cors)
        .layer(CookieLayer::default())
        .layer(prometheus_layer)
        .with_state(state);
    Ok(router)
}
