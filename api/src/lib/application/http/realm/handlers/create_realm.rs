use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;

use crate::application::auth::Identity;
use crate::application::http::realm::policies::RealmPolicy;
use crate::application::http::realm::validators::CreateRealmValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::realm::{entities::realm::Realm, ports::realm_service::RealmService};
use crate::domain::user::ports::user_service::UserService;

#[derive(TypedPath)]
#[typed_path("/realms")]
pub struct CreateRealmRoute;

#[utoipa::path(
    post,
    path = "",
    tag = "realm",
    responses(
        (status = 201, body = Realm)
    ),
    request_body = CreateRealmValidator
)]
pub async fn create_realm(
    _: CreateRealmRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRealmValidator>,
) -> Result<Response<Realm>, ApiError> {
    let c = RealmPolicy::create(identity.clone(), state.clone()).await?;

    if !c {
        return Err(ApiError::Forbidden(
            "You do not have permission to create a realm".into(),
        ));
    }

    let user = match identity {
        Identity::User(user) => user,
        Identity::Client(client) => {
            let service_account = state
                .user_service
                .get_by_client_id(client.id)
                .await
                .map_err(|_| ApiError::Forbidden("Service account not found".to_string()))?;
            service_account
        }
    };

    state
        .realm_service
        .create_realm_with_user(payload.name, &user)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{Router, http::StatusCode, middleware};
    use axum_extra::routing::RouterExt;
    use axum_test::TestServer;
    use clap::Parser;
    use serde_json::json;

    use crate::{
        application::{
            auth::auth,
            http::{realm::handlers::create_realm::create_realm, server::app_state::AppState},
            server::AppServer,
        },
        domain::{
            jwt::{
                entities::jwt_claim::{ClaimsTyp, JwtClaim},
                ports::jwt_service::JwtService,
            },
            user::entities::model::User,
            utils::generate_uuid_v7,
        },
        env::Env,
    };

    async fn create_test_app() -> (Router, AppState) {
        dotenv::dotenv().ok();

        let env = Arc::new(Env::parse());

        let app_server = AppServer::new(env.clone())
            .await
            .expect("Failed to create app server");

        let app_state = app_server.create_app_state(env.clone());

        let router = Router::new()
            .typed_post(create_realm)
            .layer(middleware::from_fn_with_state(app_state.clone(), auth))
            .with_state(app_state.clone());

        (router, app_state)
    }

    async fn create_test_user_and_token(state: &AppState) -> (User, String) {
        let test_user = User {
            id: generate_uuid_v7(),
            realm_id: generate_uuid_v7(),
            client_id: None,
            username: "test-user".to_string(),
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            enabled: true,
            roles: vec![], // Ajoutez les rôles nécessaires pour les permissions
            realm: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let claims = JwtClaim::new(
            test_user.id.clone(),
            test_user.username.clone(),
            "http://localhost:3333/realms/master".to_string(),
            vec!["master-realm".to_string(), "account".to_string()],
            ClaimsTyp::Bearer,
            "master".to_string(),
            Some(test_user.email.clone()),
        );

        let token = state.jwt_service.generate_token(claims).await.unwrap();

        (test_user, token.token)
    }

    #[tokio::test]
    async fn test_create_realm_success() {
        let (router, _) = create_test_app().await;

        let server = TestServer::new(router).unwrap();

        let response = server
            .post("/realms")
            .json(&json!({
                "name": "Test Realm"
            }))
            .await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }
}
