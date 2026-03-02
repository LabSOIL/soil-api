use super::models::{FluxData, FluxDataCreate, FluxDataUpdate};
use crate::common::auth::Role;
use crate::routes::private::sensors::profile::db as ProfileDB;
use axum::response::IntoResponse;
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, crud_handlers};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use soil_sensor_toolbox::compute_gas_flux;
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

crud_handlers!(FluxData, FluxDataUpdate, FluxDataCreate);

/// A single error from a batch ingest operation.
#[derive(Serialize, ToSchema)]
pub struct BatchIngestError {
    pub index: usize,
    pub message: String,
}

/// Result of a batch ingest operation.
#[derive(Serialize, ToSchema)]
pub struct BatchIngestResult {
    pub inserted: usize,
    pub errors: Vec<BatchIngestError>,
}

/// Request body for the ingest endpoint: raw chamber time series data.
#[derive(Deserialize, ToSchema)]
pub struct IngestFluxRequest {
    pub sensorprofile_id: uuid::Uuid,
    pub measured_on: DateTime<Utc>,
    pub replicate: String,
    pub setting: Option<String>,
    pub raw_readings: Vec<RawReading>,
}

/// A single raw reading from the chamber time series.
#[derive(Deserialize, Serialize, ToSchema, Clone)]
pub struct RawReading {
    pub t: f64,
    pub co2: f64,
    pub ch4: f64,
    pub h2o: f64,
    pub temp: f64,
    pub press: f64,
    pub soilp: Option<f64>,
}

#[utoipa::path(
    post,
    path = "/ingest",
    request_body = IngestFluxRequest,
    responses(
        (status = 201, description = "Flux data ingested and computed.", body = FluxData),
        (status = 404, description = "Sensor profile not found"),
        (status = 422, description = "Invalid raw readings"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Ingest raw chamber data and compute fluxes server-side",
    description = "Accepts raw chamber time series readings, computes gas fluxes using the soil-sensor-toolbox, and stores both raw and processed data.",
    operation_id = "ingest_flux_data",
)]
pub async fn ingest_flux_data(
    axum::extract::State(db): axum::extract::State<DatabaseConnection>,
    axum::Json(req): axum::Json<IngestFluxRequest>,
) -> impl IntoResponse {
    if req.raw_readings.is_empty() {
        return Err((
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json("raw_readings must not be empty".to_string()),
        ));
    }

    // Fetch sensor profile to get volume_ml and area_cm2
    let profile = match ProfileDB::Entity::find_by_id(req.sensorprofile_id)
        .one(&db)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                axum::Json(format!(
                    "Sensor profile {} not found",
                    req.sensorprofile_id
                )),
            ));
        }
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(format!("Database error: {e}")),
            ));
        }
    };

    let volume_ml = profile.volume_ml.unwrap_or(16852.1);
    let area_cm2 = profile.area_cm2.unwrap_or(318.0);
    let volume_m3 = volume_ml * 1e-6;
    let area_m2 = area_cm2 * 1e-4;

    // Extract arrays from raw readings
    let timestamps: Vec<f64> = req.raw_readings.iter().map(|r| r.t).collect();
    let co2_ppm: Vec<f64> = req.raw_readings.iter().map(|r| r.co2).collect();
    let ch4_ppb: Vec<f64> = req.raw_readings.iter().map(|r| r.ch4).collect();
    let h2o_mmol: Vec<f64> = req.raw_readings.iter().map(|r| r.h2o).collect();
    let temp_c: Vec<f64> = req.raw_readings.iter().map(|r| r.temp).collect();
    let press_kpa: Vec<f64> = req.raw_readings.iter().map(|r| r.press).collect();

    // Compute gas fluxes server-side
    let result = compute_gas_flux(
        &timestamps,
        &co2_ppm,
        &ch4_ppb,
        &h2o_mmol,
        &temp_c,
        &press_kpa,
        volume_m3,
        area_m2,
    );

    // Compute SWC: average of soilp values where present
    let valid_soilp: Vec<f64> = req
        .raw_readings
        .iter()
        .filter_map(|r| r.soilp)
        .collect();
    let swc = if valid_soilp.is_empty() {
        None
    } else {
        Some(valid_soilp.iter().sum::<f64>() / valid_soilp.len() as f64)
    };

    let n_measurements = i32::try_from(req.raw_readings.len()).ok();

    // Serialize raw_readings to JSON
    let raw_json = match serde_json::to_value(&req.raw_readings) {
        Ok(v) => Some(v),
        Err(e) => {
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(format!("Failed to serialize raw_readings: {e}")),
            ));
        }
    };

    // Create the flux_data record
    let active_model = super::db::ActiveModel {
        id: ActiveValue::Set(uuid::Uuid::new_v4()),
        sensorprofile_id: ActiveValue::Set(req.sensorprofile_id),
        measured_on: ActiveValue::Set(req.measured_on),
        replicate: ActiveValue::Set(req.replicate),
        setting: ActiveValue::Set(req.setting),
        flux_co2_umol_m2_s: ActiveValue::Set(Some(result.flux_co2_umol_m2_s)),
        flux_ch4_nmol_m2_s: ActiveValue::Set(Some(result.flux_ch4_nmol_m2_s)),
        flux_h2o_umol_m2_s: ActiveValue::Set(Some(result.flux_h2o_umol_m2_s)),
        r2_co2: ActiveValue::Set(Some(result.r2_co2)),
        r2_ch4: ActiveValue::Set(Some(result.r2_ch4)),
        r2_h2o: ActiveValue::Set(Some(result.r2_h2o)),
        swc: ActiveValue::Set(swc),
        n_measurements: ActiveValue::Set(n_measurements),
        raw_readings: ActiveValue::Set(raw_json),
    };

    match active_model.insert(&db).await {
        Ok(model) => {
            let flux_data: FluxData = model.into();
            Ok((axum::http::StatusCode::CREATED, axum::Json(flux_data)))
        }
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(format!("Failed to insert flux data: {e}")),
        )),
    }
}

