use crate::sensors::data::db::Entity as SensorDataDB;
use crate::sensors::db::Entity as SensorDB;
use crate::sensors::models::SensorWithData;
use crate::sensors::services::simplify_sensor_data_lttb;
use axum::extract::Path;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{routing, Router};
use sea_orm::{query::*, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sea_query::Expr;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SensorQueryParams {
    pub low_resolution: Option<bool>,
}
pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/:sensor_id", routing::get(get_one))
        // .route("/", routing::get(get_all))
        .with_state(db)
}

pub async fn get_one(
    Path(sensor_id): Path<Uuid>,
    Query(params): Query<SensorQueryParams>, // Use a structured approach to query params
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    let low_resolution = params.low_resolution.unwrap_or(false);

    // Fetch the sensor by ID
    let sensor = SensorDB::find()
        .filter(crate::sensors::db::Column::Id.eq(sensor_id))
        .column_as(Expr::cust("ST_X(sensor.geom)"), "coord_x")
        .column_as(Expr::cust("ST_Y(sensor.geom)"), "coord_y")
        .column_as(Expr::cust("ST_Z(sensor.geom)"), "coord_z")
        .into_model::<super::models::SensorWithCoords>()
        .one(&db)
        .await
        .unwrap();

    if sensor.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(SensorWithData {
                id: Uuid::nil(),
                name: Some(String::new()),
                description: Some(String::new()),
                data: Vec::new(),
                coord_x: None,
                coord_y: None,
                coord_z: None,
            }),
        );
    }

    let sensor = sensor.unwrap();

    // Fetch the associated sensor data
    let mut sensor_data = SensorDataDB::find()
        .filter(crate::sensors::data::db::Column::SensorId.eq(sensor_id))
        .order_by_asc(crate::sensors::data::db::Column::TimeUtc)
        .all(&db)
        .await
        .unwrap();

    // Apply LTTB downsampling if requested
    if low_resolution && sensor_data.len() > 100 {
        sensor_data = simplify_sensor_data_lttb(sensor_data, 100);
    }

    let response = SensorWithData {
        id: sensor.id,
        name: sensor.name,
        description: sensor.description,
        data: sensor_data,
        coord_x: sensor.coord_x,
        coord_y: sensor.coord_y,
        coord_z: sensor.coord_z,
    };

    (StatusCode::OK, Json(response))
}

// #[utoipa::path(get, path = "/api/sensors", responses((status = OK, body = SensorWithoutData)))]
// pub async fn get_all(
//     Query(params): Query<FilterOptions>,
//     State(db): State<DatabaseConnection>,
// ) -> impl IntoResponse {
//     // Default values for range and sorting
//     let default_sort_column = "id";
//     let default_sort_order = "ASC";

//     // Parse the filter, range, and sort parameters
//     let filters: HashMap<String, String> = if let Some(filter) = params.filter {
//         serde_json::from_str(&filter).unwrap_or_default()
//     } else {
//         HashMap::new()
//     };

//     let (offset, limit) = if let Some(range) = params.range {
//         let range_vec: Vec<u64> = serde_json::from_str(&range).unwrap_or(vec![0, 24]); // Default to [0, 24]
//         let start = range_vec.get(0).copied().unwrap_or(0);
//         let end = range_vec.get(1).copied().unwrap_or(24);
//         let limit = end - start + 1;
//         (start, limit) // Offset is `start`, limit is the number of documents to fetch
//     } else {
//         (0, 25) // Default to 25 documents starting at 0
//     };

//     let (sort_column, sort_order) = if let Some(sort) = params.sort {
//         let sort_vec: Vec<String> = serde_json::from_str(&sort).unwrap_or(vec![
//             default_sort_column.to_string(),
//             default_sort_order.to_string(),
//         ]);
//         (
//             sort_vec
//                 .get(0)
//                 .cloned()
//                 .unwrap_or(default_sort_column.to_string()),
//             sort_vec
//                 .get(1)
//                 .cloned()
//                 .unwrap_or(default_sort_order.to_string()),
//         )
//     } else {
//         (
//             default_sort_column.to_string(),
//             default_sort_order.to_string(),
//         )
//     };

//     // Apply filters
//     let mut condition = Condition::all();
//     for (key, mut value) in filters {
//         value = value.trim().to_string();

//         // Check if the value is a valid UUID
//         if let Ok(uuid) = Uuid::parse_str(&value) {
//             // If the value is a valid UUID, filter it as a UUID
//             condition = condition.add(Expr::col(Alias::new(&key)).eq(uuid));
//         } else {
//             // Otherwise, treat it as a regular string filter
//             condition = condition.add(Expr::col(Alias::new(&key)).eq(value));
//         }
//     }

//     // Sorting and pagination
//     let order_direction = if sort_order == "ASC" {
//         Order::Asc
//     } else {
//         Order::Desc
//     };

//     let order_column = match sort_column.as_str() {
//         "id" => <SensorDB as sea_orm::EntityTrait>::Column::Id,
//         "name" => <SensorDB as sea_orm::EntityTrait>::Column::Name,
//         "last_updated" => <SensorDB as sea_orm::EntityTrait>::Column::LastUpdated,
//         "area_id" => <SensorDB as sea_orm::EntityTrait>::Column::AreaId,
//         _ => <SensorDB as sea_orm::EntityTrait>::Column::Id,
//     };

//     // Querying sensors with filtering, sorting, and pagination
//     let sensors: Vec<super::models::SensorSimple> =
//         SensorSimple::get_all(&db, condition, order_column, order_direction, offset, limit).await;

//     let total_sensors: u64 = SensorDB::find().count(&db).await.unwrap();
//     let max_offset_limit = (offset + limit).min(total_sensors);
//     let content_range = format!(
//         "sensors {}-{}/{}",
//         offset,
//         max_offset_limit - 1,
//         total_sensors
//     );

//     // Return the Content-Range as a header
//     let mut headers = HeaderMap::new();
//     headers.insert("Content-Range", content_range.parse().unwrap());
//     (headers, Json(json!(sensors)))
// }
