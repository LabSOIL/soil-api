use super::db::Model;
use async_trait::async_trait;
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use sea_orm::{
    ActiveValue, Condition, DatabaseConnection, EntityTrait, FromQueryResult, Order, QueryOrder,
    QuerySelect, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    ToSchema, Serialize, Deserialize, FromQueryResult, ToUpdateModel, ToCreateModel, Clone,
)]
#[active_model = "super::db::ActiveModel"]
pub struct WebsiteSensorExclusion {
    #[crudcrate(update_model = false, on_create = Uuid::new_v4())]
    id: Uuid,
    #[crudcrate(update_model = false)]
    website_id: Uuid,
    #[crudcrate(update_model = false)]
    sensorprofile_id: Uuid,
}

impl From<Model> for WebsiteSensorExclusion {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            website_id: model.website_id,
            sensorprofile_id: model.sensorprofile_id,
        }
    }
}

#[async_trait]
impl CRUDResource for WebsiteSensorExclusion {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ActiveModelType = super::db::ActiveModel;
    type CreateModel = WebsiteSensorExclusionCreate;
    type UpdateModel = WebsiteSensorExclusionUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_PLURAL: &'static str = "website_sensor_exclusions";
    const RESOURCE_NAME_SINGULAR: &'static str = "website_sensor_exclusion";
    const RESOURCE_DESCRIPTION: &'static str = "Excludes specific sensor profiles from a website.";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self>, DbErr> {
        let models = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        Ok(models.into_iter().map(Self::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let model =
            Self::EntityType::find_by_id(id)
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound(format!(
                    "{} not found",
                    Self::RESOURCE_NAME_SINGULAR
                )))?;
        Ok(Self::from(model))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_data: Self::UpdateModel,
    ) -> Result<Self, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "{} not found",
                Self::RESOURCE_NAME_PLURAL
            )))?
            .into();

        let updated_model = update_data.merge_into_activemodel(existing);
        let updated = updated_model.update(db).await?;
        Ok(Self::from(updated))
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", Self::ColumnType::Id),
            ("website_id", Self::ColumnType::WebsiteId),
            ("sensorprofile_id", Self::ColumnType::SensorprofileId),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("website_id", Self::ColumnType::WebsiteId),
            ("sensorprofile_id", Self::ColumnType::SensorprofileId),
        ]
    }
}
