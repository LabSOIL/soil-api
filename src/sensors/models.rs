use super::db::Model;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, query::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition,
    DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, Order, Statement,
};
use serde::{Deserialize, Serialize};
use std::vec;
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
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now(), on_create = chrono::Utc::now())]
    pub last_updated: chrono::DateTime<Utc>,
    pub area_id: Uuid,
    #[crudcrate(non_db_attr = true, default = None)]
    pub data_from: Option<chrono::DateTime<Utc>>,
    #[crudcrate(non_db_attr = true, default = None)]
    pub data_to: Option<chrono::DateTime<Utc>>,
    #[crudcrate(non_db_attr = true, default = vec![])]
    pub data: Vec<crate::sensors::data::models::SensorData>,
    #[crudcrate(non_db_attr = true, default = vec![])]
    pub data_base64: Option<String>,
    #[crudcrate(update_model = false, create_model = false)]
    pub area: Option<crate::areas::models::Area>,
    #[crudcrate(update_model = false, create_model = false)]
    pub assignments: Vec<crate::sensors::profile::assignment::models::SensorProfileAssignment>,
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
            area: None,
            assignments: vec![],
            data_from: None,
            data_to: None,
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

        let assignments: Vec<crate::sensors::profile::assignment::models::SensorProfileAssignment> =
            models
                .load_many(crate::sensors::profile::assignment::db::Entity, db)
                .await?
                .pop()
                .unwrap()
                .into_iter()
                .map(|assignment| assignment.into())
                .collect();

        let mut sensors: Vec<Sensor> = Vec::new();

        for model in models {
            let sensor: Sensor = model.into();
            let (data_from, data_to) = get_data_range(db, sensor.id).await?;

            let sensor_assignments: Vec<
                crate::sensors::profile::assignment::models::SensorProfileAssignment,
            > = assignments
                .iter()
                .filter(|assignment| assignment.sensor_id == sensor.id)
                .cloned()
                .collect();
            sensors.push(Sensor {
                assignments: sensor_assignments,
                data_from,
                data_to,
                ..sensor
            });
        }

        Ok(sensors)
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
        let (data_from, data_to) = get_data_range(db, sensor.id).await?;

        sensor.data = data.into_iter().map(|d| d.into()).collect();

        let assignments: Vec<crate::sensors::profile::assignment::models::SensorProfileAssignment> =
            crate::sensors::profile::assignment::db::Entity::find()
                .filter(crate::sensors::profile::assignment::db::Column::SensorId.eq(id))
                .all(db)
                .await?
                .into_iter()
                .map(|assignment| assignment.into())
                .collect();

        let sensor = Sensor {
            assignments,
            data_from,
            data_to,
            ..sensor.into()
        };
        Ok(sensor)
    }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let active_model: Self::ActiveModelType = create_model.clone().into();
        let result = Self::EntityType::insert(active_model).exec(db).await?;

        // Process sensor data from base64 if provided
        if let Some(ref data_base64) = create_model.data_base64 {
            // Process the base64 string into SensorData objects
            let new_data_result = crate::sensors::services::process_sensor_data_base64(
                data_base64,
                result.last_insert_id.into(),
            )
            .map_err(|e| DbErr::Custom(e))?;

            // Prepare bulk insert of new sensor data records into the DB
            let active_models: Vec<crate::sensors::data::db::ActiveModel> = new_data_result
                .into_iter()
                .map(|record| record.into_active_model())
                .collect();
            if !active_models.is_empty() {
                const CHUNK_SIZE: usize = 1000; // Define the chunk size
                for chunk in active_models.chunks(CHUNK_SIZE) {
                    crate::sensors::data::db::Entity::insert_many(chunk.to_vec())
                        .exec(db)
                        .await?;
                }
            }
        }

        match Self::get_one(db, result.last_insert_id.into()).await {
            Ok(obj) => Ok(obj),
            Err(_) => Err(DbErr::RecordNotFound(
                format!("{} not created", Self::RESOURCE_NAME_SINGULAR).into(),
            )),
        }
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
        // Process sensor data from base64 if provided
        if let Some(ref data_base64) = update_model.data_base64 {
            // Process the base64 string into SensorData objects
            let new_data_result =
                crate::sensors::services::process_sensor_data_base64(data_base64, id)
                    .map_err(|e| DbErr::Custom(e))?;

            // Query existing sensor data records for this sensor, sorted by time_utc descending
            let existing_data = crate::sensors::data::db::Entity::find()
                .filter(crate::sensors::data::db::Column::SensorId.eq(id))
                .order_by_desc(crate::sensors::data::db::Column::TimeUtc)
                .all(db)
                .await;

            let existing_data = existing_data?;

            // Determine the latest timestamp from existing data, if any
            let latest_time = existing_data.first().map(|record| record.time_utc);
            // Filter new data: only keep records with time_utc greater than the latest timestamp
            let mut filtered_new_data = new_data_result;
            if let Some(latest) = latest_time {
                filtered_new_data.retain(|record| record.time_utc > latest);
            }

            // If there are no new records to insert, return early
            if filtered_new_data.is_empty() {
                // Break early if there are no new records to insert
                let obj = Self::get_one(&db, id).await?;
                return Ok(obj);
            }

            // Prepare bulk insert of new sensor data records into the DB
            let active_models: Vec<crate::sensors::data::db::ActiveModel> = filtered_new_data
                .into_iter()
                .map(|record| record.into_active_model())
                .collect();

            if !active_models.is_empty() {
                const CHUNK_SIZE: usize = 1000; // Define the chunk size
                for chunk in active_models.chunks(CHUNK_SIZE) {
                    crate::sensors::data::db::Entity::insert_many(chunk.to_vec())
                        .exec(db)
                        .await?;
                }
            }
        }

        // Update the main Sensor record using the merge_into_activemodel logic
        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;

        let obj = Self::get_one(&db, response_obj.id).await?;

        Ok(obj)
    }
    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
        // If the sensor has a relationship to sensor profiles via sensor profile
        // assignments, we need to delete those first. Refuse to delete the sensor
        // if it has any sensor profile assignments.
        let sensor_profile_assignments = crate::sensors::profile::assignment::db::Entity::find()
            .filter(crate::sensors::profile::assignment::db::Column::SensorId.eq(id))
            .all(db)
            .await?;

        if !sensor_profile_assignments.is_empty() {
            return Err(DbErr::Custom(
                "Cannot delete sensor with sensor profile assignments".into(),
            ));
        }

        // First delete related data
        crate::sensors::data::db::Entity::delete_many()
            .filter(crate::sensors::data::db::Column::SensorId.eq(id))
            .exec(db)
            .await?;

        let res = <Self::EntityType as EntityTrait>::delete_by_id(id)
            .exec(db)
            .await?;

        Ok(res.rows_affected as usize)
    }

    async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr> {
        for id in &ids {
            let sensor_profile_assignments =
                crate::sensors::profile::assignment::db::Entity::find()
                    .filter(crate::sensors::profile::assignment::db::Column::SensorId.eq(*id))
                    .all(db)
                    .await?;

            if !sensor_profile_assignments.is_empty() {
                return Err(DbErr::Custom(
                    format!(
                        "Cannot delete sensor with sensor profile assignments: {}",
                        id
                    )
                    .into(),
                ));
            }

            crate::sensors::data::db::Entity::delete_many()
                .filter(crate::sensors::data::db::Column::SensorId.eq(*id))
                .exec(db)
                .await?;
        }

        Self::EntityType::delete_many()
            .filter(Self::ID_COLUMN.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(ids)
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

impl Sensor {
    pub async fn get_one_low_resolution(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Sensor, DbErr> {
        // Fetch the sensor record from the sensor table.
        let mut sensor = Self::get_one(db, id).await?;

        // Construct the raw SQL query using the provided script.
        // (Here the bucket interval is set to 24hr.)
        let sql = format!(
            "WITH buckets AS (
                SELECT
                  sensor_id,
                  to_timestamp(floor(extract('epoch' from time_utc) / (60*60*24)) * (60*60*24))::timestamptz AS bucket,
                  temperature_1,
                  temperature_2,
                  temperature_3,
                  temperature_average,
                  soil_moisture_count
                FROM sensordata
                WHERE sensor_id = '{}'
              )
              SELECT
                sensor_id,
                bucket,
                min(temperature_1) AS min_temp_1,
                max(temperature_1) AS max_temp_1,
                avg(temperature_1) AS avg_temp_1,
                min(temperature_2) AS min_temp_2,
                max(temperature_2) AS max_temp_2,
                avg(temperature_2) AS avg_temp_2,
                min(temperature_3) AS min_temp_3,
                max(temperature_3) AS max_temp_3,
                avg(temperature_3) AS avg_temp_3,
                min(temperature_average) AS min_temp_avg,
                max(temperature_average) AS max_temp_avg,
                avg(temperature_average) AS avg_temp_avg,
                round(avg(soil_moisture_count))::integer AS avg_soil_moisture_count,
                count(*) AS record_count
              FROM buckets
              GROUP BY sensor_id, bucket
              ORDER BY bucket",
            id
        );

        let stmt = Statement::from_sql_and_values(db.get_database_backend(), &sql, vec![]);

        // Execute the raw SQL query.
        let rows = db.query_all(stmt).await?;

        // Map each row into a SensorData object.
        let mut aggregated_data = Vec::new();
        for row in rows {
            let sensor_id: Uuid = row.try_get("", "sensor_id")?;
            let bucket: DateTime<Utc> = row.try_get("", "bucket")?;
            let avg_temp_1: f64 = row.try_get("", "avg_temp_1")?;
            let avg_temp_2: f64 = row.try_get("", "avg_temp_2")?;
            let avg_temp_3: f64 = row.try_get("", "avg_temp_3")?;
            let avg_temp_avg: f64 = row.try_get("", "avg_temp_avg")?;
            let avg_soil_moisture_count: i32 = row.try_get("", "avg_soil_moisture_count")?;

            let sensor_data = crate::sensors::data::models::SensorData {
                instrument_seq: 0,
                time_utc: bucket,
                temperature_1: avg_temp_1,
                temperature_2: avg_temp_2,
                temperature_3: avg_temp_3,
                temperature_average: avg_temp_avg,
                soil_moisture_count: avg_soil_moisture_count,
                shake: 0,
                error_flat: 0,
                sensor_id,
                last_updated: bucket,
            };
            aggregated_data.push(sensor_data);
        }

        // Insert gap rows if there's a large gap between consecutive buckets.
        aggregated_data.sort_by_key(|d| d.time_utc);
        let mut processed_data = Vec::new();
        let gap_threshold = chrono::Duration::days(1);
        for window in aggregated_data.windows(2) {
            processed_data.push(window[0].clone());
            if window[1].time_utc - window[0].time_utc > gap_threshold {
                // Insert a gap row with NaN values to break the line in Plotly.
                processed_data.push(crate::sensors::data::models::SensorData {
                    instrument_seq: 0,
                    time_utc: window[0].time_utc + gap_threshold,
                    temperature_1: f64::NAN,
                    temperature_2: f64::NAN,
                    temperature_3: f64::NAN,
                    temperature_average: f64::NAN,
                    // For soil_moisture_count, using -1 as a sentinel; adjust if needed.
                    soil_moisture_count: -1,
                    shake: 0,
                    error_flat: 0,
                    sensor_id: window[0].sensor_id,
                    last_updated: window[0].time_utc + gap_threshold,
                });
            }
        }
        if let Some(last) = aggregated_data.last() {
            processed_data.push(last.clone());
        }

        sensor.data = processed_data.into_iter().map(|d| d.into()).collect();
        Ok(Sensor::from(sensor))
    }
}

async fn get_data_range(
    db: &DatabaseConnection,
    sensor_id: Uuid,
) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>), DbErr> {
    // Do a query for the data related to the model, and find the first data
    // point and last and add them to data_from and data_to
    let first_data = crate::sensors::data::db::Entity::find()
        .filter(crate::sensors::data::db::Column::SensorId.eq(sensor_id))
        .order_by_asc(crate::sensors::data::db::Column::TimeUtc)
        .one(db)
        .await?;

    let last_data = crate::sensors::data::db::Entity::find()
        .filter(crate::sensors::data::db::Column::SensorId.eq(sensor_id))
        .order_by_desc(crate::sensors::data::db::Column::TimeUtc)
        .one(db)
        .await?;

    if let (Some(first), Some(last)) = (first_data, last_data) {
        Ok((Some(first.time_utc), Some(last.time_utc)))
    } else {
        Ok((None, None))
    }
}
