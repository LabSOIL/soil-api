//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "soiltype")]
pub struct Model {
    pub name: String,
    pub description: String,
    #[sea_orm(primary_key)]
    pub iterator: i32,
    #[sea_orm(unique)]
    pub id: Uuid,
    pub last_updated: NaiveDateTime,
    pub image: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::soilprofile::Entity")]
    Soilprofile,
}

impl Related<super::soilprofile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Soilprofile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
