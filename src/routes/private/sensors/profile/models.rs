use std::collections::HashMap;

use super::db::Model;
use crate::{config::Config, routes::private::sensors::profile::db::SoilTypeEnum};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    Order, QueryOrder, QuerySelect, Statement, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use soil_sensor_toolbox::mc_calc_vwc;
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
    pub soil_type_vwc: SoilTypeEnum,
    #[crudcrate(update_model = false, create_model = false)]
    #[schema(no_recursion)]
    pub assignments:
        Vec<crate::routes::private::sensors::profile::assignment::models::SensorProfileAssignment>,
    // Temperature data grouped by depth in cm
    #[crudcrate(non_db_attr = true, default = HashMap::new())]
    pub temperature_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
    // VWC moisture data grouped by depth in cm
    #[crudcrate(non_db_attr = true, default = HashMap::new())]
    pub moisture_vwc_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
    // Raw moisture counts grouped by depth in cm (for reference)
    #[crudcrate(non_db_attr = true, default = HashMap::new())]
    pub moisture_raw_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
    // Legacy field for backward compatibility
    #[crudcrate(non_db_attr = true, default = HashMap::new())]
    pub data_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
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
            soil_type_vwc: model.soil_type_vwc,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
            coord_srid: model.coord_srid,
            assignments,
            temperature_by_depth_cm: HashMap::new(),
            moisture_vwc_by_depth_cm: HashMap::new(),
            moisture_raw_by_depth_cm: HashMap::new(),
            data_by_depth_cm: HashMap::new(),
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

        // Load temperature data grouped by depth
        let temperature_data = sensor_profile
            .load_average_temperature_series_by_depth_cm(db, Some(24)) // 24-hour windows for low resolution
            .await?;

        // Load moisture data (both VWC and raw) grouped by depth
        let (moisture_vwc_data, moisture_raw_data) = sensor_profile
            .load_moisture_data_by_depth_cm(db, Some(24))
            .await?;

        // Populate all the data fields
        sensor_profile.temperature_by_depth_cm = temperature_data;
        sensor_profile.moisture_vwc_by_depth_cm = moisture_vwc_data;
        sensor_profile.moisture_raw_by_depth_cm = moisture_raw_data;
        sensor_profile.data_by_depth_cm = sensor_profile.temperature_by_depth_cm.clone(); // Legacy compatibility

        Ok(sensor_profile)
    }

    pub async fn get_one_high_resolution(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<SensorProfile, DbErr> {
        let mut sensor_profile = Self::get_one(db, id).await?;

        // Load temperature data grouped by depth (full resolution)
        let temperature_data = sensor_profile
            .load_average_temperature_series_by_depth_cm(db, None) // No windowing for high resolution
            .await?;

        // Load moisture data (both VWC and raw) grouped by depth (full resolution)
        let (moisture_vwc_data, moisture_raw_data) = sensor_profile
            .load_moisture_data_by_depth_cm(db, None)
            .await?;

        // Populate all the data fields
        sensor_profile.temperature_by_depth_cm = temperature_data;
        sensor_profile.moisture_vwc_by_depth_cm = moisture_vwc_data;
        sensor_profile.moisture_raw_by_depth_cm = moisture_raw_data;
        sensor_profile.data_by_depth_cm = sensor_profile.temperature_by_depth_cm.clone(); // Legacy compatibility

        Ok(sensor_profile)
    }

    /// Load both VWC and raw moisture data grouped by depth
    pub async fn load_moisture_data_by_depth_cm(
        &self,
        db: &DatabaseConnection,
        window_hours: Option<i64>,
    ) -> Result<
        (
            HashMap<i32, Vec<DepthAverageData>>,
            HashMap<i32, Vec<DepthAverageData>>,
        ),
        DbErr,
    > {
        // Convert SoilTypeEnum to SoilType for VWC calculation
        let soil_type: soil_sensor_toolbox::SoilType = self.soil_type_vwc.clone().into();

        // Build the SQL to include both moisture and temperature data
        let rows = if let Some(hours) = window_hours {
            let sql = r"
            WITH depths AS (
                SELECT
                    spa.depth_cm_moisture AS depth_cm,
                    spa.sensor_id,
                    spa.date_from,
                    spa.date_to
                FROM sensorprofile_assignment AS spa
                WHERE spa.sensorprofile_id = $1 AND spa.depth_cm_moisture IS NOT NULL
            ), buckets AS (
                SELECT
                    d.depth_cm,
                    to_timestamp(
                        floor(extract(epoch FROM sd.time_utc) / ($2 * 3600))
                        * ($2 * 3600)
                    )::timestamptz AS time_utc,
                    sd.soil_moisture_count::double precision AS moisture_count,
                    sd.temperature_1 AS temperature
                FROM depths AS d
                JOIN sensordata AS sd
                  ON sd.sensor_id = d.sensor_id
                 AND sd.time_utc BETWEEN d.date_from AND d.date_to
            )
            SELECT
                depth_cm,
                time_utc,
                AVG(moisture_count) AS moisture_count,
                AVG(temperature) AS temperature
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
            let sql = r"
            WITH depths AS (
                SELECT
                    spa.depth_cm_moisture AS depth_cm,
                    spa.sensor_id,
                    spa.date_from,
                    spa.date_to
                FROM sensorprofile_assignment AS spa
                WHERE spa.sensorprofile_id = $1 AND spa.depth_cm_moisture IS NOT NULL
            )
            SELECT
                d.depth_cm,
                sd.time_utc AS time_utc,
                sd.soil_moisture_count::double precision AS moisture_count,
                sd.temperature_1 AS temperature
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

        // Process the rows and create both VWC and raw moisture data
        let mut vwc_map: HashMap<i32, Vec<DepthAverageData>> = HashMap::new();
        let mut raw_map: HashMap<i32, Vec<DepthAverageData>> = HashMap::new();

        for row in rows {
            let depth_cm: i32 = row.try_get("", "depth_cm")?;
            let time_utc: DateTime<Utc> = row.try_get("", "time_utc")?;
            let moisture_count: f64 = row.try_get("", "moisture_count")?;
            let temperature: f64 = row.try_get("", "temperature")?;

            // Calculate VWC using the mc_calc_vwc function
            let vwc = mc_calc_vwc(moisture_count, temperature, soil_type);

            // Add VWC data
            vwc_map
                .entry(depth_cm)
                .or_default()
                .push(DepthAverageData { time_utc, y: vwc });

            // Add raw moisture count data
            raw_map.entry(depth_cm).or_default().push(DepthAverageData {
                time_utc,
                y: moisture_count,
            });
        }

        Ok((vwc_map, raw_map))
    }

    /// Helper method to get VWC (moisture) data by depth for API consumers
    /// This provides easy access to the converted moisture values
    #[allow(dead_code)]
    pub async fn get_moisture_data_by_depth_cm(
        &self,
        db: &DatabaseConnection,
        window_hours: Option<i64>,
    ) -> Result<HashMap<i32, Vec<DepthAverageData>>, DbErr> {
        let (vwc_data, _raw_data) = self
            .load_moisture_data_by_depth_cm(db, window_hours)
            .await?;
        Ok(vwc_data)
    }

    /// Helper method to get temperature data by depth for API consumers  
    /// This provides easy access to the grouped temperature values
    #[allow(dead_code)]
    pub async fn get_temperature_data_by_depth_cm(
        &self,
        db: &DatabaseConnection,
        window_hours: Option<i64>,
    ) -> Result<HashMap<i32, Vec<DepthAverageData>>, DbErr> {
        self.load_average_temperature_series_by_depth_cm(db, window_hours)
            .await
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

    /// Load average (or raw) soil-moisture by the `depth_cm_moisture` assignment.
    ///
    /// - `window_hours = Some(h)`: bucket into h-hour windows and average.
    /// - `window_hours = None`: return every datapoint (full resolution).
    pub async fn load_average_moisture_series_by_depth_cm(
        &self,
        db: &DatabaseConnection,
        window_hours: Option<i64>,
    ) -> Result<HashMap<i32, Vec<DepthAverageData>>, DbErr> {
        let (vwc_data, _raw_data) = self
            .load_moisture_data_by_depth_cm(db, window_hours)
            .await?;
        Ok(vwc_data)
    }
}
