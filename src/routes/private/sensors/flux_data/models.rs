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
pub struct FluxData {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub sensorprofile_id: Uuid,
    pub measured_on: DateTime<Utc>,
    pub replicate: String,
    pub setting: Option<String>,
    pub flux_co2_umol_m2_s: Option<f64>,
    pub flux_ch4_nmol_m2_s: Option<f64>,
    pub flux_h2o_umol_m2_s: Option<f64>,
    pub r2_co2: Option<f64>,
    pub r2_ch4: Option<f64>,
    pub r2_h2o: Option<f64>,
    pub swc: Option<f64>,
    pub n_measurements: Option<i32>,
    #[crudcrate(update_model = false)]
    pub raw_readings: Option<serde_json::Value>,
}

impl From<Model> for FluxData {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            sensorprofile_id: model.sensorprofile_id,
            measured_on: model.measured_on,
            replicate: model.replicate,
            setting: model.setting,
            flux_co2_umol_m2_s: model.flux_co2_umol_m2_s,
            flux_ch4_nmol_m2_s: model.flux_ch4_nmol_m2_s,
            flux_h2o_umol_m2_s: model.flux_h2o_umol_m2_s,
            r2_co2: model.r2_co2,
            r2_ch4: model.r2_ch4,
            r2_h2o: model.r2_h2o,
            swc: model.swc,
            n_measurements: model.n_measurements,
            raw_readings: model.raw_readings,
        }
    }
}

#[async_trait]
impl CRUDResource for FluxData {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ActiveModelType = super::db::ActiveModel;
    type CreateModel = FluxDataCreate;
    type UpdateModel = FluxDataUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "flux data";
    const RESOURCE_NAME_PLURAL: &'static str = "flux data records";
    const RESOURCE_DESCRIPTION: &'static str =
        "Gas flux measurements from chamber collars (CO2, CH4, H2O).";

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
        Ok(models.into_iter().map(FluxData::from).collect())
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
        Ok(FluxData::from(model))
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
            ("replicate", Self::ColumnType::Replicate),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("sensorprofile_id", Self::ColumnType::SensorprofileId),
            ("replicate", Self::ColumnType::Replicate),
            ("setting", Self::ColumnType::Setting),
        ]
    }
}
