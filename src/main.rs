mod areas;
mod config;
mod gnss;
mod instrument_experiments;
mod plots;
mod projects;
mod samples;
mod sensors;
mod soil;
mod transects;

use crate::plots::models::Gradientchoices;
use crate::plots::schemas::{Area, FilterOptions, Plot, PlotWithCoords};
use axum::{routing::get, Router};
use sea_orm::{Database, DatabaseConnection};
use tracing_subscriber;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

/// Get health of the API.
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(
        paths(crate::plots::views::get_all, health),
        components(schemas(Plot, Area, Gradientchoices, FilterOptions, PlotWithCoords))
    )]
    struct ApiDoc;

    // Set up tracing/logging
    tracing_subscriber::fmt::init();
    println!("Starting server...");

    // Load configuration
    let cfg = config::Config::from_env();

    let db: DatabaseConnection = Database::connect(&*cfg.db_url.as_ref().unwrap())
        .await
        .expect("Could not connect to the database");

    if db.ping().await.is_ok() {
        println!("Connected to the database");
    } else {
        println!("Could not connect to the database");
    }

    // Build the router with routes from the plots module
    let app = Router::new()
        .route("/healthz", get(health))
        .nest("/v1/plots", plots::views::router(db.clone()))
        .nest("/v1/areas", areas::views::router(db.clone()))
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
