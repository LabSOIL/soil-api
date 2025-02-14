use super::db::Model;
use crate::common::crud::traits::CRUDResource;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use crudcrate::{ToCreateModel, ToUpdateModel};
use rand::Rng;
use sea_orm::{
    entity::prelude::*, ActiveValue, Condition, DatabaseConnection, EntityTrait, FromQueryResult,
    Order, PaginatorTrait, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(ToSchema, Serialize, Deserialize, FromQueryResult, ToUpdateModel, ToCreateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct Project {
    color: String,
    #[crudcrate(update = false, create = false)]
    last_updated: NaiveDateTime,
    description: Option<String>,
    #[crudcrate(update = false, create = false)]
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
impl CRUDResource for Project {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Project;
    type CreateModel = ProjectCreate;
    type UpdateModel = ProjectUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_PLURAL: &'static str = "projects";
    const RESOURCE_NAME_SINGULAR: &'static str = "project";

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
        let model =
            Self::EntityType::find_by_id(id)
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound(
                    format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
                ))?;
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
            .ok_or(DbErr::RecordNotFound(
                format!("{} not created", Self::RESOURCE_NAME_SINGULAR).into(),
            ))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_PLURAL).into(),
            ))?
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
// #[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
// pub struct ProjectCreate {
//     pub color: Option<String>,
//     pub description: Option<String>,
//     pub name: String,
// }

// #[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
// pub struct ProjectUpdate {
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub color: Option<Option<String>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub description: Option<Option<String>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub name: Option<Option<String>>,
// }

// impl ProjectUpdate {
//     pub fn merge_into_activemodel(self, mut model: ActiveModel) -> ActiveModel {
//         model.color = match self.color {
//             Some(Some(color)) => Set(color),
//             None => NotSet,
//             _ => NotSet,
//         };
//         model.description = match self.description {
//             Some(description) => Set(description),
//             None => NotSet,
//         };
//         model.name = match self.name {
//             Some(Some(name)) => Set(name),
//             None => NotSet,
//             _ => NotSet,
//         };
//         model
//     }
// }

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
pub struct ProjectBasic {
    pub id: Uuid,
    pub name: String,
}

// impl From<ProjectCreate> for super::db::ActiveModel {
//     fn from(project: ProjectCreate) -> Self {
//         // If color is not provided, generate a random color
//         let color = project.color.unwrap_or_else(|| {
//             let mut rng = rand::rng();
//             format!("#{:06x}", rng.random::<u32>() & 0xFFFFFF)
//         });
//         super::db::ActiveModel {
//             color: ActiveValue::set(color),
//             description: ActiveValue::set(project.description),
//             name: ActiveValue::set(project.name),
//             id: ActiveValue::set(Uuid::new_v4()),
//             last_updated: ActiveValue::set(chrono::Utc::now().naive_utc()),
//         }
//     }
// }
