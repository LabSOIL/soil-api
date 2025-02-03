use axum::Router;
use sea_orm::{Database, DatabaseConnection};
use soil_api_rust::common::views::{get_ui_config, healthz};
use soil_api_rust::{
    areas,
    config,
    projects,
    transects,
    // plots,
    //   samples, sensors, soil, transects
};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Set up tracing/logging
    tracing_subscriber::fmt::init();
    println!("Starting server...");

    // Load configuration
    let cfg = config::Config::from_env();

    let db: DatabaseConnection = Database::connect(&*cfg.db_url.as_ref().unwrap())
        .await
        .unwrap();

    if db.ping().await.is_ok() {
        println!("Connected to the database");
    } else {
        println!("Could not connect to the database");
    }

    // Build the router with routes from the plots module
    let app = Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .route("/api/config", axum::routing::get(get_ui_config))
        .with_state(db.clone())
        // .nest("/v1/plots", plots::views::router(db.clone()))
        .nest("/api/areas", areas::views::router(db.clone()))
        .nest("/api/projects", projects::views::router(db.clone()))
        // .nest("/v1/plot_samples", samples::views::router(db.clone()))
        // .nest("/v1/sensors", sensors::views::router(db.clone()))
        .nest("/api/transects", transects::views::router(db.clone()))
        // .nest("/v1/soil_types", soil::types::views::router(db.clone()))
        // .nest(
        //     "/v1/soil_profiles",
        //     soil::profiles::views::router(db.clone()),
        // )
        // .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        // .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        ;

    // Bind to an address and serve the application
    let addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("Listening on {}", addr);

    // Run the server (correct axum usage without `hyper::Server`)
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
