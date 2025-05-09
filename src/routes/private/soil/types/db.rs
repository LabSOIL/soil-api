use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "soiltype")]
pub struct Model {
    pub name: String,
    pub description: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub last_updated: DateTime<Utc>,
    pub image: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::routes::private::soil::profiles::db::Entity")]
    Soilprofile,
    #[sea_orm(has_many = "crate::routes::private::soil::classification::db::Entity")]
    SoilClassification,
}

impl Related<crate::routes::private::soil::profiles::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Soilprofile.def()
    }
}

impl Related<crate::routes::private::soil::classification::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SoilClassification.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
