mod areas;
mod common;
mod config;
mod gnss;
mod instrument_experiments;
mod plots;
mod projects;
mod samples;
mod sensors;
mod soil;
mod transects;

use crate::common::views::{get_ui_config, healthz};
use axum::extract::DefaultBodyLimit;
use axum::Router;
use axum_keycloak_auth::{instance::KeycloakAuthInstance, instance::KeycloakConfig, Url};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Set up tracing/logging
    tracing_subscriber::fmt::init();
    println!("Starting server...");

    // Load configuration
    let config = config::Config::from_env();

    let db: DatabaseConnection = Database::connect(&*config.db_url.as_ref().unwrap())
        .await
        .unwrap();

    if db.ping().await.is_ok() {
        println!("Connected to the database");
    } else {
        println!("Could not connect to the database");
    }

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    // Left commented here in case of need to downgrade
    // Migrator::down(&db, Some(1))  // Downgrade one migration step
    //     .await
    //     .expect("Failed to run downgrade migration");

    println!("DB migrations complete");

    let keycloak_auth_instance: Arc<KeycloakAuthInstance> = Arc::new(KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse(&config.keycloak_url).unwrap())
            .realm(String::from(&config.keycloak_realm))
            .build(),
    ));

    println!(
        "Starting server {} ({} deployment) ...",
        config.app_name,
        config.deployment.to_uppercase()
    );

    // Build the router with routes from the plots module
    let app = Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .route("/api/config", axum::routing::get(get_ui_config))
        .with_state(db.clone())
        .nest(
            "/api/plots",
            plots::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/areas",
            areas::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/projects",
            projects::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/gnss",
            gnss::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/plot_samples",
            samples::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/sensors",
            sensors::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/sensor_profiles",
            sensors::profile::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/sensor_profile_assignments",
            sensors::profile::assignment::views::router(
                db.clone(),
                Some(keycloak_auth_instance.clone()),
            ),
        )
        .nest(
            "/api/transects",
            transects::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/instruments",
            instrument_experiments::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/instrument_channels",
            instrument_experiments::channels::views::router(
                db.clone(),
                Some(keycloak_auth_instance.clone()),
            ),
        )
        .nest(
            "/api/soil_types",
            soil::types::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/soil_profiles",
            soil::profiles::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .layer(DefaultBodyLimit::max(30 * 1024 * 1024));
    // .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
    // .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
    // .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))

    // Bind to an address and serve the application
    let addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("Listening on {}", addr);

    // Run the server (correct axum usage without `hyper::Server`)
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
