mod docker;
mod postgres_context;

use std::sync::Arc;

use axum_test::TestServer;
use postgres_context::DBContext;
use test_context::test_context;

use ferriskey_api::{
    application::http::server::http_server::{router, state},
    env::Env,
};

use sea_orm::Database;
pub use sea_orm::DatabaseConnection;
use sea_orm::{ConnectionTrait, Statement};

fn mock_env(database_url: &str) -> Env {
    Env {
        port: "80".to_string(),
        database_url: database_url.to_string(),
        portal_url: "http://localhost:80".to_string(),
        allowed_origins: "AllowOrigin::any()".to_string(),
        admin_password: "password".to_string(),
        admin_username: "admin".to_string(),
        admin_email: "password".to_string(),
        ..Default::default()
    }
}

/// Test that we can interact with the database somehow.
#[test_context(DBContext)]
#[tokio::test]
async fn test_db_client(ctx: &mut DBContext) -> Result<(), String> {
    let db = Database::connect(ctx.url()).await.unwrap();

    let stmt = Statement::from_string(
        db.get_database_backend(),
        "SELECT 'hello world' as value".to_owned(),
    );
    let row = db.query_one(stmt).await.unwrap().unwrap();
    let result: String = row.try_get("", "value").unwrap();

    assert_eq!(result, "hello world");
    Ok(())
}

/// Test that we can start the server with the database, and return 404
#[test_context(DBContext)]
#[tokio::test]
async fn test_404(ctx: &mut DBContext) -> Result<(), String> {
    let env = mock_env(ctx.url());
    let state = state(Arc::new(env)).await.unwrap();

    let app = router(state).unwrap();

    let server = TestServer::new(app).unwrap();

    let r = server.get("/something-non-existent").await;
    assert_eq!(r.status_code(), 404); // we can start the server and it returns 404, yey
    Ok(())
}
