use std::sync::Arc;

use crate::application::http::server::app_state::AppState;
use crate::application::http::server::openapi::ApiDoc;
use crate::args::Args;
use ferriskey_api_abyss::routes::abyss_routes;
use ferriskey_api_aegis::router::aegis_routes;
use ferriskey_api_authentication::router::authentication_routes;
use ferriskey_api_broker::router::broker_routes;
use ferriskey_api_client::router::client_routes;
use ferriskey_api_compass::router::compass_routes;
use ferriskey_api_email_template::router::email_template_routes;
use ferriskey_api_maintenance::router::maintenance_routes;
use ferriskey_api_organization::router::organization_routes;
use ferriskey_api_portal_layouts::router::portal_layouts_routes;
use ferriskey_api_portal_theme::router::portal_theme_routes;
use ferriskey_api_realm::router::realm_routes;
use ferriskey_api_role::router::role_routes;
use ferriskey_api_seawatch::router::seawatch_router;
use ferriskey_api_trident::router::trident_routes;
use ferriskey_api_user::router::user_routes;
use ferriskey_api_webhook::router::webhook_routes;

use super::config::get_config;
use anyhow::Context;
use axum::Router;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION};
use axum::http::{HeaderValue, Method};
use axum::routing::get;
use axum_cookie::prelude::*;
use axum_prometheus::PrometheusMetricLayer;
use ferriskey_api_health::health_routes;
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
            let method = request.method().as_str();
            let path = request
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .map(|p| p.as_str())
                .unwrap_or(request.uri().path());
            let otel_name = format!("{method} {path}");
            let uri: String = request.uri().to_string();
            info_span!("http_request", method, uri, otel.name = %otel_name)
        },
    );

    let allowed_origins = state
        .args
        .server
        .allowed_origins
        .iter()
        .map(|origin| {
            HeaderValue::from_str(origin)
                .with_context(|| format!("Invalid origin in configuration: '{}'", origin))
        })
        .collect::<anyhow::Result<Vec<HeaderValue>>>()?;

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
        .merge(authentication_routes(state.clone(), &root_path))
        .merge(role_routes(state.clone()))
        .merge(webhook_routes(state.clone()))
        .merge(maintenance_routes(state.clone()))
        .merge(email_template_routes(state.clone()))
        .merge(portal_theme_routes(state.clone()))
        .merge(portal_layouts_routes(state.clone()))
        .merge(trident_routes(state.clone()))
        .merge(seawatch_router(state.clone()))
        .merge(compass_routes(state.clone()))
        .merge(abyss_routes(state.clone()))
        .merge(aegis_routes(state.clone()))
        .merge(broker_routes(state.clone(), &root_path))
        .merge(organization_routes(state.clone()))
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
