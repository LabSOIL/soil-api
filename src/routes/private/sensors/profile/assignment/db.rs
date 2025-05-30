use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "sensorprofile_assignment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub sensor_id: Uuid,
    pub sensorprofile_id: Uuid,
    pub date_from: DateTime<Utc>,
    pub date_to: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub depth_cm_sensor1: i32,
    pub depth_cm_sensor2: i32,
    pub depth_cm_sensor3: i32,
    pub depth_cm_moisture: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::routes::private::sensors::db::Entity",
        from = "Column::SensorId",
        to = "crate::routes::private::sensors::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Sensor,
    #[sea_orm(
        belongs_to = "crate::routes::private::sensors::profile::db::Entity",
        from = "Column::SensorprofileId",
        to = "crate::routes::private::sensors::profile::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Sensorprofile,
}

impl Related<crate::routes::private::sensors::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensor.def()
    }
}

impl Related<crate::routes::private::sensors::profile::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensorprofile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
