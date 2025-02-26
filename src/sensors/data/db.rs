use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, ToSchema)]
#[sea_orm(table_name = "sensordata")]
pub struct Model {
    pub instrument_seq: i32,
    pub temperature_1: f64,
    pub temperature_2: f64,
    pub temperature_3: f64,
    pub soil_moisture_count: i32,
    pub shake: i32,
    pub error_flat: i32,
    #[sea_orm(primary_key)]
    pub sensor_id: Uuid,
    pub last_updated: DateTime<Utc>,
    #[sea_orm(primary_key)]
    pub time_utc: DateTime<Utc>,
    pub temperature_average: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::sensors::db::Entity",
        from = "Column::SensorId",
        to = "crate::sensors::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Sensor,
}

impl Related<crate::sensors::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
