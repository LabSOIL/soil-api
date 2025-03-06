use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "sensorprofile")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub area_id: Uuid,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    pub coord_srid: Option<i32>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::routes::areas::db::Entity",
        from = "Column::AreaId",
        to = "crate::routes::areas::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(has_many = "crate::routes::sensors::profile::assignment::db::Entity")]
    SensorprofileAssignment,
}

impl Related<crate::routes::areas::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<crate::routes::sensors::profile::assignment::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SensorprofileAssignment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
