use crate::plots::db::Entity as Plot;
use crate::sensors::db::Entity as Sensor;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "plotsensorassignments")]
pub struct Model {
    pub date_from: DateTime<Utc>,
    pub date_to: DateTime<Utc>,
    pub plot_id: Uuid,
    pub sensor_id: Uuid,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub depth_cm: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Plot",
        from = "Column::PlotId",
        to = "crate::plots::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Plot,
    #[sea_orm(
        belongs_to = "Sensor",
        from = "Column::SensorId",
        to = "crate::sensors::db::Column::Id",
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
