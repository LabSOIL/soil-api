use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
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
use sea_orm::{
    query::*, sea_query::Expr, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
// use sea_query::Expr;
use serde::Deserialize;
use uuid::Uuid;

const RESOURCE_NAME: &str = "sensors";
#[derive(Deserialize)]
pub struct SensorQueryParams {
    pub low_resolution: Option<bool>,
}
pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/:sensor_id", routing::get(get_one))
        .route("/", routing::get(get_all))
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

pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    let (offset, limit) = parse_range(params.range.clone());

    let condition = apply_filters(params.filter.clone(), &[("name", super::db::Column::Name)]);

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[("id", super::db::Column::Id)],
        super::db::Column::Id,
    );

    let objs: Vec<super::db::Model> = super::db::Entity::find()
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    // Map the results from the database models
    let response_objs: Vec<super::models::SensorSimple> =
        objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = <super::db::Entity>::find()
        .filter(condition.clone())
        .count(&db)
        .await
        .unwrap_or(0);

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);

    (headers, Json(response_objs))
}
