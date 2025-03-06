use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "instrumentexperiment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub device_filename: Option<String>,
    pub data_source: Option<String>,
    #[allow(clippy::struct_field_names)]
    pub instrument_model: Option<String>,
    pub init_e: Option<f64>,
    pub sample_interval: Option<f64>,
    pub run_time: Option<f64>,
    pub quiet_time: Option<f64>,
    pub sensitivity: Option<f64>,
    pub samples: Option<i32>,
    pub last_updated: DateTime<Utc>,
    pub project_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::routes::instrument_experiments::channels::db::Entity")]
    Instrumentexperimentchannel,
    #[sea_orm(
        belongs_to = "crate::routes::projects::db::Entity",
        from = "Column::ProjectId",
        to = "crate::routes::projects::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Project,
}

impl Related<crate::routes::instrument_experiments::channels::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Instrumentexperimentchannel.def()
    }
}

impl Related<crate::routes::projects::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
