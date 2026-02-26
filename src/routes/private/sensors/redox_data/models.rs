use super::db::Model;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    Order, QueryOrder, QuerySelect, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel, Debug, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct RedoxData {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub sensorprofile_id: Uuid,
    pub measured_on: DateTime<Utc>,
    pub ch1_5cm_mv: Option<f64>,
    pub ch2_15cm_mv: Option<f64>,
    pub ch3_25cm_mv: Option<f64>,
    pub ch4_35cm_mv: Option<f64>,
    pub temp_c: Option<f64>,
}

impl From<Model> for RedoxData {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            sensorprofile_id: model.sensorprofile_id,
            measured_on: model.measured_on,
            ch1_5cm_mv: model.ch1_5cm_mv,
            ch2_15cm_mv: model.ch2_15cm_mv,
            ch3_25cm_mv: model.ch3_25cm_mv,
            ch4_35cm_mv: model.ch4_35cm_mv,
            temp_c: model.temp_c,
        }
    }
}

#[async_trait]
impl CRUDResource for RedoxData {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ActiveModelType = super::db::ActiveModel;
    type CreateModel = RedoxDataCreate;
    type UpdateModel = RedoxDataUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "redox data";
    const RESOURCE_NAME_PLURAL: &'static str = "redox data records";
    const RESOURCE_DESCRIPTION: &'static str =
        "Redox potential measurements at multiple depths (5, 15, 25, 35 cm).";

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
        Ok(models.into_iter().map(RedoxData::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "{} not found",
                Self::RESOURCE_NAME_SINGULAR
            )))?;
        Ok(RedoxData::from(model))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self, DbErr> {
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
            ("measured_on", Self::ColumnType::MeasuredOn),
            ("sensorprofile_id", Self::ColumnType::SensorprofileId),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("sensorprofile_id", Self::ColumnType::SensorprofileId),
        ]
    }
}
