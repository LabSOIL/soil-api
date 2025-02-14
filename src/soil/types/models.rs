use super::db::Model;
use crate::common::crud::traits::CRUDResource;
use async_trait::async_trait;
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilType {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description: String,
    pub image: Option<String>,
}

impl From<Model> for SoilType {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: Some(model.name),
            description: model.description,
            image: model.image,
        }
    }
}

impl From<Model> for SoilTypeBasic {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: Some(model.name),
            description: model.description,
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeBasic {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description: String,
}

impl SoilType {
    pub async fn from_db(
        soil_type: crate::soil::types::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let soil_type = crate::soil::types::db::Entity::find()
            .filter(crate::soil::types::db::Column::Id.eq(soil_type.id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        SoilType::from(soil_type)
    }
}

impl SoilTypeBasic {
    pub async fn from_db(
        soil_type: crate::soil::types::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let soil_type = crate::soil::types::db::Entity::find()
            .filter(crate::soil::types::db::Column::Id.eq(soil_type.id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        SoilTypeBasic::from(soil_type)
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeCreate {
    pub name: String,
    pub description: String,
    pub image: Option<String>,
}

impl From<SoilTypeCreate> for crate::soil::types::db::ActiveModel {
    fn from(soil_type: SoilTypeCreate) -> Self {
        crate::soil::types::db::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
            last_updated: sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc()),
            name: sea_orm::ActiveValue::Set(soil_type.name),
            description: sea_orm::ActiveValue::Set(soil_type.description),
            image: sea_orm::ActiveValue::Set(soil_type.image),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

#[async_trait]
impl CRUDResource for SoilType {
    type EntityType = crate::soil::types::db::Entity;
    type ColumnType = crate::soil::types::db::Column;
    type ModelType = crate::soil::types::db::Model;
    type ActiveModelType = crate::soil::types::db::ActiveModel;
    type ApiModel = SoilType;
    type CreateModel = SoilTypeCreate;
    type UpdateModel = SoilTypeUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "soiltype";
    const RESOURCE_NAME_PLURAL: &'static str = "soiltypes";

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

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let active_model: Self::ActiveModelType = create_model.into();
        let inserted = active_model.insert(db).await?;
        Self::get_one(db, inserted.id).await
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        // Find the existing model
        let existing: Self::ActiveModelType = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Soil type not found".into()))?
            .into();
        // Merge update fields (only update if provided)
        let mut active: Self::ActiveModelType = existing;
        if let Some(name) = update_model.name {
            active.name = ActiveValue::Set(name);
        }
        if let Some(description) = update_model.description {
            active.description = ActiveValue::Set(description);
        }
        if let Some(image) = update_model.image {
            active.image = ActiveValue::Set(Some(image));
        }
        active.last_updated = ActiveValue::Set(chrono::Utc::now().naive_utc());
        let updated = active.update(db).await?;
        Self::get_one(db, updated.id).await
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
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
        ]
    }
}
