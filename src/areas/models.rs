use super::db::Model;
use crate::plots::models::Plot;
use crate::projects::db::Entity as ProjectDB;
use crate::projects::models::Project;
use crate::sensors::models::SensorSimple;
use crate::soil::profiles::models::SoilProfile;
use crate::transects::models::Transect;
use chrono::NaiveDateTime;
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, query::*, ActiveValue, Condition, DatabaseConnection, EntityTrait, Order,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, ToCreateModel, ToUpdateModel, Deserialize)]
#[active_model = "super::db::ActiveModel"]
pub struct Area {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now().naive_utc(), on_create = chrono::Utc::now().naive_utc())]
    pub last_updated: NaiveDateTime,
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_id: Uuid,
    #[crudcrate(update_model = false, create_model = false)]
    pub project: Option<Project>,
    #[crudcrate(update_model = false, create_model = false)]
    pub soil_profiles: Vec<SoilProfile>,
    #[crudcrate(update_model = false, create_model = false)]
    pub plots: Vec<Plot>,
    #[crudcrate(update_model = false, create_model = false)]
    pub sensors: Vec<SensorSimple>,
    #[crudcrate(update_model = false, create_model = false)]
    pub transects: Vec<Transect>,
    #[crudcrate(update_model = false, create_model = false)]
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

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
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
        let area = Self::get_one(&db, inserted.id).await.unwrap();
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
        let area = Self::get_one(&db, response_obj.id).await?;
        Ok(area)
    }

    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
        let res = Self::EntityType::delete_by_id(id).exec(db).await?;
        Ok(res.rows_affected as usize)
    }

    fn default_index_column() -> Self::ColumnType {
        super::db::Column::Id
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", super::db::Column::Name),
            ("description", super::db::Column::Description),
        ]
    }
}
