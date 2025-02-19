use crate::areas::db::Entity as Area;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "sensor")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub comment: Option<String>,
    pub area_id: Uuid,
    pub last_updated: NaiveDateTime,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Area",
        from = "Column::AreaId",
        to = "crate::areas::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(has_many = "crate::plots::sensors::db::Entity")]
    Plotsensorassignments,
    #[sea_orm(has_many = "crate::sensors::data::db::Entity")]
    Sensordata,
}

impl Related<Area> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<crate::plots::sensors::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plotsensorassignments.def()
    }
}

impl Related<crate::sensors::data::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensordata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
