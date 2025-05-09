use crate::routes::private::areas::db::Entity as Area;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "soilclassification")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub area_id: Uuid,
    pub soil_type_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub depth_upper_cm: i32,
    pub depth_lower_cm: i32,
    pub created_on: DateTime<Utc>,
    pub sample_date: Option<chrono::NaiveDate>,
    pub last_updated: DateTime<Utc>,
    pub fe_abundance_g_per_cm3: Option<f64>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    pub coord_srid: Option<i32>,
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
