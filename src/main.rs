use axum::{
    extract::State,
    routing::get,
    response::Json,
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool, FromRow};
use serde::Serialize;
use uuid::Uuid;
use chrono::{NaiveDate, NaiveDateTime};
use tracing::{info, error};
use sqlx::Type;
mod config;

#[derive(Debug, sqlx::Type, Serialize)]
#[sqlx(type_name = "gradientchoices", rename_all = "lowercase")]
enum GradientChoices {
    Flat,
    Slope,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
struct Plot {
    id: Uuid,
    name: String,
    plot_iterator: Option<i32>,
    area_id: Uuid,
    gradient: Option<GradientChoices>,  // Nullable enum
    vegetation_type: Option<String>,
    topography: Option<String>,
    aspect: Option<String>,
    created_on: Option<NaiveDate>,
    weather: Option<String>,
    lithology: Option<String>,
    last_updated: Option<NaiveDateTime>,
    image: Option<String>,
    geom: Option<String>,  // Geometry as WKT String
}

async fn get_plots(State(pool): State<PgPool>) -> Json<Vec<Plot>> {
    let plots = sqlx::query_as!(
        Plot,
        r#"
        SELECT id, name, plot_iterator, area_id, gradient::text as "gradient: GradientChoices",
               vegetation_type, topography, aspect, created_on, weather, lithology,
               last_updated, image, ST_AsText(geom) as geom
        FROM plot
        "#
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch plots");

    Json(plots)
}

#[tokio::main]
async fn main() {
    // Set up tracing/logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let cfg = config::Config::from_env();

    // Create a PostgreSQL connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.db_url.as_ref().unwrap())
        .await
        .expect("Could not connect to the database");

    // Build the router with the new route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/plots", get(get_plots))  // New route to get all plots
        .with_state(pool);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
