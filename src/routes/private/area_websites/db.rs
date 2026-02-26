use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "area_website")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub area_id: Uuid,
    pub website_id: Uuid,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::routes::private::areas::db::Entity",
        from = "Column::AreaId",
        to = "crate::routes::private::areas::db::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Area,
    #[sea_orm(
        belongs_to = "crate::routes::private::websites::db::Entity",
        from = "Column::WebsiteId",
        to = "crate::routes::private::websites::db::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Website,
}

impl Related<crate::routes::private::areas::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<crate::routes::private::websites::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Website.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
