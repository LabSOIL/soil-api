use std::vec;

use super::db::{ActiveModel, Model};
use crate::areas::models::Area;
use crate::sensors::data::models::SensorData;
use crate::sensors::db;
use async_trait::async_trait;
use sea_orm::{sea_query::Expr, IntoActiveModel};
use serde::{Deserialize, Serialize};

use sea_orm::{
    entity::prelude::*, query::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition,
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, Order,
};
// use sea_orm::{NotSet, Set};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, ToSchema, Serialize, ToCreateModel, Deserialize, ToUpdateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct Sensor {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub name: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub description: Option<String>,
    pub comment: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now().naive_utc(), on_create = chrono::Utc::now().naive_utc())]
    pub last_updated: chrono::NaiveDateTime,
    pub area_id: Uuid,
    #[crudcrate(non_db_attr = true, default = vec![])]
    pub data: Vec<crate::sensors::data::models::SensorData>,
    #[crudcrate(non_db_attr = true, default = vec![])]
    pub data_base64: Option<String>,
    // #[crudcrate(update_model = false, create_model = false)]
    // area: Option<crate::areas::models::AreaBasicWithProject>,
}

impl From<Model> for Sensor {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            serial_number: model.serial_number,
            manufacturer: model.manufacturer,
            description: model.description,
            comment: model.comment,
            last_updated: model.last_updated,
            area_id: model.area_id,
            data: vec![],
            data_base64: None,
            // area: None,
        }
    }
}

#[async_trait]
impl CRUDResource for Sensor {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Sensor;
    type CreateModel = SensorCreate;
    type UpdateModel = SensorUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "sensor";
    const RESOURCE_NAME_PLURAL: &'static str = "sensors";

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
        let (sensor, data) = Self::EntityType::find()
            .find_with_related(super::data::db::Entity)
            .filter(Self::ColumnType::Id.eq(id))
            .order_by_asc(crate::sensors::data::db::Column::TimeUtc)
            .all(db)
            .await?
            .pop()
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))?;

        let mut sensor: super::models::Sensor = sensor.into();
        sensor.data = data.into_iter().map(|d| d.into()).collect();

        Ok(Self::ApiModel::from(sensor))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        println!("Starting update for sensor with id: {}", id);

        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))?
            .into();

        println!("Found sensor in database: {:?}", db_obj);

        // Process sensor data from base64 if provided
        if let Some(ref data_base64) = update_model.data_base64 {
            println!("Processing base64 sensor data");

            // Process the base64 string into SensorData objects
            let new_data_result =
                crate::sensors::services::process_sensor_data_base64(data_base64, id)
                    .map_err(|e| DbErr::Custom(e))?;

            println!(
                "Processed {} new sensor data records",
                new_data_result.len()
            );

            // Query existing sensor data records for this sensor, sorted by time_utc descending
            let existing_data = crate::sensors::data::db::Entity::find()
                .filter(crate::sensors::data::db::Column::SensorId.eq(id))
                .order_by_desc(crate::sensors::data::db::Column::TimeUtc)
                .all(db)
                .await;

            println!("Error: {:?}", existing_data);
            let existing_data = existing_data?;
            println!("Found {} existing sensor data records", existing_data.len());

            // Determine the latest timestamp from existing data, if any
            let latest_time = existing_data.first().map(|record| record.time_utc);
            println!("Latest timestamp from existing data: {:?}", latest_time);
            // Filter new data: only keep records with time_utc greater than the latest timestamp
            let mut filtered_new_data = new_data_result;
            if let Some(latest) = latest_time {
                filtered_new_data.retain(|record| record.time_utc > latest);
            }

            println!(
                "Filtered to {} new sensor data records after timestamp check",
                filtered_new_data.len()
            );

            // If there are no new records to insert, return early
            if filtered_new_data.is_empty() {
                println!("No new sensor data records to insert");
                // Break early if there are no new records to insert
                let obj = Self::get_one(&db, id).await?;
                return Ok(obj);
            }

            // Prepare bulk insert of new sensor data records into the DB
            let active_models: Vec<crate::sensors::data::db::ActiveModel> = filtered_new_data
                .into_iter()
                .map(|record| record.into_active_model())
                .collect();

            println!(
                "Prepared {} new sensor data records for bulk insert",
                active_models.len()
            );
            println!("First record: {:?}", active_models.first());
            if !active_models.is_empty() {
                const CHUNK_SIZE: usize = 1000; // Define the chunk size
                for chunk in active_models.chunks(CHUNK_SIZE) {
                    let resp = crate::sensors::data::db::Entity::insert_many(chunk.to_vec())
                        .exec(db)
                        .await;

                    println!("Bulk insert response: {:?}", resp);
                }

                println!("Inserted new sensor data records into the database");
            } else {
                println!("No new sensor data records to insert");
            }
        }

        // Update the main Sensor record using the merge_into_activemodel logic
        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        println!("Updated sensor record in database: {:?}", response_obj);

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
