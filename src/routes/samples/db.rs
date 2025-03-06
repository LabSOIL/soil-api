use crate::routes::plots::db::Entity as Plot;
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "plotsample")]
pub struct Model {
    pub name: String,
    #[sea_orm(column_type = "Double")]
    pub upper_depth_cm: f64,
    #[sea_orm(column_type = "Double")]
    pub lower_depth_cm: f64,
    pub plot_id: Uuid,
    #[sea_orm(column_type = "Double", nullable)]
    pub sample_weight: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub subsample_weight: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub ph: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub rh: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub loi: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub mfc: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub c: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub n: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub cn: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub clay_percent: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub silt_percent: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub sand_percent: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub fe_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub na_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub al_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub k_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub ca_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub mg_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub mn_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub s_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub cl_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub p_ug_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub si_ug_per_g: Option<f64>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(column_type = "Double", nullable)]
    pub subsample_replica_weight: Option<f64>,
    pub created_on: Option<NaiveDate>,
    pub last_updated: DateTime<Utc>,
    #[sea_orm(column_type = "Double", nullable)]
    pub fungi_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub bacteria_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub archea_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub methanogens_per_g: Option<f64>,
    #[sea_orm(column_type = "Double", nullable)]
    pub methanotrophs_per_g: Option<f64>,
    pub replicate: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Plot",
        from = "Column::PlotId",
        to = "crate::routes::plots::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Plot,
}

impl Related<Plot> for Entity {
    fn to() -> RelationDef {
        Relation::Plot.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
