use axum::routing::Router;
use rstest::fixture;
use sea_orm::sea_query::TableCreateStatement;
use sea_orm::Database;
use sea_orm::{ConnectionTrait, DatabaseConnection, Schema};
use soil_api_rust::soil::types::db::Entity;
use soil_api_rust::soil::types::views::router;

pub async fn setup_database() -> DatabaseConnection {
    // Use an in-memory SQLite database for testing.
    let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();

    let schema = Schema::new(db.get_database_backend());
    let stmt: TableCreateStatement = schema.create_table_from_entity(Entity).to_owned();

    db.execute(db.get_database_backend().build(&stmt))
        .await
        .unwrap();

    db
}

#[fixture]
pub async fn mock_api() -> Router {
    // Use a unique in-memory SQLite database per test run
    let db = setup_database().await;
    router(db)
}
