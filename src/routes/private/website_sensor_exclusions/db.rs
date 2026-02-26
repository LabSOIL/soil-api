use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "website_sensor_exclusion")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub website_id: Uuid,
    pub sensorprofile_id: Uuid,
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
        belongs_to = "crate::routes::private::sensors::profile::db::Entity",
        from = "Column::SensorprofileId",
        to = "crate::routes::private::sensors::profile::db::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SensorProfile,
}

impl Related<crate::routes::private::websites::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Website.def()
    }
}

impl Related<crate::routes::private::sensors::profile::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SensorProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
