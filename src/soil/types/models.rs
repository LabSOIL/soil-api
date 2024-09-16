use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

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
    #[sea_orm(has_many = "crate::soil::profiles::models::Entity")]
    Soilprofile,
}

impl Related<crate::soil::profiles::models::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Soilprofile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
