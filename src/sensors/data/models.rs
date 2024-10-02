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
