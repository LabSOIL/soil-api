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

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct SensorProfileAssignment {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub sensor_id: Uuid,
    pub sensorprofile_id: Uuid,
    pub date_from: DateTime<Utc>,
    pub date_to: DateTime<Utc>,
    #[crudcrate(
        update_model = false,
        create_model = false,
        on_update = chrono::Utc::now(),
        on_create = chrono::Utc::now()
    )]
    pub last_updated: DateTime<Utc>,
    #[crudcrate(update_model = false, create_model = false)]
    pub sensor_profile: Option<crate::routes::sensors::profile::models::SensorProfile>,
    #[crudcrate(update_model = false, create_model = false)]
    pub sensor: Option<crate::routes::sensors::models::Sensor>,
    #[serde(default)]
    #[crudcrate(non_db_attr = true, default = vec![])]
    pub data: Vec<crate::routes::sensors::data::models::SensorData>,
}

impl From<Model> for SensorProfileAssignment {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            sensor_id: model.sensor_id,
            sensorprofile_id: model.sensorprofile_id,
            date_from: model.date_from,
            date_to: model.date_to,
            last_updated: model.last_updated,
            sensor_profile: None,
            sensor: None,
            data: vec![],
        }
    }
}

#[async_trait]
impl CRUDResource for SensorProfileAssignment {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = SensorProfileAssignment;
    type CreateModel = SensorProfileAssignmentCreate;
    type UpdateModel = SensorProfileAssignmentUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "assignment (sensor profile)";
    const RESOURCE_NAME_PLURAL: &'static str = "assignments (sensor profile)";
    const RESOURCE_DESCRIPTION: &'static str = "This is a record of a sensor being assigned to a sensor profile for a specific time period, should a sensor move or be changed over time. It therefore helps piece the data together for a sensor profile.";

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

        if models.is_empty() {
            return Ok(vec![]);
        }

        let sensor_profile: crate::routes::sensors::profile::models::SensorProfile = models
            .load_one(crate::routes::sensors::profile::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .unwrap()
            .into();

        let sensor: crate::routes::sensors::models::Sensor = models
            .load_one(crate::routes::sensors::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .unwrap()
            .into();

        let models: Vec<Self::ApiModel> = models
            .into_iter()
            .map(|model| {
                let mut model: SensorProfileAssignment = model.into();
                model.sensor_profile = Some(sensor_profile.clone());
                model.sensor = Some(sensor.clone());
                model
            })
            .collect();
        Ok(models)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .all(db)
            .await?;

        let sensor_profile: crate::routes::sensors::profile::models::SensorProfile = model
            .clone()
            .load_one(crate::routes::sensors::profile::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .unwrap()
            .into();

        let sensor: crate::routes::sensors::models::Sensor = model
            .clone()
            .load_one(crate::routes::sensors::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .unwrap()
            .into();
        let model: Self::ApiModel = model
            .into_iter()
            .map(|model| {
                let mut model: SensorProfileAssignment = model.into();
                model.sensor_profile = Some(sensor_profile.clone());
                model.sensor = Some(sensor.clone());
                model
            })
            .collect::<Vec<Self::ApiModel>>()
            .pop()
            .ok_or(DbErr::RecordNotFound("Record not found".into()))?;
        Ok(model)
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
            ("sensor_id", Self::ColumnType::SensorId),
            ("sensorprofile_id", Self::ColumnType::SensorprofileId),
            ("date_from", Self::ColumnType::DateFrom),
            ("date_to", Self::ColumnType::DateTo),
            ("last_updated", Self::ColumnType::LastUpdated),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![]
    }
}
