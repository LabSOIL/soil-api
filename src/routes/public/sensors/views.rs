use crate::config::Config;
use crate::routes::private::sensors::profile::db as ProfileDB;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_response_cache::CacheLayer;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use serde::Serialize;
use soil_sensor_toolbox::{SoilType, SoilTypeModel};
use std::collections::HashMap;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, ToSchema)]
struct SoilTypeResponse {
    id: String,
    name: String,
}
impl From<SoilTypeModel> for SoilTypeResponse {
    fn from(soil_type: SoilTypeModel) -> Self {
        Self {
            name: soil_type.name,
            id: soil_type.machine_name,
        }
    }
}
pub fn router(db: &DatabaseConnection) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_one_temperature))
        .routes(routes!(get_one_moisture))
        .routes(routes!(get_soil_types))
        .layer(
            CacheLayer::with_lifespan(Config::from_env().public_cache_timeout_seconds)
                .use_stale_on_failure(),
        )
        .with_state(db.clone())
}

#[utoipa::path(
    get,
    path = "/{id}/temperature",
    responses(
        (status = 200, description = "Sensor profile found.", body = crate::routes::public::sensors::models::SensorProfile),
        (status = 404, description = "Sensor profile not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get sensor - temperature (public)",
    description = "Returns the sensor and its temperature data.",
    operation_id = "get_one_sensor_profile_tempterature_public",
)]
pub async fn get_one_temperature(
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
            profile.data_by_depth_cm = profile
                .load_average_temperature_series_by_depth_cm(&db, hour_average)
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

#[utoipa::path(
    get,
    path = "/{id}/moisture",
    responses(
        (status = 200, description = "Sensor profile found.", body = crate::routes::public::sensors::models::SensorProfile),
        (status = 404, description = "Sensor profile not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get sensor - moisture (public)",
    description = "Returns the sensor and its moisture data.",
    operation_id = "get_one_sensor_profile_moisture_public",
)]
pub async fn get_one_moisture(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ProfileDB::Entity::find_by_id(id).one(&db).await {
        Ok(Some(profile)) => {
            // First convert to the private model to use the temp function
            let mut profile: crate::routes::private::sensors::profile::models::SensorProfile =
                profile.into();

            // Get the average moisture by depth cm
            let hour_average = Some(1);
            profile.data_by_depth_cm = profile
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

#[utoipa::path(
    get,
    path = "/soil_types",
    responses(
        (status = 200, description = "Sensor profile found.", body = Vec<SoilTypeResponse>),
        (status = 404, description = "Sensor profile not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get sensor soil types (public)",
    description = "Returns the soil types available for sensors to convert moisture counts to VWC.",
    operation_id = "get_soil_types_public",
)]
pub async fn get_soil_types() -> impl IntoResponse {
    let soil_types: Vec<SoilTypeResponse> = SoilType::ALL
        .iter()
        .copied()
        .map(|soil| {
            let model: SoilTypeModel = soil.into();
            let response: SoilTypeResponse = model.into();
            response
        })
        .collect();

    (StatusCode::OK, Json(soil_types))
}
