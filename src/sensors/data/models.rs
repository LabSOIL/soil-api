use super::db::Model;
use chrono::NaiveDateTime;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct SensorData {
    pub id: Uuid,
    pub instrument_seq: i32,
    pub time_utc: NaiveDateTime,
    pub temperature_1: Option<f64>,
    pub temperature_2: Option<f64>,
    pub temperature_3: Option<f64>,
    pub soil_moisture_count: Option<f64>,
    pub shake: Option<i32>,
    pub error_flat: Option<i32>,
    pub temperature_average: Option<f64>,
}

impl From<Model> for SensorData {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            instrument_seq: model.instrument_seq,
            time_utc: model.time_utc,
            temperature_1: model.temperature_1,
            temperature_2: model.temperature_2,
            temperature_3: model.temperature_3,
            soil_moisture_count: model.soil_moisture_count,
            shake: model.shake,
            error_flat: model.error_flat,
            temperature_average: model.temperature_average,
        }
    }
}
