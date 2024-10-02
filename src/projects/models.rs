use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use sea_orm::{entity::prelude::*, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait};
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

impl Project {
    pub async fn from_area(area: &crate::areas::db::Model, db: &DatabaseConnection) -> Self {
        super::db::Entity::find()
            .filter(super::db::Column::Id.eq(area.project_id))
            .into_model::<Project>()
            .one(db)
            .await
            .unwrap()
            .unwrap()
    }

    pub async fn from_db(project: crate::projects::db::Model, db: &DatabaseConnection) -> Self {
        super::db::Entity::find()
            .filter(super::db::Column::Id.eq(project.id))
            .into_model::<Project>()
            .one(db)
            .await
            .unwrap()
            .unwrap()
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
    pub color: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
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
            iterator: ActiveValue::NotSet,
        }
    }
}
