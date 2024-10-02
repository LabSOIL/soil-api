use crate::areas::db::Entity as Area;
use crate::plots::sensors::db::Entity as PlotSensorAssignments;
use crate::samples::db::Entity as PlotSample;
use crate::transects::nodes::db::Entity as TransectNode;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
// use geozero::wkb;
use sea_orm::entity::prelude::*;
use sea_orm::EntityTrait;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "gradientchoices")]
pub enum Gradientchoices {
    #[sea_orm(string_value = "flat")]
    Flat,
    #[sea_orm(string_value = "slope")]
    Slope,
}

// #[derive(sqlx::Type)]
// #[sqlx(transparent)]
// struct MyInt4(i32);
// use crate::common::db::types::PointZ;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "plot")]
pub struct Model {
    #[sea_orm(unique)]
    pub name: String,
    pub plot_iterator: i32,
    pub area_id: Uuid,
    pub gradient: Gradientchoices,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    pub created_on: Option<NaiveDate>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    #[sea_orm(primary_key)]
    pub iterator: i32,
    #[sea_orm(unique)]
    pub id: Uuid,
    // pub geom: Option<String>,
    // pub geom: PointZ,
    pub last_updated: NaiveDateTime,
    pub image: Option<String>,
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
    #[sea_orm(has_many = "PlotSample")]
    Plotsample,
    #[sea_orm(has_many = "PlotSensorAssignments")]
    Plotsensorassignments,
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

impl Related<PlotSensorAssignments> for Entity {
    fn to() -> RelationDef {
        Relation::Plotsensorassignments.def()
    }
}

impl Related<TransectNode> for Entity {
    fn to() -> RelationDef {
        Relation::Transectnode.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
