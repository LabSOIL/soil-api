use crate::plots::models::Entity as Plot;
use crate::sensors::models::Entity as Sensor;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "plotsensorassignments")]
pub struct Model {
    pub date_from: NaiveDateTime,
    pub date_to: NaiveDateTime,
    pub plot_id: Uuid,
    pub sensor_id: Uuid,
    #[sea_orm(primary_key)]
    pub iterator: i32,
    #[sea_orm(unique)]
    pub id: Uuid,
    pub depth_cm: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Plot",
        from = "Column::PlotId",
        to = "crate::plots::models::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Plot,
    #[sea_orm(
        belongs_to = "Sensor",
        from = "Column::SensorId",
        to = "crate::sensors::models::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Sensor,
}

impl Related<Plot> for Entity {
    fn to() -> RelationDef {
        Relation::Plot.def()
    }
}

impl Related<Sensor> for Entity {
    fn to() -> RelationDef {
        Relation::Sensor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
