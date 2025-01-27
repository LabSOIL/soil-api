use crate::plots::models::PlotSimple;
use crate::projects::db::Entity as ProjectDB;
use crate::projects::models::Project;
use crate::soil::profiles::models::SoilProfile;
use crate::transects::models::Transect;
use crate::{areas::db::ActiveModel as AreaActiveModel, sensors::models::SensorSimple};
use chrono::NaiveDateTime;
use sea_orm::{entity::prelude::*, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct Area {
    pub id: Uuid,
    pub last_updated: NaiveDateTime,
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_id: Uuid,
    pub project: Project,
    pub soil_profiles: Vec<SoilProfile>,
    pub plots: Vec<PlotSimple>,
    pub sensors: Vec<SensorSimple>,
    pub transects: Vec<Transect>,
    pub geom: Option<Value>,
}
#[derive(ToSchema, Serialize, Deserialize)]
pub struct AreaCreate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_id: Uuid,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct AreaUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_id: Uuid,
}

#[derive(ToSchema, Serialize)]
pub struct AreaBasicWithProject {
    pub id: Uuid,
    pub name: Option<String>,
    pub project: crate::common::models::GenericNameAndID,
}

impl AreaBasicWithProject {
    pub async fn from(area_id: Uuid, db: DatabaseConnection) -> Self {
        let area = super::db::Entity::find()
            .filter(crate::areas::db::Column::Id.eq(area_id))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        let project = ProjectDB::find()
            .filter(crate::projects::db::Column::Id.eq(area.project_id))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        AreaBasicWithProject {
            id: area_id,
            name: area.name,
            project: crate::common::models::GenericNameAndID {
                id: project.id,
                name: project.name,
            },
        }
    }
}

impl From<AreaCreate> for AreaActiveModel {
    fn from(area_create: AreaCreate) -> Self {
        AreaActiveModel {
            name: ActiveValue::Set(area_create.name),
            description: ActiveValue::Set(area_create.description),
            project_id: ActiveValue::Set(area_create.project_id),
            // iterator: ActiveValue::NotSet,
            id: ActiveValue::Set(Uuid::new_v4()),
            last_updated: ActiveValue::NotSet,
        }
    }
}

impl Area {
    pub async fn from_db(area: super::db::Model, db: &DatabaseConnection) -> Self {
        let plots: Vec<PlotSimple> = PlotSimple::from_area(&area, db).await;
        let sensors: Vec<SensorSimple> = SensorSimple::from_area(&area, db).await;
        let transects: Vec<Transect> = Transect::from_area(&area, db).await;
        let soil_profiles: Vec<SoilProfile> = SoilProfile::from_area(&area, db).await;
        let project: Project = Project::from_area(&area, db).await;

        // Fetch convex hull geom for the area
        let geom: Option<Value> = crate::areas::services::get_convex_hull(&db, area.id).await;

        Area {
            id: area.id,
            name: area.name,
            description: area.description,
            project_id: area.project_id,
            last_updated: area.last_updated,
            plots,
            soil_profiles,
            sensors,
            transects, // Include transects with nodes
            project,
            geom,
        }
    }
}

impl From<AreaUpdate> for super::db::ActiveModel {
    fn from(area: AreaUpdate) -> Self {
        super::db::ActiveModel {
            name: ActiveValue::Set(area.name),
            description: ActiveValue::Set(area.description),
            project_id: ActiveValue::Set(area.project_id),
            id: ActiveValue::NotSet,
            last_updated: ActiveValue::NotSet,
            // iterator: ActiveValue::NotSet,
        }
    }
}

impl super::db::ActiveModel {
    pub fn merge(self, other: Self) -> Self {
        super::db::ActiveModel {
            name: match other.name {
                ActiveValue::Set(value) => ActiveValue::Set(value),
                _ => self.name,
            },
            description: match other.description {
                ActiveValue::Set(value) => ActiveValue::Set(value),
                _ => self.description,
            },
            project_id: match other.project_id {
                ActiveValue::Set(value) => ActiveValue::Set(value),
                _ => self.project_id,
            },
            // Keep all other fields unchanged if not set in `other`
            id: self.id,
            last_updated: self.last_updated,
            // iterator: self.iterator,
        }
    }
}
