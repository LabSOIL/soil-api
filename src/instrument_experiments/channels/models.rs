// instrument_experiments/channels/models.rs

use async_trait::async_trait;
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use utoipa::ToSchema;
use uuid::Uuid;

use super::db::Model;

/// The API model for an instrument experiment channel.
#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct InstrumentExperimentChannel {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub channel_name: String,
    pub experiment_id: Uuid,
    pub baseline_spline: Option<Json>,
    pub time_values: Option<Json>,
    pub raw_values: Option<Json>,
    pub baseline_values: Option<Json>,
    pub baseline_chosen_points: Option<Json>,
    pub integral_chosen_pairs: Option<Json>,
    pub integral_results: Option<Json>,
}

impl From<Model> for InstrumentExperimentChannel {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            channel_name: model.channel_name,
            experiment_id: model.experiment_id,
            baseline_spline: model.baseline_spline,
            time_values: model.time_values,
            raw_values: model.raw_values,
            baseline_values: model.baseline_values,
            baseline_chosen_points: model.baseline_chosen_points,
            integral_chosen_pairs: model.integral_chosen_pairs,
            integral_results: model.integral_results,
        }
    }
}

#[async_trait]
impl CRUDResource for InstrumentExperimentChannel {
    type EntityType = crate::instrument_experiments::channels::db::Entity;
    type ColumnType = crate::instrument_experiments::channels::db::Column;
    type ModelType = crate::instrument_experiments::channels::db::Model;
    type ActiveModelType = crate::instrument_experiments::channels::db::ActiveModel;
    type ApiModel = InstrumentExperimentChannel;
    type CreateModel = InstrumentExperimentChannelCreate;
    type UpdateModel = InstrumentExperimentChannelUpdate;

    const ID_COLUMN: Self::ColumnType = crate::instrument_experiments::channels::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "instrumentexperimentchannel";
    const RESOURCE_NAME_PLURAL: &'static str = "instrumentexperimentchannels";

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
        Ok(models
            .into_iter()
            .map(InstrumentExperimentChannel::from)
            .collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Instrument experiment channel not found".into(),
            ))?;
        Ok(InstrumentExperimentChannel::from(model))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let db_obj: crate::instrument_experiments::channels::db::ActiveModel =
            Self::EntityType::find_by_id(id)
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound(
                    "Instrument experiment channel not found".into(),
                ))?
                .into();

        let updated_obj = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let obj = Self::get_one(db, response_obj.id).await?;
        Ok(obj)
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            (
                "id",
                crate::instrument_experiments::channels::db::Column::Id,
            ),
            (
                "channel_name",
                crate::instrument_experiments::channels::db::Column::ChannelName,
            ),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![(
            "channel_name",
            crate::instrument_experiments::channels::db::Column::ChannelName,
        )]
    }
}
