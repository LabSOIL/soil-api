use crate::routes::private::plots::db::Entity as Plot;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "transectnode")]
pub struct Model {
    pub plot_id: Uuid,
    pub transect_id: Uuid,
    pub order: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Plot",
        from = "Column::PlotId",
        to = "crate::routes::private::plots::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Plot,
    #[sea_orm(
        belongs_to = "crate::routes::private::transects::db::Entity",
        from = "Column::TransectId",
        to = "crate::routes::private::transects::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Transect,
}

impl Related<Plot> for Entity {
    fn to() -> RelationDef {
        Relation::Plot.def()
    }
}

impl Related<crate::routes::private::transects::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transect.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
