use crate::common::models::FilterOptions;
use crate::transects::models::Transect;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    http::header::HeaderMap,
    routing, Json, Router,
};
use sea_orm::query::*;
use sea_orm::{Condition, DatabaseConnection, EntityTrait};
use sea_query::{Alias, Expr, Order};

use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all_transects))
        .route("/:transect_id", routing::get(get_one_transect))
        .with_state(db)
}

#[utoipa::path(get, path = "/v1/transects", responses((status = 200, body = Transect)))]
pub async fn get_all_transects(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    // Default sorting and range values
    let default_sort_column = "id";
    let default_sort_order = "ASC";

    // Parse filter, range, and sort parameters
    let filters: HashMap<String, String> = if let Some(filter) = params.filter {
        serde_json::from_str(&filter).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let (offset, limit) = if let Some(range) = params.range {
        let range_vec: Vec<u64> = serde_json::from_str(&range).unwrap_or(vec![0, 24]);
        let start = range_vec.get(0).copied().unwrap_or(0);
        let end = range_vec.get(1).copied().unwrap_or(24);
        let limit = end - start + 1;
        (start, limit)
    } else {
        (0, 25)
    };

    let (sort_column, sort_order) = if let Some(sort) = params.sort {
        let sort_vec: Vec<String> = serde_json::from_str(&sort).unwrap_or(vec![
            default_sort_column.to_string(),
            default_sort_order.to_string(),
        ]);
        (
            sort_vec
                .get(0)
                .cloned()
                .unwrap_or(default_sort_column.to_string()),
            sort_vec
                .get(1)
                .cloned()
                .unwrap_or(default_sort_order.to_string()),
        )
    } else {
        (
            default_sort_column.to_string(),
            default_sort_order.to_string(),
        )
    };

    // Apply filters
    let mut condition = Condition::all();
    for (key, mut value) in filters {
        value = value.trim().to_string();

        // Check if the value is a UUID, otherwise treat as a string filter
        if let Ok(uuid) = Uuid::parse_str(&value) {
            condition = condition.add(Expr::col(Alias::new(&key)).eq(uuid));
        } else {
            condition = condition.add(Expr::col(Alias::new(&key)).eq(value));
        }
    }

    // Sorting and pagination
    let order_direction = if sort_order == "ASC" {
        Order::Asc
    } else {
        Order::Desc
    };

    let order_column = match sort_column.as_str() {
        "id" => crate::transects::db::Column::Id,
        "name" => crate::transects::db::Column::Name,
        _ => crate::transects::db::Column::Id, // Default to sorting by ID
    };

    // Fetch transects with filtering, sorting, and pagination
    let transects: Vec<Transect> =
        Transect::get_all(&db, condition, order_column, order_direction, offset, limit).await;

    // Get total count for content range header
    let total_transects: u64 = crate::transects::db::Entity::find()
        .count(&db)
        .await
        .unwrap();
    let max_offset_limit = (offset + limit).min(total_transects);
    let content_range = format!(
        "transects {}-{}/{}",
        offset,
        max_offset_limit - 1,
        total_transects
    );

    // Return Content-Range as a header
    let mut headers = HeaderMap::new();
    headers.insert("Content-Range", content_range.parse().unwrap());
    (headers, Json(transects))
}

#[utoipa::path(get, path = "/v1/transects/{transect_id}", responses((status = 200, body = Transect)))]
pub async fn get_one_transect(
    State(db): State<DatabaseConnection>,

    Path(transect_id): Path<Uuid>,
) -> impl IntoResponse {
    let transect = Transect::get_one(transect_id, &db).await.unwrap();

    (StatusCode::OK, Json(transect))
}
