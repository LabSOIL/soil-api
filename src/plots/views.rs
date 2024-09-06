// use crate::plots::models::{GradientChoices, Plot};
use crate::plots::models::Plot;
use axum::extract::State;
use axum::response::Json;
use sqlx::PgPool; // Make sure to import GradientChoices // Ensure to import GradientChoices

pub async fn get_plots(State(pool): State<PgPool>) -> Json<Vec<Plot>> {
    let plots = sqlx::query_as!(
        Plot,
        "SELECT id, name, plot_iterator, area_id,
                gradient::text as gradient,  -- Cast gradient enum to text
                vegetation_type, topography, aspect, created_on,
                weather, lithology, last_updated, image,
                ST_AsText(geom) as geom
        FROM plot"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch plots");

    // Map string to enum
    let plots: Vec<Plot> = plots.into_iter().collect();

    Json(plots)
}
