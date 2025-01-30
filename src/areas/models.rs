use crate::plots::models::PlotSimple;
use crate::projects::db::Entity as ProjectDB;
use crate::projects::models::Project;
use crate::soil::profiles::models::SoilProfile;
use crate::transects::models::Transect;
use crate::{areas::db::ActiveModel as AreaActiveModel, sensors::models::SensorSimple};
use chrono::NaiveDateTime;
use sea_orm::{
    entity::prelude::*, query::*, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    EntityTrait, NotSet, Order, QueryOrder, Set,
};
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

impl Area {
    pub async fn get_one(area_id: Uuid, db: DatabaseConnection) -> Self {
        let obj = super::db::Entity::find_by_id(area_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        let project = obj
            .find_related(crate::projects::db::Entity)
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        let plots = obj
            .find_related(crate::plots::db::Entity)
            .all(&db)
            .await
            .unwrap();

        let sensors = obj
            .find_related(crate::sensors::db::Entity)
            .all(&db)
            .await
            .unwrap();

        let soil_profiles = obj
            .find_related(crate::soil::profiles::db::Entity)
            .all(&db)
            .await
            .unwrap();

        let transects = obj
            .find_related(crate::transects::db::Entity)
            .all(&db)
            .await
            .unwrap();

        let convex_hull = super::services::get_convex_hull(&db, obj.id).await;

        let area = super::models::Area {
            geom: convex_hull,
            last_updated: obj.last_updated,
            project_id: obj.project_id,
            id: obj.id,
            name: obj.name,
            description: obj.description,
            project: project.into(),
            plots: plots.into_iter().map(Into::into).collect(),
            sensors: sensors.into_iter().map(Into::into).collect(),
            soil_profiles: soil_profiles.into_iter().map(Into::into).collect(),
            transects: transects.into_iter().map(Into::into).collect(),
        };
        area
    }
}

impl Area {
    pub async fn get_all(
        db: DatabaseConnection,
        condition: Condition,
        order_column: super::db::Column,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Vec<Self> {
        // let objs = super::db::Entity::find().all(&db).await.unwrap();
        let objs = super::db::Entity::find()
            .filter(condition.clone())
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(&db)
            .await
            .unwrap();
        let mut areas = Vec::new();
        for obj in objs {
            let project = obj
                .find_related(crate::projects::db::Entity)
                .one(&db)
                .await
                .unwrap()
                .unwrap();

            let plots = obj
                .find_related(crate::plots::db::Entity)
                .all(&db)
                .await
                .unwrap();

            let sensors = obj
                .find_related(crate::sensors::db::Entity)
                .all(&db)
                .await
                .unwrap();

            let soil_profiles = obj
                .find_related(crate::soil::profiles::db::Entity)
                .all(&db)
                .await
                .unwrap();

            let transects = obj
                .find_related(crate::transects::db::Entity)
                .all(&db)
                .await
                .unwrap();

            let convex_hull = super::services::get_convex_hull(&db, obj.id).await;

            let area = super::models::Area {
                geom: convex_hull,
                last_updated: obj.last_updated,
                project_id: obj.project_id,
                id: obj.id,
                name: obj.name,
                description: obj.description,
                project: project.into(),
                plots: plots.into_iter().map(Into::into).collect(),
                sensors: sensors.into_iter().map(Into::into).collect(),
                soil_profiles: soil_profiles.into_iter().map(Into::into).collect(),
                transects: transects.into_iter().map(Into::into).collect(),
            };
            areas.push(area);
        }
        areas
    }
}
#[derive(ToSchema, Serialize, Deserialize)]
pub struct AreaCreate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_id: Uuid,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct AreaUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub description: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub project_id: Option<Option<Uuid>>,
}

impl AreaUpdate {
    pub fn merge_into_activemodel(self, mut model: AreaActiveModel) -> AreaActiveModel {
        model.name = match self.name {
            Some(Some(value)) => Set(Some(value)),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.description = match self.description {
            Some(Some(value)) => Set(Some(value)),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.project_id = match self.project_id {
            Some(Some(value)) => Set(value),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model
    }
}

#[derive(ToSchema, Serialize)]
pub struct AreaBasicWithProject {
    pub id: Uuid,
    pub name: Option<String>,
    pub project: crate::common::crud::models::GenericNameAndID,
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
            project: crate::common::crud::models::GenericNameAndID {
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
