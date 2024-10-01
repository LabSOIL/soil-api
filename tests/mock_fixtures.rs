use axum::{routing::get, Router};
use rstest::fixture;
use sea_orm::Database;
use sea_orm::{ConnectionTrait, DatabaseConnection, Schema};
use soil_api_rust::areas::db::Entity as AreasEntity;
use soil_api_rust::gnss::db::Entity as GnssEntity;
use soil_api_rust::instrument_experiments::channels::db::Entity as InstrumentChannelsEntity;
use soil_api_rust::instrument_experiments::db::Entity as InstrumentExperimentsEntity;
use soil_api_rust::plots::db::Entity as PlotsEntity;
use soil_api_rust::plots::sensors::db::Entity as PlotSensorsEntity;
use soil_api_rust::projects::db::Entity as ProjectsEntity;
use soil_api_rust::samples::db::Entity as SamplesEntity;
use soil_api_rust::sensors::data::db::Entity as SensorDataEntity;
use soil_api_rust::sensors::db::Entity as SensorsEntity;
use soil_api_rust::soil::profiles::db::Entity as SoilProfilesEntity;
use soil_api_rust::soil::types::db::Entity as SoilTypesEntity;
use soil_api_rust::transects::db::Entity as TransectsEntity;
use soil_api_rust::transects::nodes::db::Entity as TransectNodesEntity;
use soil_api_rust::{areas, common, plots, projects, samples, sensors, soil, transects};

pub async fn setup_database() -> DatabaseConnection {
    // Use an in-memory SQLite database for testing.
    let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();

    let schema = Schema::new(db.get_database_backend());
    let entities = vec![
        schema.create_table_from_entity(AreasEntity).to_owned(),
        schema.create_table_from_entity(GnssEntity).to_owned(),
        schema
            .create_table_from_entity(InstrumentExperimentsEntity)
            .to_owned(),
        schema
            .create_table_from_entity(InstrumentChannelsEntity)
            .to_owned(),
        schema.create_table_from_entity(PlotsEntity).to_owned(),
        schema
            .create_table_from_entity(PlotSensorsEntity)
            .to_owned(),
        schema.create_table_from_entity(ProjectsEntity).to_owned(),
        schema.create_table_from_entity(SamplesEntity).to_owned(),
        schema.create_table_from_entity(SensorsEntity).to_owned(),
        schema.create_table_from_entity(SensorDataEntity).to_owned(),
        schema
            .create_table_from_entity(SoilProfilesEntity)
            .to_owned(),
        schema.create_table_from_entity(SoilTypesEntity).to_owned(),
        schema.create_table_from_entity(TransectsEntity).to_owned(),
        schema
            .create_table_from_entity(TransectNodesEntity)
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
        .nest("/v1/plots", plots::views::router(db.clone()))
        .nest("/v1/areas", areas::views::router(db.clone()))
        .nest("/v1/projects", projects::views::router(db.clone()))
        .nest("/v1/plot_samples", samples::views::router(db.clone()))
        .nest("/v1/sensors", sensors::views::router(db.clone()))
        .nest("/v1/transects", transects::views::router(db.clone()))
        .nest("/v1/soil_types", soil::types::views::router(db.clone()))
        .nest(
            "/v1/soil_profiles",
            soil::profiles::views::router(db.clone()),
        )
}
