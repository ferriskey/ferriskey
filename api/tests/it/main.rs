mod docker;
mod postgres_context;

use std::sync::Arc;

use axum_test::TestServer;
use postgres_context::DBContext;
use test_context::test_context;

use ferriskey_api::{
    application::http::server::http_server::{router, state},
    args::{Args, DatabaseArgs},
};

use sea_orm::Database;
pub use sea_orm::DatabaseConnection;
use sea_orm::{ConnectionTrait, Statement};
use url::Url;

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
    let db_url = Url::parse(ctx.url()).unwrap();
    let env = Args {
        db: DatabaseArgs::from(db_url),
        ..Default::default()
    };
    let (state, _) = state(Arc::new(env)).await.unwrap();

    let app = router(state).unwrap();

    let server = TestServer::new(app).unwrap();

    let r = server.get("/something-non-existent").await;
    assert_eq!(r.status_code(), 404); // we can start the server and it returns 404, yey
    Ok(())
}
