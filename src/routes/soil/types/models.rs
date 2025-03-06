use super::db::Model;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    Order, QueryOrder, QuerySelect, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct SoilType {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_update = Utc::now(), on_create = Utc::now())]
    pub last_updated: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub image: Option<String>,
}

impl From<Model> for SoilType {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: model.name,
            description: model.description,
            image: model.image,
        }
    }
}

#[async_trait]
impl CRUDResource for SoilType {
    type EntityType = crate::routes::soil::types::db::Entity;
    type ColumnType = crate::routes::soil::types::db::Column;
    type ModelType = crate::routes::soil::types::db::Model;
    type ActiveModelType = crate::routes::soil::types::db::ActiveModel;
    type ApiModel = SoilType;
    type CreateModel = SoilTypeCreate;
    type UpdateModel = SoilTypeUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "soil type";
    const RESOURCE_NAME_PLURAL: &'static str = "soil types";
    const RESOURCE_DESCRIPTION: &'static str =
        "A categorisation of soil that is associated to a soil profile.";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        let mut models = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;

        // For get all, don't return the image attribute
        for model in &mut models {
            model.image = None;
        }

        Ok(models.into_iter().map(SoilType::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Soil type not found".into()))?;
        Ok(SoilType::from(model))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "{} not found",
                Self::RESOURCE_NAME_SINGULAR
            )))?
            .into();

        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let obj = Self::get_one(db, response_obj.id).await?;
        Ok(obj)
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("last_updated", Self::ColumnType::LastUpdated),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", Self::ColumnType::Name),
            ("description", Self::ColumnType::Description),
        ]
    }
}
