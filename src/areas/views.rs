use crate::areas::models::Entity as AreaDB; // Add Relation here
use crate::areas::schemas::{Area, FilterOptions};
use crate::models::sensor::Entity as SensorDB;
use crate::models::soilprofile::Entity as SoilProfileDB;
use crate::models::transect::Entity as TransectDB;
use crate::plots::models::Entity as PlotDB;
use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    http::header::HeaderMap,
    routing, Json, Router,
};
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::{query::*, DatabaseConnection};
use sea_query::{Alias, Expr};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .with_state(db)
}

#[utoipa::path(get, path = "/api/areas", responses((status = OK, body = PlotWithCoords)))]
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    // Default values for range and sorting
    let default_sort_column = "id";
    let default_sort_order = "ASC";

    // 1. Parse the filter, range, and sort parameters
    let filters: HashMap<String, String> = if let Some(filter) = params.filter {
        serde_json::from_str(&filter).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let (offset, limit) = if let Some(range) = params.range {
        let range_vec: Vec<u64> = serde_json::from_str(&range).unwrap_or(vec![0, 24]); // Default to [0, 24]
        let start = range_vec.get(0).copied().unwrap_or(0);
        let end = range_vec.get(1).copied().unwrap_or(24);
        let limit = end - start + 1;
        (start, limit) // Offset is `start`, limit is the number of documents to fetch
    } else {
        (0, 25) // Default to 25 documents starting at 0
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

        // Check if the value is a valid UUID
        if let Ok(uuid) = Uuid::parse_str(&value) {
            // If the value is a valid UUID, filter it as a UUID
            condition = condition.add(Expr::col(Alias::new(&key)).eq(uuid));
        } else {
            // Otherwise, treat it as a regular string filter
            condition = condition.add(Expr::col(Alias::new(&key)).eq(value));
        }
    }

    // Query with filtering, sorting, and pagination
    let order_direction = if sort_order == "ASC" {
        Order::Asc
    } else {
        Order::Desc
    };
    let order_column = match sort_column.as_str() {
        "id" => <AreaDB as sea_orm::EntityTrait>::Column::Id,
        "name" => <AreaDB as sea_orm::EntityTrait>::Column::Name,
        "last_updated" => <AreaDB as sea_orm::EntityTrait>::Column::LastUpdated,
        "description" => <AreaDB as sea_orm::EntityTrait>::Column::Description,
        "project_id" => <AreaDB as sea_orm::EntityTrait>::Column::ProjectId,
        _ => <AreaDB as sea_orm::EntityTrait>::Column::Id,
    };

    let objs = AreaDB::find()
        .filter(condition)
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    let mut areas: Vec<Area> = Vec::new();

    // Loop through each area and fetch related data asynchronously
    for obj in objs {
        // Get all plots with matching area_id
        let plots: Vec<crate::plots::models::Model> = PlotDB::find()
            .filter(crate::plots::models::Column::AreaId.eq(obj.id))
            .all(&db)
            .await
            .unwrap();

        // Get all sensors with matching area_id
        let sensors: Vec<crate::models::sensor::Model> = SensorDB::find()
            .filter(crate::models::sensor::Column::AreaId.eq(obj.id))
            .all(&db)
            .await
            .unwrap();

        // Get all transects with matching area_id
        let transects: Vec<crate::models::transect::Model> = TransectDB::find()
            .filter(crate::models::transect::Column::AreaId.eq(obj.id))
            .all(&db)
            .await
            .unwrap();

        // Get all soil profiles with matching area_id
        let soil_profiles: Vec<crate::models::soilprofile::Model> = SoilProfileDB::find()
            .filter(crate::models::soilprofile::Column::AreaId.eq(obj.id))
            .all(&db)
            .await
            .unwrap();

        areas.push(Area::from(obj, plots, soil_profiles, sensors, transects));
    }

    let total_areas: u64 = AreaDB::find().count(&db).await.unwrap();
    let max_offset_limit = (offset + limit).min(total_areas);
    let content_range = format!("areas {}-{}/{}", offset, max_offset_limit - 1, total_areas);

    // Return the Content-Range as a header
    let mut headers = HeaderMap::new();
    headers.insert("Content-Range", content_range.parse().unwrap());
    (headers, Json(json!(areas)))
}
