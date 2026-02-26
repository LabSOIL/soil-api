use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "website_plot_exclusion")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub website_id: Uuid,
    pub plot_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::routes::private::websites::db::Entity",
        from = "Column::WebsiteId",
        to = "crate::routes::private::websites::db::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Website,
    #[sea_orm(
        belongs_to = "crate::routes::private::plots::db::Entity",
        from = "Column::PlotId",
        to = "crate::routes::private::plots::db::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Plot,
}

impl Related<crate::routes::private::websites::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Website.def()
    }
}

impl Related<crate::routes::private::plots::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plot.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
