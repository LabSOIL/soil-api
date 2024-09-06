// use crate::plots::models::PlotRead;
// use axum::extract::State;
// use axum::response::Json;
// use serde::Deserialize;
// use sqlx::PgPool; // Add missing import for serde
// use uuid::Uuid;

// pub async fn get_plots(State(pool): State<PgPool>) -> Json<Vec<PlotRead>> {
//     let plots = sqlx::query_as!(
//         PlotRead,
//         "SELECT id, name, area_id, gradient, vegetation_type,
//             topography, aspect, created_on, weather, lithology,
//             last_updated, image
//          FROM plot"
//     )
//     .fetch_all(&pool)
//     .await
//     .expect("Failed to fetch plots");

//     // Map Plot to PlotRead and extract coordinates from geom
//     let plot_reads: Vec<PlotRead> = plots.into_iter().collect();

//     Json(plot_reads)
// }
use crate::plots::models::PlotRead; // Fix unresolved import
use crate::plots::schemas::FilterOptions; // Fix unresolved import
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;

// use crate::{
//     models::{Plot, PlotRead},
//     schemas::FilterOptions,
//     // AppState,
// };

// fn to_plot_response(plot: &PlotRead) -> PlotRead {
//     PlotRead {
//         id: plot.id.to_owned(),
//         name: Some(plot.name.to_owned()),
//         area_id: plot.area_id.to_owned(),
//         gradient: plot.gradient.to_owned(),
//         vegetation_type: plot.vegetation_type.to_owned(),
//         topography: plot.topography.to_owned(),
//         aspect: plot.aspect.to_owned(),
//         // created_on: Some(plot.created_on.unwrap()),
//         weather: plot.weather.to_owned(),
//         lithology: plot.lithology.to_owned(),
//         // last_updated: Some(plot.last_updated.unwrap()),
//         image: plot.image.to_owned(),
//         iterator: plot.iterator.to_owned(),
//     }
// }

pub async fn get_plots(
    opts: Option<Query<FilterOptions>>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Param
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // Query with macro
    let plots = sqlx::query_as!(
        PlotRead,
        "SELECT plot.id,
            ST_X(st_transform(plot.geom, 2056)) as coord_x,
            ST_Y(st_transform(plot.geom, 2056)) as coord_y,
            ST_Z(st_transform(plot.geom, 2056)) as coord_z,
            ST_X(st_transform(plot.geom, 4326)) as longitude,
            ST_Y(st_transform(plot.geom, 4326)) as latitude,
            plot.name, plot.area_id, plot.gradient::text, plot.vegetation_type,
            plot.topography, plot.aspect, plot.weather, plot.lithology,
            plot.image, plot.iterator, plot.plot_iterator, plot.last_updated, plot.created_on,
            area.name as area_name, area.description as area_description, area.project_id as area_project_id
         FROM plot, area
         WHERE plot.area_id = area.id
         ORDER BY id
         OFFSET $1
         LIMIT $2",
        offset as i32,
        limit as i32,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Response
    let plot_responses = plots.iter().collect::<Vec<&PlotRead>>();

    let json_response = serde_json::json!({
        "status": "ok",
        "count": plot_responses.len(),
        "plots": plot_responses,
        "offset": offset,
        "area": {
            "limit": limit,
            "page": opts.page.unwrap_or(1),
        },
    });

    Ok(Json(json_response))
}
