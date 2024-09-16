use crate::areas::models::Entity as AreaDB;
use crate::common::schemas::FilterOptions;
use crate::plots::models::Entity as PlotDB;
use crate::plots::models::Gradientchoices;
use crate::plots::schemas::{Area, Plot, PlotWithCoords};
use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    http::header::HeaderMap,
    routing, Json, Router,
};
use sea_orm::sqlx::Result;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::{query::*, DatabaseConnection};
use sea_query::{Alias, Expr};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

impl Serialize for Gradientchoices {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let gradient = match self {
            Gradientchoices::Flat => "Flat",
            Gradientchoices::Slope => "Slope",
        };
        serializer.serialize_str(gradient)
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .with_state(db)
}

#[utoipa::path(get, path = "/api/plots", responses((status = OK, body = PlotWithCoords)))]
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
        "id" => <PlotDB as sea_orm::EntityTrait>::Column::Id,
        "name" => <PlotDB as sea_orm::EntityTrait>::Column::Name,
        "plot_iterator" => <PlotDB as sea_orm::EntityTrait>::Column::PlotIterator,
        "area_id" => <PlotDB as sea_orm::EntityTrait>::Column::AreaId,
        "gradient" => <PlotDB as sea_orm::EntityTrait>::Column::Gradient,
        "vegetation_type" => <PlotDB as sea_orm::EntityTrait>::Column::VegetationType,
        "topography" => <PlotDB as sea_orm::EntityTrait>::Column::Topography,
        "aspect" => <PlotDB as sea_orm::EntityTrait>::Column::Aspect,
        "created_on" => <PlotDB as sea_orm::EntityTrait>::Column::CreatedOn,
        "weather" => <PlotDB as sea_orm::EntityTrait>::Column::Weather,
        "lithology" => <PlotDB as sea_orm::EntityTrait>::Column::Lithology,
        "iterator" => <PlotDB as sea_orm::EntityTrait>::Column::Iterator,
        "last_updated" => <PlotDB as sea_orm::EntityTrait>::Column::LastUpdated,
        "image" => <PlotDB as sea_orm::EntityTrait>::Column::Image,
        _ => <PlotDB as sea_orm::EntityTrait>::Column::Id,
    };

    let objs = PlotDB::find()
        .filter(condition)
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .column_as(Expr::cust("ST_X(plot.geom)"), "coord_x")
        .column_as(Expr::cust("ST_Y(plot.geom)"), "coord_y")
        .column_as(Expr::cust("ST_Z(plot.geom)"), "coord_z")
        .find_also_related(AreaDB)
        .into_model::<PlotWithCoords, Area>()
        .all(&db)
        .await
        .unwrap();

    // Map the results from the database models to the Plot struct
    let plots: Vec<Plot> = objs
        .into_iter()
        .map(|(plot, area)| Plot::from((plot, area)))
        .collect();

    let total_plots: u64 = PlotDB::find().count(&db).await.unwrap();
    let max_offset_limit = (offset + limit).min(total_plots);
    let content_range = format!("plots {}-{}/{}", offset, max_offset_limit - 1, total_plots);

    // Return the Content-Range as a header
    let mut headers = HeaderMap::new();
    headers.insert("Content-Range", content_range.parse().unwrap());
    (headers, Json(json!(plots)))
}