/// Process a single ingest request, returning Ok(()) on success or an error message.
async fn process_single_ingest(
    db: &DatabaseConnection,
    req: IngestFluxRequest,
) -> Result<(), String> {
    if req.raw_readings.is_empty() {
        return Err("raw_readings must not be empty".to_string());
    }

    let profile = ProfileDB::Entity::find_by_id(req.sensorprofile_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {e}"))?
        .ok_or_else(|| format!("Sensor profile {} not found", req.sensorprofile_id))?;

    let volume_ml = profile.volume_ml.unwrap_or(16852.1);
    let area_cm2 = profile.area_cm2.unwrap_or(318.0);
    let volume_m3 = volume_ml * 1e-6;
    let area_m2 = area_cm2 * 1e-4;

    let timestamps: Vec<f64> = req.raw_readings.iter().map(|r| r.t).collect();
    let co2_ppm: Vec<f64> = req.raw_readings.iter().map(|r| r.co2).collect();
    let ch4_ppb: Vec<f64> = req.raw_readings.iter().map(|r| r.ch4).collect();
    let h2o_mmol: Vec<f64> = req.raw_readings.iter().map(|r| r.h2o).collect();
    let temp_c: Vec<f64> = req.raw_readings.iter().map(|r| r.temp).collect();
    let press_kpa: Vec<f64> = req.raw_readings.iter().map(|r| r.press).collect();

    let result = compute_gas_flux(
        &timestamps,
        &co2_ppm,
        &ch4_ppb,
        &h2o_mmol,
        &temp_c,
        &press_kpa,
        volume_m3,
        area_m2,
    );

    let valid_soilp: Vec<f64> = req.raw_readings.iter().filter_map(|r| r.soilp).collect();
    let swc = if valid_soilp.is_empty() {
        None
    } else {
        Some(valid_soilp.iter().sum::<f64>() / valid_soilp.len() as f64)
    };

    let n_measurements = i32::try_from(req.raw_readings.len()).ok();

    let raw_json = serde_json::to_value(&req.raw_readings)
        .map_err(|e| format!("Failed to serialize raw_readings: {e}"))?;

    let active_model = super::db::ActiveModel {
        id: ActiveValue::Set(uuid::Uuid::new_v4()),
        sensorprofile_id: ActiveValue::Set(req.sensorprofile_id),
        measured_on: ActiveValue::Set(req.measured_on),
        replicate: ActiveValue::Set(req.replicate),
        setting: ActiveValue::Set(req.setting),
        flux_co2_umol_m2_s: ActiveValue::Set(Some(result.flux_co2_umol_m2_s)),
        flux_ch4_nmol_m2_s: ActiveValue::Set(Some(result.flux_ch4_nmol_m2_s)),
        flux_h2o_umol_m2_s: ActiveValue::Set(Some(result.flux_h2o_umol_m2_s)),
        r2_co2: ActiveValue::Set(Some(result.r2_co2)),
        r2_ch4: ActiveValue::Set(Some(result.r2_ch4)),
        r2_h2o: ActiveValue::Set(Some(result.r2_h2o)),
        swc: ActiveValue::Set(swc),
        n_measurements: ActiveValue::Set(n_measurements),
        raw_readings: ActiveValue::Set(Some(raw_json)),
    };

    active_model
        .insert(db)
        .await
        .map_err(|e| format!("Failed to insert flux data: {e}"))?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/ingest_batch",
    request_body = Vec<IngestFluxRequest>,
    responses(
        (status = 200, description = "Batch ingest results.", body = BatchIngestResult),
    ),
    summary = "Batch ingest raw chamber data and compute fluxes server-side",
    description = "Accepts an array of raw chamber time series readings. Each entry is processed independently; errors are recorded per-entry without aborting the batch.",
    operation_id = "ingest_flux_data_batch",
)]
pub async fn ingest_flux_data_batch(
    axum::extract::State(db): axum::extract::State<DatabaseConnection>,
    axum::Json(requests): axum::Json<Vec<IngestFluxRequest>>,
) -> axum::Json<BatchIngestResult> {
    let mut inserted = 0usize;
    let mut errors = Vec::new();

    for (index, req) in requests.into_iter().enumerate() {
        match process_single_ingest(&db, req).await {
            Ok(()) => inserted += 1,
            Err(message) => errors.push(BatchIngestError { index, message }),
        }
    }

    axum::Json(BatchIngestResult { inserted, errors })
}

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    FluxData: CRUDResource,
{
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one_handler))
        .routes(routes!(get_all_handler))
        .routes(routes!(create_one_handler))
        .routes(routes!(update_one_handler))
        .routes(routes!(delete_one_handler))
        .routes(routes!(delete_many_handler))
        .routes(routes!(ingest_flux_data))
        .routes(routes!(ingest_flux_data_batch))
        .with_state(db.clone());

    if let Some(instance) = keycloak_auth_instance {
        mutating_router = mutating_router.layer(
            KeycloakAuthLayer::<Role>::builder()
                .instance(instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![Role::Administrator])
                .build(),
        );
    } else {
        println!(
            "Warning: Mutating routes of {} router are not protected",
            FluxData::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}
