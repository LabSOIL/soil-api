use std::collections::HashMap;

// use super::models::{Area, Plot};
// use crate::routes::private::areas::services::get_convex_hull;
// use crate::routes::private::plots::db as PlotDB;
use crate::routes::private::sensors::profile::db as ProfileDB;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;
use sea_orm::{
    // ColumnTrait,
    EntityTrait,
    //   ModelTrait, QueryFilter
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

pub fn router(db: &DatabaseConnection) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_one))
        .with_state(db.clone())
}

#[utoipa::path(
    get,
    path = "/{id}",
    responses(
        (status = 200, description = "Sensor profile found.", body = crate::routes::public::sensors::models::SensorProfile),
        (status = 404, description = "Sensor profile not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get sensor (public)",
    description = "Returns the sensor and its data.",
    operation_id = "get_one_sensor_profile_public",
)]
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ProfileDB::Entity::find_by_id(id).one(&db).await {
        Ok(Some(profile)) => {
            // First convert to the private model to use the temp function
            let mut profile: crate::routes::private::sensors::profile::models::SensorProfile =
                profile.into();

            // Get the average temperature by depth cm
            let hour_average = Some(1);
            profile.average_temperature_by_depth_cm = profile
                .load_average_temperature_series_by_depth_cm(&db, hour_average)
                .await
                .unwrap_or(HashMap::new());

            // Get the average moisture by depth cm
            profile.average_moisture_by_depth_cm = profile
                .load_average_moisture_series_by_depth_cm(&db, hour_average)
                .await
                .unwrap_or(HashMap::new());

            let profile: super::models::SensorProfile = profile.into();

            Ok((StatusCode::OK, Json(profile)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json("Sensor profile not found".to_string()),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal server error".to_string()),
        )),
    }
}
