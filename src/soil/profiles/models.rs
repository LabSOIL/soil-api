use crate::areas::models::Entity as Area;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde_json::Value as Json;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "soilprofile")]
pub struct Model {
    pub name: String,
    pub profile_iterator: i32,
    pub gradient: String,
    pub description_horizon: Option<Json>,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    pub created_on: Option<NaiveDateTime>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    #[sea_orm(primary_key)]
    pub iterator: i32,
    #[sea_orm(unique)]
    pub id: Uuid,
    // #[sea_orm(column_type = "custom(\"geometry\")", nullable)]
    // pub geom: Option<String>,
    pub last_updated: NaiveDateTime,
    pub soil_diagram: Option<String>,
    pub photo: Option<String>,
    #[sea_orm(column_type = "Double", nullable)]
    pub parent_material: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Area",
        from = "Column::AreaId",
        to = "crate::areas::models::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(
        belongs_to = "crate::soil::types::models::Entity",
        from = "Column::SoilTypeId",
        to = "crate::soil::types::models::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Soiltype,
}

impl Related<Area> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<crate::soil::types::models::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Soiltype.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
