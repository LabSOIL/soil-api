use crate::common::auth::Role;
use crate::instrument_experiments::channels::db as channel_db;
use crate::instrument_experiments::db;
use crate::instrument_experiments::models::{
    InstrumentExperiment, InstrumentExperimentCreate, InstrumentExperimentUpdate,
};
use axum::extract::{Path, State};
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use crudcrate::{CRUDResource, crud_handlers};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::{Value as JsonValue, json};
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

crud_handlers!(
    InstrumentExperiment,
    InstrumentExperimentUpdate,
    InstrumentExperimentCreate
);

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    InstrumentExperiment: CRUDResource,
{
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one_handler))
        .routes(routes!(get_all_handler))
        .routes(routes!(create_one_handler))
        .routes(routes!(update_one_handler))
        .routes(routes!(delete_one_handler))
        .routes(routes!(delete_many_handler))
        // Custom endpoints for CSV data
        .routes(routes!(get_raw_data))
        .routes(routes!(get_filtered_data))
        .routes(routes!(get_summary_data))
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
            InstrumentExperiment::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}

/// Returns CSV data (as JSON) built from the raw time and raw_values of each channel.
#[utoipa::path(
    get,
    path = "/{id}/raw",
    responses(
        (status = 200, description = "Raw data found", body = Vec<Vec<String>>),
        (status = 404, description = "Experiment not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, description = "Experiment ID")
    )
)]
pub async fn get_raw_data(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<Vec<String>>>, (StatusCode, Json<String>)> {
    // Fetch the experiment (using a direct query here for brevity)
    let _experiment: db::Model = db::Entity::find()
        .filter(db::Column::Id.eq(id))
        .one(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, Json("Not found".to_string())))?
        .ok_or((StatusCode::NOT_FOUND, Json("Not found".to_string())))?;

    // Fetch associated channels for this experiment.
    let channels = channel_db::Entity::find()
        .filter(channel_db::Column::ExperimentId.eq(id))
        .all(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("DB error".to_string()),
            )
        })?;

    let mut channels = channels;
    channels.sort_by(|a, b| a.channel_name.cmp(&b.channel_name));

    // Build CSV header: "Time/s" plus each channel's name.
    let mut header = vec!["Time/s".to_string()];
    for channel in &channels {
        header.push(channel.channel_name.clone());
    }
    let mut csv_data = vec![header];

    // Use the first channel's time_values as the reference.
    if let Some(first_channel) = channels.first() {
        let time_values: Vec<f64> = if let Some(json) = &first_channel.time_values {
            serde_json::from_value(json.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        for time_value in time_values.clone() {
            let mut row = vec![time_value.to_string()];
            for channel in &channels {
                let raw_values: Vec<f64> = if let Some(json) = &channel.raw_values {
                    serde_json::from_value(json.clone()).unwrap_or_default()
                } else {
                    Vec::new()
                };
                let value = raw_values
                    .get(
                        time_values
                            .iter()
                            .position(|&v| (v - time_value).abs() < f64::EPSILON)
                            .unwrap_or(0),
                    )
                    .map_or("N/A".to_string(), std::string::ToString::to_string);
                row.push(value);
            }
            csv_data.push(row);
        }
    }
    Ok(Json(csv_data))
}

/// Returns baseline-filtered CSV data built by slicing each channel’s baseline_values
/// according to the "start" and "end" markers in each channel’s integral_results.
#[utoipa::path(
    get,
    path = "/{id}/filtered",
    responses(
        (status = 200, description = "Filtered data found", body = Vec<Vec<JsonValue>>),
        (status = 404, description = "Experiment not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, description = "Experiment ID")
    )
)]
pub async fn get_filtered_data(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<Vec<JsonValue>>>, (StatusCode, Json<String>)> {
    // Query channels for the given experiment ID.
    let channels = channel_db::Entity::find()
        .filter(channel_db::Column::ExperimentId.eq(id))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;
    if channels.is_empty() {
        return Err((StatusCode::NOT_FOUND, Json("No channels found".to_string())));
    }
    let mut channels = channels;
    channels.sort_by(|a, b| a.channel_name.cmp(&b.channel_name));

    let mut samples = build_vector_of_samples(&channels);

    // Ensure unique column names.
    ensure_unique_column_name(&mut samples);

    // Sort samples by their column name.
    samples.sort_by(|a, b| {
        let a_col = a.get("column").and_then(|v| v.as_str()).unwrap_or("");
        let b_col = b.get("column").and_then(|v| v.as_str()).unwrap_or("");
        a_col.cmp(b_col)
    });

    // Adjust each sample: set adjusted_start = 0 and adjusted_end = end - start.
    for sample in &mut samples {
        let start = sample
            .get("start")
            .and_then(sea_orm::JsonValue::as_f64)
            .unwrap_or(0.0);
        let end = sample
            .get("end")
            .and_then(sea_orm::JsonValue::as_f64)
            .unwrap_or(0.0);
        sample.insert("adjusted_start".to_string(), json!(0.0));
        sample.insert("adjusted_end".to_string(), json!(end - start));
    }

    // Build CSV header: "time/s" plus each sample's column name.
    let mut header: Vec<JsonValue> = vec![JsonValue::String("time/s".to_string())];
    for sample in &samples {
        if let Some(col) = sample.get("column").and_then(|v| v.as_str()) {
            header.push(JsonValue::String(col.to_string()));
        }
    }
    let mut csv_data: Vec<Vec<JsonValue>> = vec![header];

    // Determine the time_step from the first channel.
    let first_channel = channels.first().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json("No channel found".to_string()),
    ))?;
    let time_values: Vec<f64> = if let Some(ref json) = first_channel.time_values {
        serde_json::from_value(json.clone()).unwrap_or_default()
    } else {
        Vec::new()
    };
    if time_values.len() < 2 {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Not enough time values".to_string()),
        ));
    }
    #[allow(clippy::cast_possible_truncation)]
    let time_step = (time_values[1] - time_values[0]).round() as i64;
    #[allow(clippy::cast_possible_truncation)]
    let max_time = samples
        .iter()
        .filter_map(|sample| sample.get("end").and_then(sea_orm::JsonValue::as_f64))
        .fold(0.0, f64::max)
        .round() as i64;

    // Build CSV rows: for each time value (stepping by time_step), add baseline data from each sample.
    let mut t = 0;
    while t <= max_time {
        let mut row: Vec<JsonValue> = vec![JsonValue::Number(serde_json::Number::from(t))];
        let mut empty_count = 0;
        for sample in &samples {
            let baseline: Vec<JsonValue> = sample
                .get("baseline_values")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let value = baseline
                .get(usize::try_from(t / time_step).unwrap())
                .cloned()
                .unwrap_or(JsonValue::Null);
            if value.is_null() {
                empty_count += 1;
            }
            row.push(value);
        }
        if empty_count == samples.len() {
            break;
        }
        csv_data.push(row);
        t += time_step;
    }
    Ok(Json(csv_data))
}

