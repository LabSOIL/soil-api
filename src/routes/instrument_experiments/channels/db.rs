use sea_orm::entity::prelude::*;
use serde_json::Value as Json;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "instrumentexperimentchannel")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub channel_name: String,
    pub experiment_id: Uuid,
    pub baseline_spline: Option<Json>,
    pub time_values: Option<Json>,
    pub raw_values: Option<Json>,
    pub baseline_values: Option<Json>,
    pub baseline_chosen_points: Option<Json>,
    pub integral_chosen_pairs: Option<Json>,
    pub integral_results: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::routes::instrument_experiments::db::Entity",
        from = "Column::ExperimentId",
        to = "crate::routes::instrument_experiments::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Instrumentexperiment,
}

impl Related<crate::routes::instrument_experiments::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Instrumentexperiment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
