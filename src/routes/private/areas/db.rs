use crate::routes::private::plots::db::Entity as Plot;
use crate::routes::private::projects::db::Entity as Project;
use crate::routes::private::sensors::profile::db::Entity as SensorProfile;
use crate::routes::private::soil::profiles::db::Entity as SoilProfile;
use crate::routes::private::transects::db::Entity as Transect;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "area")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_id: Uuid,
    pub last_updated: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "Plot")]
    Plot,
    #[sea_orm(
        belongs_to = "Project",
        from = "Column::ProjectId",
        to = "crate::routes::private::projects::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Project,
    #[sea_orm(has_many = "SensorProfile")]
    SensorProfile,
    #[sea_orm(has_many = "SoilProfile")]
    Soilprofile,
    #[sea_orm(has_many = "Transect")]
    Transect,
}

impl Related<Plot> for Entity {
    fn to() -> RelationDef {
        Relation::Plot.def()
    }
}

impl Related<Project> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<SensorProfile> for Entity {
    fn to() -> RelationDef {
        Relation::SensorProfile.def()
    }
}

impl Related<SoilProfile> for Entity {
    fn to() -> RelationDef {
        Relation::Soilprofile.def()
    }
}

impl Related<Transect> for Entity {
    fn to() -> RelationDef {
        Relation::Transect.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
