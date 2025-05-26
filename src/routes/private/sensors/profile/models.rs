use std::collections::HashMap;

use super::db::Model;
use crate::config::Config;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    Order, QueryOrder, QuerySelect, Statement, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct DepthAverageData {
    pub time_utc: DateTime<Utc>,
    pub y: f64,
}

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct SensorProfile {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(
        update_model = false,
        create_model = false,
        on_update = chrono::Utc::now(),
        on_create = chrono::Utc::now()
    )]
    pub last_updated: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub area_id: Uuid,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    pub coord_srid: Option<i32>,
    #[crudcrate(update_model = false, create_model = false)]
    #[schema(no_recursion)]
    pub assignments:
        Vec<crate::routes::private::sensors::profile::assignment::models::SensorProfileAssignment>,
    // This is a struct that carries the average data over an assigned depth, keyed by depth in cm
    #[crudcrate(non_db_attr = true, default = HashMap::new())]
    pub average_temperature_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
}

impl From<Model> for SensorProfile {
    fn from(model: Model) -> Self {
        Self::from_with_assignments(model, vec![])
    }
}

impl SensorProfile {
    fn from_with_assignments(
        model: Model,
        assignments: Vec<
            crate::routes::private::sensors::profile::assignment::models::SensorProfileAssignment,
        >,
    ) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: model.name,
            description: model.description,
            area_id: model.area_id,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
            coord_srid: model.coord_srid,
            assignments,
            average_temperature_by_depth_cm: HashMap::new(),
        }
    }
}

#[async_trait]
impl CRUDResource for SensorProfile {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ActiveModelType = super::db::ActiveModel;
    type CreateModel = SensorProfileCreate;
    type UpdateModel = SensorProfileUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "profile (sensor)";
    const RESOURCE_NAME_PLURAL: &'static str = "profiles (sensor)";
    const RESOURCE_DESCRIPTION: &'static str = "A sensor profile represents a geographical location where a sensor is placed. The assignments that belong to it define the time periods for which sensor is active and the data it collects.";

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

