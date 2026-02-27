use crate::common::geometry::Geometry;
use crate::routes::private::sensors::flux_data::db as FluxDB;
use crate::routes::private::sensors::redox_data::db as RedoxDB;
use crate::routes::public::website_access::{check_sensor_access, validate_slug};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Statement};
use serde::{Deserialize, Serialize};
use soil_sensor_toolbox::{SoilType, SoilTypeModel};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SensorQueryParams {
    pub website: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
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
    let Some(website_slug) = params.website else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Missing required query parameter: 'website'".to_string()),
        ));
    };

    if !validate_slug(&website_slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid website slug".to_string()),
        ));
    }

    // Access check + profile lookup (single SQL + ORM fetch)
    let t0 = std::time::Instant::now();
    let (profile_model, website_from, website_to) =
        match check_sensor_access(&db, id, &website_slug).await {
            Ok(Some(result)) => result,
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
    tracing::debug!("access_check: {:?}", t0.elapsed());

    let profile: crate::routes::private::sensors::profile::models::SensorProfile =
        profile_model.into();

    let t1 = std::time::Instant::now();
    let (date_from, date_to, span_days) = effective_date_range(
        &db, id, website_from, website_to, params.start, params.end,
    )
    .await;
    tracing::debug!("effective_date_range: {:?}", t1.elapsed());

    let (resolution, window_hours) = resolution_for_span(span_days);

    let t2 = std::time::Instant::now();
    let depth_data = profile
        .load_average_temperature_series_by_depth_cm(&db, window_hours, date_from, date_to)
        .await
        .unwrap_or_default();
    tracing::debug!("load_temperature({}): {:?}", resolution, t2.elapsed());

    let response = super::models::SensorProfile::from_depth_map(
        profile.id, &profile.name, resolution, "\u{00B0}C", depth_data,
    );

    Ok((StatusCode::OK, Json(response)))
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
    let Some(website_slug) = params.website else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Missing required query parameter: 'website'".to_string()),
        ));
    };

    if !validate_slug(&website_slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid website slug".to_string()),
        ));
    }

    // Access check + profile lookup (single SQL + ORM fetch)
    let t0 = std::time::Instant::now();
    let (profile_model, website_from, website_to) =
        match check_sensor_access(&db, id, &website_slug).await {
            Ok(Some(result)) => result,
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
    tracing::debug!("access_check: {:?}", t0.elapsed());

    let profile: crate::routes::private::sensors::profile::models::SensorProfile =
        profile_model.into();

    let t1 = std::time::Instant::now();
    let (date_from, date_to, span_days) = effective_date_range(
        &db, id, website_from, website_to, params.start, params.end,
    )
    .await;
    tracing::debug!("effective_date_range: {:?}", t1.elapsed());

    let (resolution, window_hours) = resolution_for_span(span_days);

    let t2 = std::time::Instant::now();
    let depth_data = profile
        .load_average_moisture_series_by_depth_cm(&db, window_hours, date_from, date_to)
        .await
        .unwrap_or_default();
    tracing::debug!("load_moisture({}): {:?}", resolution, t2.elapsed());

    let response = super::models::SensorProfile::from_depth_map(
        profile.id, &profile.name, resolution, "VWC", depth_data,
    );

    Ok((StatusCode::OK, Json(response)))
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
    let Some(website_slug) = params.website else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Missing required query parameter: 'website'".to_string()),
        ));
    };

    if !validate_slug(&website_slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid website slug".to_string()),
        ));
    }

    // Access check + profile lookup
    let (profile, date_from, date_to) = match check_sensor_access(&db, id, &website_slug).await {
        Ok(Some(result)) => result,
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

    let geom = Geometry {
        srid: profile.coord_srid.unwrap_or_default(),
        x: profile.coord_x.unwrap_or_default(),
        y: profile.coord_y.unwrap_or_default(),
        z: profile.coord_z.unwrap_or_default(),
    }
    .to_hashmap(vec![4326]);

    let mut flux_query = FluxDB::Entity::find()
        .filter(FluxDB::Column::SensorprofileId.eq(id));

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
    let Some(website_slug) = params.website else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Missing required query parameter: 'website'".to_string()),
        ));
    };

    if !validate_slug(&website_slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid website slug".to_string()),
        ));
    }

    // Access check + profile lookup
    let (profile, date_from, date_to) = match check_sensor_access(&db, id, &website_slug).await {
        Ok(Some(result)) => result,
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

    let geom = Geometry {
        srid: profile.coord_srid.unwrap_or_default(),
        x: profile.coord_x.unwrap_or_default(),
        y: profile.coord_y.unwrap_or_default(),
        z: profile.coord_z.unwrap_or_default(),
    }
    .to_hashmap(vec![4326]);

    let mut redox_query = RedoxDB::Entity::find()
        .filter(RedoxDB::Column::SensorprofileId.eq(id));

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

/// Compute the effective date range and span in days for a sensor profile.
/// If the user provides start/end, those are clipped to the website access range.
/// Returns (effective_from, effective_to, span_days).
async fn effective_date_range(
    db: &DatabaseConnection,
    profile_id: Uuid,
    website_from: Option<DateTime<Utc>>,
    website_to: Option<DateTime<Utc>>,
    requested_start: Option<DateTime<Utc>>,
    requested_end: Option<DateTime<Utc>>,
) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>, i64) {
    let sql = r"
        SELECT MIN(date_from) AS min_from, MAX(date_to) AS max_to
        FROM sensorprofile_assignment
        WHERE sensorprofile_id = $1
    ";
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        vec![profile_id.into()],
    );

    let (assign_from, assign_to) = if let Ok(Some(row)) = db.query_one(stmt).await {
        let af: Option<DateTime<Utc>> = row.try_get("", "min_from").ok();
        let at: Option<DateTime<Utc>> = row.try_get("", "max_to").ok();
        (af, at)
    } else {
        (None, None)
    };

    // Base range: website access clipped by assignment dates
    let base_from = website_from.or(assign_from);
    let base_to = website_to.or(assign_to);

    // If user provided start/end, intersect with base range
    let effective_from = match (requested_start, base_from) {
        (Some(req), Some(base)) => Some(req.max(base)),
        (Some(req), None) => Some(req),
        (None, base) => base,
    };
    let effective_to = match (requested_end, base_to) {
        (Some(req), Some(base)) => Some(req.min(base)),
        (Some(req), None) => Some(req),
        (None, base) => base,
    };

    let span_days = match (effective_from, effective_to) {
        (Some(from), Some(to)) => (to - from).num_days(),
        _ => 365, // Unknown range -> default to daily aggregate
    };

    (effective_from, effective_to, span_days)
}

/// Select the resolution label and optional window size for a given span in days.
fn resolution_for_span(span_days: i64) -> (&'static str, Option<i64>) {
    if span_days <= 7 {
        ("raw", None)         // every data point
    } else {
        ("hourly", Some(1))   // 1-hour window from raw data (matches production v2.2.0)
    }
}
