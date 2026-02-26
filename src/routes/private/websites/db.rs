use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "website")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub url: Option<String>,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::routes::private::area_websites::db::Entity")]
    AreaWebsite,
    #[sea_orm(has_many = "crate::routes::private::website_plot_exclusions::db::Entity")]
    WebsitePlotExclusion,
    #[sea_orm(has_many = "crate::routes::private::website_sensor_exclusions::db::Entity")]
    WebsiteSensorExclusion,
}

impl Related<crate::routes::private::area_websites::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AreaWebsite.def()
    }
}

impl Related<crate::routes::private::website_plot_exclusions::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WebsitePlotExclusion.def()
    }
}

impl Related<crate::routes::private::website_sensor_exclusions::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WebsiteSensorExclusion.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
