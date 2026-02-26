use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, ToSchema)]
#[sea_orm(table_name = "flux_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub sensorprofile_id: Uuid,
    pub measured_on: DateTime<Utc>,
    pub replicate: String,
    pub setting: Option<String>,
    pub flux_co2_umol_m2_s: Option<f64>,
    pub flux_ch4_nmol_m2_s: Option<f64>,
    pub flux_h2o_umol_m2_s: Option<f64>,
    pub r2_co2: Option<f64>,
    pub r2_ch4: Option<f64>,
    pub r2_h2o: Option<f64>,
    pub swc: Option<f64>,
    pub n_measurements: Option<i32>,
    #[sea_orm(column_type = "Json")]
    pub raw_readings: Option<serde_json::Value>,
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
