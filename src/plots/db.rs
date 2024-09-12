use super::sea_orm_active_enums::Gradientchoices;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use geo_types::Point;
use geozero::wkb;
use geozero::wkb::Wkb;
// use geozero::wkb::Ewkb;
// use geozero::wkb::FromWkb;
use geozero::wkb::{Ewkb, FromWkb};
use geozero::wkt::Wkt;
use sea_orm::entity::prelude::*;
use serde::Serialize;
use std::convert::TryFrom;
use utoipa::ToSchema;
use uuid::Uuid;

// impl TryFrom<Option<Ewkb>> for Geom {
//     type Error = geozero::error::Error;

//     fn try_from(value: Option<Ewkb>) -> Result<Self, Self::Error> {
//         match value {
//             Some(ewkb) => Ok(Self(Some(Point::from_wkb(&ewkb)?))),
//             None => Ok(Self(None)),
//         }
//     }
// }

// impl From<Geom> for Option<Ewkb> {
//     fn from(value: Geom) -> Self {
//         match value.0 {
//             Some(point) => Some(Ewkb::from(&point)),
//             None => None,
//         }
//     }
// }
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
    pub geom: Option<String>,
    pub last_updated: NaiveDateTime,
    pub image: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::area::Entity",
        from = "Column::AreaId",
        to = "super::area::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(has_many = "super::plotsample::Entity")]
    Plotsample,
    #[sea_orm(has_many = "super::plotsensorassignments::Entity")]
    Plotsensorassignments,
    #[sea_orm(has_many = "super::transectnode::Entity")]
    Transectnode,
}

impl Related<super::area::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<super::plotsample::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plotsample.def()
    }
}

impl Related<super::plotsensorassignments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plotsensorassignments.def()
    }
}

impl Related<super::transectnode::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transectnode.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
