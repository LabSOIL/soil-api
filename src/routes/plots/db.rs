use crate::routes::areas::db::Entity as Area;
use crate::routes::samples::db::Entity as PlotSample;
use crate::routes::transects::nodes::db::Entity as TransectNode;
use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "gradientchoices")]
pub enum Gradientchoices {
    #[sea_orm(string_value = "flat")]
    Flat,
    #[sea_orm(string_value = "slope")]
    Slope,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "plot")]
pub struct Model {
    #[sea_orm(unique)]
    pub name: String,
    pub area_id: Uuid,
    pub gradient: Gradientchoices,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    pub created_on: Option<NaiveDate>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub coord_x: f64,
    pub coord_y: f64,
    pub coord_z: f64,
    pub coord_srid: i32,
    pub last_updated: DateTime<Utc>,
    pub image: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Area",
        from = "Column::AreaId",
        to = "crate::routes::areas::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(has_many = "PlotSample")]
    Plotsample,
    #[sea_orm(has_many = "TransectNode")]
    Transectnode,
}

impl Related<Area> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<PlotSample> for Entity {
    fn to() -> RelationDef {
        Relation::Plotsample.def()
    }
}

impl Related<TransectNode> for Entity {
    fn to() -> RelationDef {
        Relation::Transectnode.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
