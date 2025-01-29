use super::db::{ActiveModel, Model};
use crate::common::traits::ApiResource;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use sea_orm::{
    entity::prelude::*, ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    FromQueryResult, NotSet, Order, PaginatorTrait, QueryOrder, QuerySelect, Set,
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
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Project;

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: super::db::Column,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Vec<Self::ApiModel> {
        Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Self::ApiModel::from)
            .collect()
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Option<Self::ApiModel> {
        Self::EntityType::find_by_id(id)
            .one(db)
            .await
            .unwrap()
            .map(Self::ApiModel::from)
    }

    async fn create(
        db: &DatabaseConnection,
        active_model: Self::ActiveModelType,
    ) -> Result<Self::ApiModel, DbErr> {
        let res = active_model.insert(db).await?;
        Ok(Self::ApiModel::from(res))
    }

    async fn update(
        db: &DatabaseConnection,
        active_model: Self::ActiveModelType,
    ) -> Result<Self::ApiModel, DbErr> {
        let res = active_model.update(db).await?;
        Ok(Self::ApiModel::from(res))
    }

    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
        Self::EntityType::delete_by_id(id)
            .exec(db)
            .await
            .map(|res| res.rows_affected as usize)
    }

    fn default_sort_column() -> &'static str {
        "id"
    }

    fn sortable_columns() -> &'static [(&'static str, super::db::Column)] {
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            ("description", super::db::Column::Description),
            ("color", super::db::Column::Color),
            ("last_updated", super::db::Column::LastUpdated),
        ]
    }

    fn filterable_columns<'a>() -> &'a [(&'a str, impl ColumnTrait)] {
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            ("description", super::db::Column::Description),
            ("color", super::db::Column::Color),
            ("last_updated", super::db::Column::LastUpdated),
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
            _ => NotSet,
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
