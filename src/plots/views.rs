use crate::plots::models::PlotRead;
use crate::plots::schemas::FilterOptions;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;

pub async fn get_plots(
    opts: Option<Query<FilterOptions>>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Params
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let page = opts.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    // First query: Get total count of plots (without pagination)
    let total_count: Option<i64> = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM plot
        "#
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Second query: Get paginated plot records with area info
    let plots = sqlx::query_as!(
        PlotRead,
        r#"
        SELECT plot.id,
            ST_X(st_transform(plot.geom, 2056)) as coord_x,
            ST_Y(st_transform(plot.geom, 2056)) as coord_y,
            ST_Z(st_transform(plot.geom, 2056)) as coord_z,
            ST_X(st_transform(plot.geom, 4326)) as longitude,
            ST_Y(st_transform(plot.geom, 4326)) as latitude,
            plot.name, plot.area_id, plot.gradient::text, plot.vegetation_type,
            plot.topography, plot.aspect, plot.weather, plot.lithology,
            plot.image, plot.iterator, plot.plot_iterator, plot.last_updated, plot.created_on,
            area.name as area_name, area.description as area_description, area.project_id as area_project_id
        FROM plot
        JOIN area ON plot.area_id = area.id
        ORDER BY plot.id
        OFFSET $1
        LIMIT $2
        "#,
        offset as i32,
        limit as i32
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    // Response body
    let json_response = serde_json::json!({
        "status": "ok",
        "count": plots.len(),
        "total_count": total_count,  // Return the total count
        "plots": plots,
        "offset": offset,
        "pagination": {
            "limit": limit,
            "page": page,
        },
    });

    // Set Content-Range header using the total count
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Range",
        HeaderValue::from_str(&format!(
            "plot {}-{}/{}",
            offset,
            offset + plots.len(),
            total_count.unwrap_or(0)
        ))
        .unwrap(),
    );
    headers.insert(
        "Content-Length",
        HeaderValue::from_str(&json_response.to_string().len().to_string()).unwrap(),
    );

    // Return JSON response with headers
    Ok((StatusCode::OK, headers, Json(json_response)))
}