fn ensure_unique_column_name(
    samples: &mut Vec<serde_json::Map<String, JsonValue>>,
) -> &mut Vec<serde_json::Map<String, JsonValue>> {
    let mut updates = Vec::new();
    for i in 0..samples.len() {
        let col_i = samples[i]
            .get("column")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        for (j, sample_j) in samples.iter().enumerate() {
            if i != j {
                let col_j = sample_j
                    .get("column")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if col_i == col_j {
                    updates.push((i, format!("{col_i}_{i}")));
                }
            }
        }
    }

    for (i, new_col) in updates {
        samples[i].insert("column".to_string(), JsonValue::String(new_col));
    }

    samples
}

fn build_vector_of_samples(
    channels: &Vec<channel_db::Model>,
) -> Vec<serde_json::Map<String, JsonValue>> {
    // Build a vector of sample results.
    // Each sample result is a JSON object (represented as a serde_json::Map)
    let mut samples: Vec<serde_json::Map<String, JsonValue>> = Vec::new();

    for channel in channels {
        // Parse time_values and baseline_values as Vec<f64>.
        let time_values: Vec<f64> = if let Some(ref json) = channel.time_values {
            serde_json::from_value(json.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };
        let baseline_values: Vec<f64> = if let Some(ref json) = channel.baseline_values {
            serde_json::from_value(json.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Parse integral_results as an array of JSON objects.
        let integral_results: Vec<JsonValue> = if let Some(ref json) = channel.integral_results {
            serde_json::from_value(json.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        for mut result in integral_results {
            // Insert channel_name into the result.
            if let Some(obj) = result.as_object_mut() {
                obj.insert(
                    "channel_name".to_string(),
                    JsonValue::String(channel.channel_name.clone()),
                );
                // Compute a column name using channel_name and sample_name (or "undefined")
                let sample_name = obj
                    .get("sample_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("undefined");
                let column = format!("{}_{}", channel.channel_name, sample_name)
                    .to_lowercase()
                    .replace(' ', "_");
                obj.insert("column".to_string(), JsonValue::String(column));
            }

            // Get start and end markers.
            let start = result
                .get("start")
                .and_then(sea_orm::JsonValue::as_f64)
                .unwrap_or(0.0);
            let end = result
                .get("end")
                .and_then(sea_orm::JsonValue::as_f64)
                .unwrap_or(0.0);
            // Find indices in time_values.
            let start_index = time_values.iter().position(|&v| (v - start).abs() < 1e-6);
            let end_index = time_values.iter().position(|&v| (v - end).abs() < 1e-6);
            if let (Some(si), Some(ei)) = (start_index, end_index) {
                let slice: Vec<f64> = baseline_values.get(si..ei).unwrap_or(&[]).to_vec();
                if let Some(obj) = result.as_object_mut() {
                    obj.insert(
                        "baseline_values".to_string(),
                        serde_json::to_value(slice).unwrap_or(JsonValue::Null),
                    );
                }
            }
            samples.push(result.as_object().cloned().unwrap());
        }
    }
    samples
}

/// Returns a summary CSV that reports each channel’s integral results.
#[utoipa::path(
    get,
    path = "/{id}/summary",
    responses(
        (status = 200, description = "Summary data found", body = Vec<Vec<String>>),
        (status = 404, description = "Experiment not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, description = "Experiment ID")
    )
)]
pub async fn get_summary_data(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<Vec<String>>>, (StatusCode, Json<String>)> {
    let channels = channel_db::Entity::find()
        .filter(channel_db::Column::ExperimentId.eq(id))
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;
    if channels.is_empty() {
        return Err((StatusCode::NOT_FOUND, Json("No channels found".to_string())));
    }
    let mut channels = channels;
    channels.sort_by(|a, b| a.channel_name.cmp(&b.channel_name));

    // Determine the maximum number of samples (i.e. length of integral_results) among all channels.
    let mut max_samples = 0;
    let mut channel_results: Vec<(String, Vec<JsonValue>)> = Vec::new();
    for channel in &channels {
        let integral_results: Vec<JsonValue> = if let Some(ref json) = channel.integral_results {
            serde_json::from_value(json.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };
        max_samples = max_samples.max(integral_results.len());
        channel_results.push((channel.channel_name.clone(), integral_results));
    }

    // Build CSV header: "measurement" plus four columns per sample.
    let mut header = vec!["measurement".to_string()];
    for i in 1..=max_samples {
        header.push(format!("sample{i}_start"));
        header.push(format!("sample{i}_end"));
        header.push(format!("sample{i}_electrons_transferred_mol"));
        header.push(format!("sample{i}_sample_name"));
    }
    let mut csv_data = vec![header];

    // For each channel, build a row with its name and then each sample's integral data.
    for (channel_name, integral_results) in channel_results {
        let mut row = vec![channel_name];
        for sample in &integral_results {
            let start = sample
                .get("start")
                .and_then(sea_orm::JsonValue::as_f64)
                .map_or("nan".to_string(), |v| v.to_string());
            let end = sample
                .get("end")
                .and_then(sea_orm::JsonValue::as_f64)
                .map_or("nan".to_string(), |v| v.to_string());
            let area = sample
                .get("area")
                .and_then(sea_orm::JsonValue::as_f64)
                .map_or("nan".to_string(), |v| v.to_string());
            let sample_name = sample
                .get("sample_name")
                .and_then(|v| v.as_str())
                .unwrap_or("nan")
                .to_string();
            row.push(start);
            row.push(end);
            row.push(area);
            row.push(sample_name);
        }
        // If there are fewer samples than max_samples, fill the remaining columns with "nan".
        let remaining = max_samples - row.len() / 4;
        for _ in 0..remaining {
            row.push("nan".to_string());
            row.push("nan".to_string());
            row.push("nan".to_string());
            row.push("nan".to_string());
        }
        csv_data.push(row);
    }
    Ok(Json(csv_data))
}
