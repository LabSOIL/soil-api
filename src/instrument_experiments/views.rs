use crate::instrument_experiments::channels::db as channel_db;
use crate::instrument_experiments::models::InstrumentExperiment;
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use crudcrate::routes as crud;
use csv;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<InstrumentExperiment>)
                .post(crud::create_one::<InstrumentExperiment>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<InstrumentExperiment>)
                .put(crud::update_one::<InstrumentExperiment>)
                .delete(crud::delete_one::<InstrumentExperiment>),
        )
        // Custom endpoint for raw CSV data with channels
        .route("/{id}/raw", get(get_raw_data))
        .route("/batch", delete(crud::delete_many::<InstrumentExperiment>))
        .with_state(db)
}
#[debug_handler]
pub async fn get_raw_data(
    Path(id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<Vec<String>>>, (StatusCode, Json<String>)> {
    // Fetch the experiment using the CRUD method.
    let experiment: super::db::Model = super::db::Entity::find()
        .filter(super::db::Column::Id.eq(id))
        .one(&db)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)
        .unwrap()
        .ok_or((
            axum::http::StatusCode::NOT_FOUND,
            Json("Not found".to_string()),
        ))?;

    let channels = super::channels::db::Entity::find()
        .filter(channel_db::Column::ExperimentId.eq(id))
        .all(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    // Sort channels by channel_name.
    let mut channels = channels;
    channels.sort_by(|a, b| a.channel_name.cmp(&b.channel_name));

    // Build CSV header: first column "Time/s", then one per channel.
    let mut header = vec!["Time/s".to_string()];
    for channel in &channels {
        header.push(channel.channel_name.clone());
    }
    let mut csv_data = vec![header];

    // Use the first channel's time_values as reference for the number of rows.
    if let Some(first_channel) = channels.get(0) {
        // Deserialize time_values from JSON into Vec<f64>.
        let time_values: Vec<f64> = if let Some(json) = &first_channel.time_values {
            serde_json::from_value(json.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        let len = time_values.len();
        // For each time value, build a CSV row with the time and each channel's raw value.
        for i in 0..len {
            let mut row = vec![time_values[i].to_string()];
            for channel in &channels {
                // Deserialize raw_values from JSON into Vec<f64>.
                let raw_values: Vec<f64> = if let Some(json) = &channel.raw_values {
                    serde_json::from_value(json.clone()).unwrap_or_default()
                } else {
                    Vec::new()
                };
                let value = raw_values
                    .get(i)
                    .map_or("N/A".to_string(), |v| v.to_string());
                row.push(value);
            }
            csv_data.push(row);
        }
    }

    Ok(Json(csv_data))
}
