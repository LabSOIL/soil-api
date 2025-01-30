use super::db::{ActiveModel, Model};
use crate::common::traits::ApiResource;
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use chrono::NaiveDateTime;
use sea_orm::{
    entity::prelude::*, ActiveValue, Condition, DatabaseConnection, EntityTrait, FromQueryResult,
    NotSet, Order, PaginatorTrait, QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
pub struct Project {
    color: String,
    last_updated: NaiveDateTime,
    description: Option<String>,
    id: Uuid,
    name: String,
}
impl IntoResponse for Project {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

impl From<Model> for Project {
    fn from(model: Model) -> Self {
        Self {
            color: model.color,
            last_updated: model.last_updated,
            description: model.description,
            id: model.id,
            name: model.name,
        }
    }
}

#[async_trait]
impl ApiResource for Project {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Project;
    type CreateModel = ProjectCreate;
    type UpdateModel = ProjectUpdate;

    const RESOURCE_NAME: &'static str = "projects";

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
        Ok(models.into_iter().map(Self::ApiModel::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Project not found".into()))?;
        Ok(Self::ApiModel::from(model))
    }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let active_model: Self::ActiveModelType = create_model.into();
        let inserted = active_model.insert(db).await?;
        Self::get_one(inserted.id, db)
            .await
            .ok_or(DbErr::RecordNotFound("Project not found".into()))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Project not found".into()))?
            .into();

        let updated_model = update_model.merge_into_activemodel(existing);
        let updated = updated_model.update(db).await?;
        Ok(Self::ApiModel::from(updated))
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
            .select_only()
            .count(db)
            .await
            .unwrap_or(0)
    }

    fn default_index_column() -> Self::ColumnType {
        Self::ColumnType::Id
    }

    fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("description", Self::ColumnType::Description),
            ("color", Self::ColumnType::Color),
            ("last_updated", Self::ColumnType::LastUpdated),
        ]
    }

    fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("description", Self::ColumnType::Description),
            ("color", Self::ColumnType::Color),
            ("last_updated", Self::ColumnType::LastUpdated),
        ]
    }
}

impl Project {
    pub async fn get_one(id: Uuid, db: &DatabaseConnection) -> Option<Self> {
        super::db::Entity::find_by_id(id)
            .one(db)
            .await
            .unwrap()
            .map(|model| model.into())
    }
    pub async fn get_all(
        db: DatabaseConnection,
        condition: Condition,
        order_column: super::db::Column,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Vec<Self> {
        super::db::Entity::find()
            .filter(condition.clone())
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(&db)
            .await
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .map(Project::from)
            .collect()
    }
}

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
pub struct ProjectCreate {
    pub color: String,
    pub description: Option<String>,
    pub name: String,
}

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
pub struct ProjectUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub color: Option<Option<String>>,
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
    pub name: Option<Option<String>>,
}

impl ProjectUpdate {
    pub fn merge_into_activemodel(self, mut model: ActiveModel) -> ActiveModel {
        model.color = match self.color {
            Some(Some(color)) => Set(color),
            None => NotSet,
            _ => NotSet,
        };
        model.description = match self.description {
            Some(description) => Set(description),
            None => NotSet,
        };
        model.name = match self.name {
            Some(Some(name)) => Set(name),
            None => NotSet,
            _ => NotSet,
        };
        model
    }
}

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
pub struct ProjectBasic {
    pub id: Uuid,
    pub name: String,
}

impl From<ProjectCreate> for super::db::ActiveModel {
    fn from(project: ProjectCreate) -> Self {
        super::db::ActiveModel {
            color: ActiveValue::set(project.color),
            description: ActiveValue::set(project.description),
            name: ActiveValue::set(project.name),
            id: ActiveValue::NotSet,
            last_updated: ActiveValue::NotSet,
            // iterator: ActiveValue::NotSet,
        }
    }
}
