use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sensordata")]
pub struct Model {
    pub instrument_seq: i32,
    pub time_zone: Option<i32>,
    #[sea_orm(column_type = "Double", nullable)]
    pub temperature_1: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub temperature_2: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub temperature_3: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub soil_moisture_count: Option<f64>,
    pub shake: Option<i32>,
    pub error_flat: Option<i32>,
    #[sea_orm(unique)]
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub sensor_id: Uuid,
    pub last_updated: NaiveDateTime,
    pub time_utc: NaiveDateTime,
    #[sea_orm(column_type = "Double", nullable)]
    pub temperature_average: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::sensors::models::Entity",
        from = "Column::SensorId",
        to = "crate::sensors::models::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Sensor,
}

impl Related<crate::sensors::models::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
