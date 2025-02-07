use axum::{routing::get, Router};
use rstest::fixture;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema};
use soil_api_rust::{
    areas, common, gnss, instrument_experiments, plots, projects, samples, sensors, soil, transects,
};

pub async fn setup_database() -> DatabaseConnection {
    // Use an in-memory SQLite database for testing.
    let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();

    let schema = Schema::new(db.get_database_backend());
    let entities = vec![
        schema
            .create_table_from_entity(areas::db::Entity)
            .to_owned(),
        schema.create_table_from_entity(gnss::db::Entity).to_owned(),
        schema
            .create_table_from_entity(instrument_experiments::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(instrument_experiments::channels::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(plots::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(plots::sensors::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(projects::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(samples::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(sensors::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(sensors::data::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(soil::profiles::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(soil::types::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(transects::db::Entity)
            .to_owned(),
        schema
            .create_table_from_entity(transects::nodes::db::Entity)
            .to_owned(),
    ];

    // Create all tables in the database
    for stmt in entities {
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();
    }

    db
}

#[fixture]
pub async fn mock_api() -> Router {
    // Use an in-memory SQLite database per test run
    let db = setup_database().await;

    // Build the router with routes similar to the main function
    Router::new()
        .route("/healthz", get(common::views::healthz))
        .with_state(db.clone())
        .nest("/v1/plots", plots::views::router(db.clone()))
        .nest("/v1/areas", areas::views::router(db.clone()))
        .nest("/v1/projects", projects::views::router(db.clone()))
        .nest("/v1/plot_samples", samples::views::router(db.clone()))
        // .nest("/v1/sensors", sensors::views::router(db.clone()))
        .nest("/v1/transects", transects::views::router(db.clone()))
        .nest("/v1/soil_types", soil::types::views::router(db.clone()))
        .nest(
            "/v1/soil_profiles",
            soil::profiles::views::router(db.clone()),
        )
}
