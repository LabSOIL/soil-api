use crate::routes::private::areas::db::Entity as Area;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde_json::Value as Json;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "soilprofile")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub gradient: String,
    pub description_horizon: Option<Json>,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    pub created_on: Option<DateTime<Utc>>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    pub coord_srid: i32,
    pub coord_x: f64,
    pub coord_y: f64,
    pub coord_z: f64,
    pub last_updated: DateTime<Utc>,
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
        to = "crate::routes::private::areas::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(
        belongs_to = "crate::routes::private::soil::types::db::Entity",
        from = "Column::SoilTypeId",
        to = "crate::routes::private::soil::types::db::Column::Id",
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

impl Related<crate::routes::private::soil::types::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Soiltype.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
