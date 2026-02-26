use crate::common::geometry::Geometry;
use crate::config::Config;
use crate::routes::private::sensors::flux_data::db as FluxDB;
use crate::routes::private::sensors::profile::db as ProfileDB;
use crate::routes::private::sensors::redox_data::db as RedoxDB;
use crate::routes::public::website_access::check_sensor_access;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_response_cache::CacheLayer;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use soil_sensor_toolbox::{SoilType, SoilTypeModel};
use std::collections::HashMap;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SensorQueryParams {
    pub website: String,
}

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
        .routes(routes!(get_one_flux))
        .routes(routes!(get_one_redox))
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
    Query(params): Query<SensorQueryParams>,
) -> impl IntoResponse {
    // Access check: sensor must be in an area assigned to this website and not excluded
    let date_range = match check_sensor_access(&db, id, &params.website).await {
        Ok(Some(range)) => range,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json("Sensor profile not found".to_string()),
            ))
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal server error".to_string()),
            ))
        }
    };

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

            // Apply date filtering from website access
            let (date_from, date_to) = date_range;
            if date_from.is_some() || date_to.is_some() {
                for data in profile.data_by_depth_cm.values_mut() {
                    data.retain(|d| {
                        date_from.is_none_or(|df| d.time_utc >= df)
                            && date_to.is_none_or(|dt| d.time_utc <= dt)
                    });
                }
            }

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
    Query(params): Query<SensorQueryParams>,
) -> impl IntoResponse {
    // Access check: sensor must be in an area assigned to this website and not excluded
    let date_range = match check_sensor_access(&db, id, &params.website).await {
        Ok(Some(range)) => range,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json("Sensor profile not found".to_string()),
            ))
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal server error".to_string()),
            ))
        }
    };

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

            // Apply date filtering from website access
            let (date_from, date_to) = date_range;
            if date_from.is_some() || date_to.is_some() {
                for data in profile.data_by_depth_cm.values_mut() {
                    data.retain(|d| {
                        date_from.is_none_or(|df| d.time_utc >= df)
                            && date_to.is_none_or(|dt| d.time_utc <= dt)
                    });
                }
            }

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
    path = "/{id}/flux",
    responses(
        (status = 200, description = "Sensor profile with flux data.", body = super::models::SensorProfileFlux),
        (status = 404, description = "Sensor profile not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get sensor - flux data (public)",
    description = "Returns the sensor profile and its gas flux time series data.",
    operation_id = "get_one_sensor_profile_flux_public",
)]
pub async fn get_one_flux(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Query(params): Query<SensorQueryParams>,
) -> impl IntoResponse {
    // Access check
    let date_range = match check_sensor_access(&db, id, &params.website).await {
        Ok(Some(range)) => range,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json("Sensor profile not found".to_string()),
            ))
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal server error".to_string()),
            ))
        }
    };

    match ProfileDB::Entity::find_by_id(id).one(&db).await {
        Ok(Some(profile)) => {
            let geom = Geometry {
                srid: profile.coord_srid.unwrap_or_default(),
                x: profile.coord_x.unwrap_or_default(),
                y: profile.coord_y.unwrap_or_default(),
                z: profile.coord_z.unwrap_or_default(),
            }
            .to_hashmap(vec![4326]);

            let mut flux_query = FluxDB::Entity::find()
                .filter(FluxDB::Column::SensorprofileId.eq(id));

            // Apply date filtering
            let (date_from, date_to) = date_range;
            if let Some(df) = date_from {
                flux_query = flux_query.filter(FluxDB::Column::MeasuredOn.gte(df));
            }
            if let Some(dt) = date_to {
                flux_query = flux_query.filter(FluxDB::Column::MeasuredOn.lte(dt));
            }

            let flux_records = flux_query
                .order_by_asc(FluxDB::Column::MeasuredOn)
                .all(&db)
                .await
                .unwrap_or_default();

            let flux_data: Vec<super::models::FluxDataPoint> = flux_records
                .into_iter()
                .map(|r| super::models::FluxDataPoint {
                    measured_on: r.measured_on,
                    replicate: r.replicate,
                    setting: r.setting,
                    flux_co2_umol_m2_s: r.flux_co2_umol_m2_s,
                    flux_ch4_nmol_m2_s: r.flux_ch4_nmol_m2_s,
                    flux_h2o_umol_m2_s: r.flux_h2o_umol_m2_s,
                    r2_co2: r.r2_co2,
                    r2_ch4: r.r2_ch4,
                    r2_h2o: r.r2_h2o,
                    swc: r.swc,
                })
                .collect();

            let response = super::models::SensorProfileFlux {
                id: profile.id,
                name: profile.name,
                geom,
                flux_data,
            };

            Ok((StatusCode::OK, Json(response)))
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
    path = "/{id}/redox",
    responses(
        (status = 200, description = "Sensor profile with redox data.", body = super::models::SensorProfileRedox),
        (status = 404, description = "Sensor profile not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get sensor - redox data (public)",
    description = "Returns the sensor profile and its redox potential time series data.",
    operation_id = "get_one_sensor_profile_redox_public",
)]
pub async fn get_one_redox(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Query(params): Query<SensorQueryParams>,
) -> impl IntoResponse {
    // Access check
    let date_range = match check_sensor_access(&db, id, &params.website).await {
        Ok(Some(range)) => range,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json("Sensor profile not found".to_string()),
            ))
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal server error".to_string()),
            ))
        }
    };

    match ProfileDB::Entity::find_by_id(id).one(&db).await {
        Ok(Some(profile)) => {
            let geom = Geometry {
                srid: profile.coord_srid.unwrap_or_default(),
                x: profile.coord_x.unwrap_or_default(),
                y: profile.coord_y.unwrap_or_default(),
                z: profile.coord_z.unwrap_or_default(),
            }
            .to_hashmap(vec![4326]);

            let mut redox_query = RedoxDB::Entity::find()
                .filter(RedoxDB::Column::SensorprofileId.eq(id));

            // Apply date filtering
            let (date_from, date_to) = date_range;
            if let Some(df) = date_from {
                redox_query = redox_query.filter(RedoxDB::Column::MeasuredOn.gte(df));
            }
            if let Some(dt) = date_to {
                redox_query = redox_query.filter(RedoxDB::Column::MeasuredOn.lte(dt));
            }

            let redox_records = redox_query
                .order_by_asc(RedoxDB::Column::MeasuredOn)
                .all(&db)
                .await
                .unwrap_or_default();

            let redox_data: Vec<super::models::RedoxDataPoint> = redox_records
                .into_iter()
                .map(|r| super::models::RedoxDataPoint {
                    measured_on: r.measured_on,
                    ch1_5cm_mv: r.ch1_5cm_mv,
                    ch2_15cm_mv: r.ch2_15cm_mv,
                    ch3_25cm_mv: r.ch3_25cm_mv,
                    ch4_35cm_mv: r.ch4_35cm_mv,
                    temp_c: r.temp_c,
                })
                .collect();

            let response = super::models::SensorProfileRedox {
                id: profile.id,
                name: profile.name,
                geom,
                redox_data,
            };

            Ok((StatusCode::OK, Json(response)))
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
