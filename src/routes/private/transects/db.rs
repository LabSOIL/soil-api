use crate::routes::private::areas::db::Entity as Area;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "transect")]
pub struct Model {
    pub name: String,
    pub description: Option<String>,
    pub date_created: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub area_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Area",
        from = "Column::AreaId",
        to = "crate::routes::private::areas::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(has_many = "crate::routes::private::transects::nodes::db::Entity")]
    Transectnode,
}

impl Related<Area> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<crate::routes::private::transects::nodes::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transectnode.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
