use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, ToSchema)]
#[sea_orm(table_name = "redox_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub sensorprofile_id: Uuid,
    pub measured_on: DateTime<Utc>,
    pub ch1_5cm_mv: Option<f64>,
    pub ch2_15cm_mv: Option<f64>,
    pub ch3_25cm_mv: Option<f64>,
    pub ch4_35cm_mv: Option<f64>,
    pub temp_c: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::routes::private::sensors::profile::db::Entity",
        from = "Column::SensorprofileId",
        to = "crate::routes::private::sensors::profile::db::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Sensorprofile,
}

impl Related<crate::routes::private::sensors::profile::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensorprofile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
