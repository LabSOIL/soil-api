use super::db::Model;
use chrono::{DateTime, Utc};
use crudcrate::{ToCreateModel, ToUpdateModel};
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, ToSchema, Serialize, Deserialize, Debug, ToCreateModel, ToUpdateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct SensorData {
    pub instrument_seq: i32,
    pub temperature_1: f64,
    pub temperature_2: f64,
    pub temperature_3: f64,
    pub soil_moisture_count: i32,
    pub shake: i32,
    pub error_flat: i32,
    pub sensor_id: Uuid,
    pub time_utc: DateTime<Utc>,
    pub temperature_average: f64,
}

impl From<Model> for SensorData {
    fn from(model: Model) -> Self {
        Self {
            sensor_id: model.sensor_id,
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
