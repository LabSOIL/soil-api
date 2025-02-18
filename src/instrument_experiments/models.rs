use super::db::Model;
use async_trait::async_trait;
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct InstrumentExperiment {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now().naive_utc(), on_create = chrono::Utc::now().naive_utc())]
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub date: Option<chrono::NaiveDateTime>,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub device_filename: Option<String>,
    pub data_source: Option<String>,
    pub instrument_model: Option<String>,
    pub init_e: Option<f64>,
    pub sample_interval: Option<f64>,
    pub run_time: Option<f64>,
    pub quiet_time: Option<f64>,
    pub sensitivity: Option<f64>,
    pub samples: Option<i32>,
    pub project_id: Option<Uuid>,
    #[crudcrate(update_model = false, create_model = false)]
    pub channels: Vec<super::channels::models::InstrumentExperimentChannel>,
}

impl From<Model> for InstrumentExperiment {
    fn from(model: Model) -> Self {
        Self {
            name: model.name,
            date: model.date,
            description: model.description,
            filename: model.filename,
            device_filename: model.device_filename,
            data_source: model.data_source,
            instrument_model: model.instrument_model,
            init_e: model.init_e,
            sample_interval: model.sample_interval,
            run_time: model.run_time,
            quiet_time: model.quiet_time,
            sensitivity: model.sensitivity,
            samples: model.samples,
            id: model.id,
            last_updated: model.last_updated,
            project_id: model.project_id,
            channels: vec![],
        }
    }
}

#[async_trait]
impl CRUDResource for InstrumentExperiment {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = InstrumentExperiment;
    type CreateModel = InstrumentExperimentCreate;
    type UpdateModel = InstrumentExperimentUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "instrumentexperiment";
    const RESOURCE_NAME_PLURAL: &'static str = "instrumentexperiments";

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
        Ok(models.into_iter().map(InstrumentExperiment::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Instrument experiment not found".into(),
            ))?;
        Ok(InstrumentExperiment::from(model))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))?
            .into();

        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let obj = Self::get_one(&db, response_obj.id).await?;
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
