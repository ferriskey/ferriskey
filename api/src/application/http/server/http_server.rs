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
use ferriskey_core::application::create_service;
use ferriskey_core::domain::common::FerriskeyConfig;
use tower_http::cors::CorsLayer;
use tracing::{debug, info_span};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

pub async fn state(args: Arc<Args>) -> Result<AppState, anyhow::Error> {
    let ferriskey_config: FerriskeyConfig = FerriskeyConfig::from(args.as_ref().clone());
    let service = create_service(ferriskey_config).await?;

    Ok(AppState::new(args, service))
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

    debug!("Allowed origins: {:?}", allowed_origins);

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

    let root_path = state.args.server.root_path.clone();
    let api_docs_url = format!("{}/api-docs/openapi.json", root_path);

    let router = axum::Router::new()
        .merge(Scalar::with_url(
            format!("{}/scalar", root_path),
            openapi.clone(),
        ))
        .merge(
            SwaggerUi::new(format!("{}/swagger-ui", root_path))
                .url(api_docs_url.clone(), openapi.clone()),
        )
        .merge(Redoc::with_url(format!("{}/redoc", root_path), openapi))
        .merge(RapiDoc::new(api_docs_url).path(format!("{}/rapidoc", root_path)))
        .route(&format!("{}/config", root_path), get(get_config))
        .merge(realm_routes(state.clone()))
        .merge(client_routes(state.clone()))
        .merge(user_routes(state.clone()))
        .merge(authentication_routes(&root_path))
        .merge(role_routes(state.clone()))
        .merge(webhook_routes(state.clone()))
        .merge(trident_routes(state.clone()))
        .merge(health_routes(&root_path))
        .route(
            &format!("{}/metrics", root_path),
            get(|| async move { metric_handle.render() }),
        )
        .layer(trace_layer)
        .layer(cors)
        .layer(CookieLayer::default())
        .layer(prometheus_layer)
        .with_state(state);
    Ok(router)
}
