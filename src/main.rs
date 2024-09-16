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

use axum::{routing::get, Router};
use sea_orm::{Database, DatabaseConnection};
use tracing_subscriber;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    // Set up tracing/logging
    tracing_subscriber::fmt::init();
    println!("Starting server...");

    #[derive(OpenApi)]
    #[openapi(
        paths(
            plots::views::get_all,
            areas::views::get_all,
            projects::views::get_all,
            common::views::healthz,
        ),
        components(schemas(
            plots::schemas::Plot,
            plots::schemas::PlotSimple,
            areas::schemas::Area,
            common::schemas::FilterOptions,
            projects::schemas::Project,
        ))
    )]
    struct ApiDoc;

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
        .route("/healthz", get(common::views::healthz))
        .nest("/v1/plots", plots::views::router(db.clone()))
        .nest("/v1/areas", areas::views::router(db.clone()))
        .nest("/v1/projects", projects::views::router(db.clone()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()));

    // Bind to an address and serve the application
    let addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("Listening on {}", addr);

    // Run the server (correct axum usage without `hyper::Server`)
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
