use crate::routes::areas::db::Entity as Area;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "Area")]
    Area,
    #[sea_orm(has_many = "crate::routes::instrument_experiments::db::Entity")]
    Instrumentexperiment,
}

impl Related<Area> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<crate::routes::instrument_experiments::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Instrumentexperiment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
