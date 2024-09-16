use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "instrumentexperiment")]
pub struct Model {
    pub name: Option<String>,
    pub date: Option<NaiveDateTime>,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub device_filename: Option<String>,
    pub data_source: Option<String>,
    pub instrument_model: Option<String>,
    #[sea_orm(column_type = "Double", nullable)]
    pub init_e: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub sample_interval: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub run_time: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub quiet_time: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub sensitivity: Option<f64>,
    pub samples: Option<i32>,
    #[sea_orm(primary_key)]
    pub iterator: i32,
    #[sea_orm(unique)]
    pub id: Uuid,
    pub last_updated: NaiveDateTime,
    pub project_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::instrument_experiments::channels::models::Entity")]
    Instrumentexperimentchannel,
    #[sea_orm(
        belongs_to = "crate::projects::models::Entity",
        from = "Column::ProjectId",
        to = "crate::projects::models::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Project,
}

impl Related<crate::instrument_experiments::channels::models::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Instrumentexperimentchannel.def()
    }
}

impl Related<crate::projects::models::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
