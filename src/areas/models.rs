use super::db::Model;
use crate::common::crud::traits::CRUDResource;
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
    pub project: Option<Project>,
    pub soil_profiles: Vec<SoilProfile>,
    pub plots: Vec<PlotSimple>,
    pub sensors: Vec<SensorSimple>,
    pub transects: Vec<Transect>,
    pub geom: Option<Value>,
}

impl From<Model> for Area {
    fn from(model: Model) -> Self {
        Area {
            id: model.id,
            last_updated: model.last_updated,
            name: model.name,
            description: model.description,
            project_id: model.project_id,
            project: None,
            soil_profiles: vec![],
            plots: vec![],
            sensors: vec![],
            transects: vec![],
            geom: None,
        }
    }
}

#[async_trait::async_trait]
impl CRUDResource for Area {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Area;
    type CreateModel = AreaCreate;
    type UpdateModel = AreaUpdate;

    const RESOURCE_NAME_PLURAL: &'static str = "areas";
    const RESOURCE_NAME_SINGULAR: &'static str = "area";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        let models = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        let mut areas = Vec::new();
        for model in models {
            let project = model.find_related(ProjectDB).one(db).await?.unwrap();

            let plots = model.find_related(crate::plots::db::Entity).all(db).await?;

            let sensors = model
                .find_related(crate::sensors::db::Entity)
                .all(db)
                .await?;

            let soil_profiles = model
                .find_related(crate::soil::profiles::db::Entity)
                .all(db)
                .await?;

            let transects = model
                .find_related(crate::transects::db::Entity)
                .all(db)
                .await?;

            let convex_hull = super::services::get_convex_hull(db, model.id).await;

            let area = Area {
                geom: convex_hull,
                last_updated: model.last_updated,
                project_id: model.project_id,
                id: model.id,
                name: model.name,
                description: model.description,
                project: Some(project.into()),
                plots: plots.into_iter().map(Into::into).collect(),
                sensors: sensors.into_iter().map(Into::into).collect(),
                soil_profiles: soil_profiles.into_iter().map(Into::into).collect(),
                transects: transects.into_iter().map(Into::into).collect(),
            };
            areas.push(area);
        }
        Ok(areas)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Area not found".into()))?;

        let project = model.find_related(ProjectDB).one(db).await?.unwrap();

        let plots = model.find_related(crate::plots::db::Entity).all(db).await?;

        let sensors = model
            .find_related(crate::sensors::db::Entity)
            .all(db)
            .await?;

        let soil_profiles = model
            .find_related(crate::soil::profiles::db::Entity)
            .all(db)
            .await?;

        let transects = model
            .find_related(crate::transects::db::Entity)
            .all(db)
            .await?;

        let convex_hull = super::services::get_convex_hull(db, model.id).await;

        let area = Area {
            geom: convex_hull,
            last_updated: model.last_updated,
            project_id: model.project_id,
            id: model.id,
            name: model.name,
            description: model.description,
            project: Some(project.into()),
            plots: plots.into_iter().map(Into::into).collect(),
            sensors: sensors.into_iter().map(Into::into).collect(),
            soil_profiles: soil_profiles.into_iter().map(Into::into).collect(),
            transects: transects.into_iter().map(Into::into).collect(),
        };
        Ok(area)
    }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let active_model: Self::ActiveModelType = create_model.into();
        let inserted = active_model.insert(db).await?;
        let area = Self::get_one(inserted.id, db.clone()).await;
        Ok(area)
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Area not found".into()))?
            .into();

        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let area = Self::get_one(response_obj.id, db.clone()).await;
        Ok(area)
    }

    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
        let res = Self::EntityType::delete_by_id(id).exec(db).await?;
        Ok(res.rows_affected as usize)
    }

    async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr> {
        Self::EntityType::delete_many()
            .filter(Self::ColumnType::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(ids)
    }

    async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64 {
        Self::EntityType::find()
            .filter(condition)
            .count(db)
            .await
            .unwrap()
    }

    fn default_index_column() -> Self::ColumnType {
        super::db::Column::Id
    }

    fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
        ]
    }

    fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            ("project_id", super::db::Column::ProjectId),
        ]
    }
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
            project: Some(project.into()),
            plots: plots.into_iter().map(Into::into).collect(),
            sensors: sensors.into_iter().map(Into::into).collect(),
            soil_profiles: soil_profiles.into_iter().map(Into::into).collect(),
            transects: transects.into_iter().map(Into::into).collect(),
        };
        area
    }

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
                project: Some(project.into()),
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
            id: ActiveValue::Set(Uuid::new_v4()),
            last_updated: ActiveValue::NotSet,
        }
    }
}