        if models.is_empty() {
            return Ok(vec![]);
        }
        let mut assignments: Vec<super::assignment::models::SensorProfileAssignment> = models
            .load_many(super::assignment::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .into_iter()
            .map(std::convert::Into::into)
            .collect();

        for assignment in &mut assignments {
            // Get the sensor for each assignment
            let sensor: crate::routes::private::sensors::models::Sensor =
                crate::routes::private::sensors::db::Entity::find()
                    .filter(
                        crate::routes::private::sensors::db::Column::Id.eq(assignment.sensor_id),
                    )
                    .one(db)
                    .await?
                    .ok_or(DbErr::RecordNotFound("Sensor not found".into()))?
                    .into();
            assignment.sensor = Some(sensor);
        }

        let mut sensor_profiles: Vec<SensorProfile> = Vec::new();
        for model in models {
            let sensor_profile = SensorProfile::from_with_assignments(model, assignments.clone());
            sensor_profiles.push(sensor_profile);
        }
        Ok(sensor_profiles)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let mut models = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .all(db)
            .await?;

        let mut assignments: Vec<super::assignment::models::SensorProfileAssignment> = models
            .load_many(super::assignment::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .into_iter()
            .map(std::convert::Into::into)
            .collect();

        // As in get_all, get sensor for each assignment
        for assignment in &mut assignments {
            let sensor: crate::routes::private::sensors::models::Sensor =
                crate::routes::private::sensors::db::Entity::find()
                    .filter(
                        crate::routes::private::sensors::db::Column::Id.eq(assignment.sensor_id),
                    )
                    .one(db)
                    .await?
                    .ok_or(DbErr::RecordNotFound("Sensor not found".into()))?
                    .into();
            assignment.sensor = Some(sensor);
        }
        let model = models.pop().ok_or(DbErr::RecordNotFound(format!(
            "{} not found",
            Self::RESOURCE_NAME_SINGULAR
        )))?;

        let sensor_profile = SensorProfile::from_with_assignments(model, assignments);
        Ok(sensor_profile)
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

impl SensorProfile {
    pub async fn get_one_low_resolution(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<SensorProfile, DbErr> {
        let mut sensor_profile = Self::get_one(db, id).await?;
        // For each assignment, fetch aggregated sensor data and store it in the assignment’s data field
        for assignment in &mut sensor_profile.assignments {
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
                        WHERE sensor_id = '{}' AND time_utc BETWEEN '{}' AND '{}'
                    )
                    SELECT
                        sensor_id,
                        bucket AS time_utc,
                        min(temperature_1) AS temperature_1,
                        min(temperature_2) AS temperature_2,
                        min(temperature_3) AS temperature_3,
                        min(temperature_average) AS temperature_average,
                        round(avg(soil_moisture_count))::integer AS soil_moisture_count,
                        count(*) AS record_count
                    FROM buckets
                    GROUP BY sensor_id, bucket
                    ORDER BY bucket",
                    assignment.sensor_id,
                    assignment.date_from,
                    assignment.date_to
                );
            let stmt = Statement::from_sql_and_values(db.get_database_backend(), &sql, vec![]);
            let rows = db.query_all(stmt).await?;
            let mut data_vec = Vec::new();
            for row in rows {
                let sensor_id: Uuid = row.try_get("", "sensor_id")?;
                let time_utc: DateTime<Utc> = row.try_get("", "time_utc")?;
                let temperature_1: f64 = row.try_get("", "temperature_1")?;
                let temperature_2: f64 = row.try_get("", "temperature_2")?;
                let temperature_3: f64 = row.try_get("", "temperature_3")?;
                let temperature_average: f64 = row.try_get("", "temperature_average")?;
                let soil_moisture_count: i32 = row.try_get("", "soil_moisture_count")?;
                let sensor_data = crate::routes::private::sensors::data::models::SensorData {
                    instrument_seq: 0,
                    time_utc,
                    temperature_1,
                    temperature_2,
                    temperature_3,
                    temperature_average,
                    soil_moisture_count,
                    shake: 0,
                    error_flat: 0,
                    sensor_id,
                };
                data_vec.push(sensor_data);
            }
            assignment.data = data_vec;
        }
        // Also merge all assignments’ data into sensor_profile.data (if needed for overall view)
        let mut all_data = Vec::new();
        for assignment in &sensor_profile.assignments {
            all_data.extend(assignment.data.clone());
        }
        all_data.sort_by_key(|d| d.time_utc);
        // sensor_profile.data = all_data;
        Ok(sensor_profile)
    }

    pub async fn get_one_high_resolution(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<SensorProfile, DbErr> {
        let mut sensor_profile = Self::get_one(db, id).await?;
        for assignment in &mut sensor_profile.assignments {
            let sensor_data_records = crate::routes::private::sensors::data::db::Entity::find()
                .filter(
                    crate::routes::private::sensors::data::db::Column::SensorId
                        .eq(assignment.sensor_id),
                )
                .filter(
                    crate::routes::private::sensors::data::db::Column::TimeUtc
                        .between(assignment.date_from, assignment.date_to),
                )
                .order_by_asc(crate::routes::private::sensors::data::db::Column::TimeUtc)
                .all(db)
                .await?;
            let mut data_vec = Vec::new();
            for record in sensor_data_records {
                let sensor_data: crate::routes::private::sensors::data::models::SensorData =
                    record.into();
                data_vec.push(sensor_data);
            }
            assignment.data = data_vec;
        }
        let mut all_data = Vec::new();
        for assignment in &sensor_profile.assignments {
            all_data.extend(assignment.data.clone());
        }
        all_data.sort_by_key(|d| d.time_utc);
        // sensor_profile.data = all_data;
        Ok(sensor_profile)
    }

    /// Load average (or raw) temperature by depth.
    ///
    /// - `window_hours = Some(h)`: bucket into h-hour windows and average.
    /// - `window_hours = None`: return every datapoint (full resolution).
    pub async fn load_average_temperature_series_by_depth_cm(
        &self,
        db: &DatabaseConnection,
        window_hours: Option<i64>,
    ) -> Result<HashMap<i32, Vec<DepthAverageData>>, DbErr> {
        // 1. Run the appropriate SQL and collect rows
        let rows = if let Some(hours) = window_hours {
            // Bucketing SQL
            let sql = r"
            WITH depths AS (
                SELECT u.depth_cm, u.idx, spa.sensor_id, spa.date_from, spa.date_to
                FROM sensorprofile_assignment AS spa
                CROSS JOIN LATERAL
                    unnest(array[spa.depth_cm_sensor1, spa.depth_cm_sensor2, spa.depth_cm_sensor3]) WITH ORDINALITY
                    AS u(depth_cm, idx)
                WHERE spa.sensorprofile_id = $1
            ), buckets AS (
                SELECT
                    d.depth_cm,
                    to_timestamp(
                        floor(extract(epoch FROM sd.time_utc) / ($2 * 3600))
                        * ($2 * 3600)
                    )::timestamptz AS time_utc,
                    (array[sd.temperature_1, sd.temperature_2, sd.temperature_3])[d.idx] AS temp
                FROM depths AS d
                JOIN sensordata AS sd
                  ON sd.sensor_id = d.sensor_id
                 AND sd.time_utc BETWEEN d.date_from AND d.date_to
            )
            SELECT depth_cm, time_utc, AVG(temp) AS y
            FROM buckets
            GROUP BY depth_cm, time_utc
            ORDER BY depth_cm, time_utc;
        ";

            let stmt = Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                vec![
                    self.id.into(), // $1 → profile ID
                    hours.into(),   // $2 → window in hours
                ],
            );
            db.query_all(stmt).await?
        } else {
            // Full-resolution SQL
            let sql = r"
            WITH depths AS (
                SELECT
                    u.depth_cm,
                    u.idx,
                    spa.sensor_id,
                    spa.date_from,
                    spa.date_to
                FROM sensorprofile_assignment AS spa
                CROSS JOIN LATERAL
                    unnest(
                        array[
                            spa.depth_cm_sensor1,
                            spa.depth_cm_sensor2,
                            spa.depth_cm_sensor3
                        ]
                    ) WITH ORDINALITY
                    AS u(depth_cm, idx)
                WHERE spa.sensorprofile_id = $1
            )
            SELECT
                d.depth_cm,
                sd.time_utc       AS time_utc,
                (array[
                    sd.temperature_1,
                    sd.temperature_2,
                    sd.temperature_3
                ])[d.idx]       AS y
            FROM depths AS d
            JOIN sensordata AS sd
              ON sd.sensor_id = d.sensor_id
             AND sd.time_utc BETWEEN d.date_from AND d.date_to
            ORDER BY d.depth_cm, sd.time_utc;
        ";

            let stmt = Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                vec![self.id.into()], // $1 → profile ID
            );
            db.query_all(stmt).await?
        };

        // 2. Build HashMap<depth_cm, Vec<DepthAverageData>>
        let mut map: HashMap<i32, Vec<DepthAverageData>> = HashMap::new();
        for row in rows {
            let depth_cm: i32 = row.try_get("", "depth_cm")?;
            let time_utc: DateTime<Utc> = row.try_get("", "time_utc")?;
            let y: f64 = row.try_get("", "y")?;

            map.entry(depth_cm)
                .or_default()
                .push(DepthAverageData { time_utc, y });
        }

        Ok(map)
    }
}
