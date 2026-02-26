use crate::common::geometry::Geometry;
use crate::routes::private::sensors::profile::db;
use crate::routes::private::sensors::profile::db::ProfileTypeEnum;
use crate::routes::private::sensors::profile::models::DepthAverageData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SensorProfile {
    pub id: Uuid,
    pub area_id: Uuid,
    pub name: String,
    pub geom: HashMap<i32, Geometry>,
    pub data_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
}

impl From<crate::routes::private::sensors::profile::models::SensorProfile> for SensorProfile {
    fn from(model: crate::routes::private::sensors::profile::models::SensorProfile) -> Self {
        let geom = Geometry {
            // Check that the model has srid, x, y, z and if not set to None
            srid: model.coord_srid.unwrap_or_default(),
            x: model.coord_x.unwrap_or_default(),
            y: model.coord_y.unwrap_or_default(),
            z: model.coord_z.unwrap_or_default(),
        }
        .to_hashmap(vec![4326]);

        Self {
            id: model.id,
            area_id: model.area_id,
            name: model.name,
            geom,
            data_by_depth_cm: model.data_by_depth_cm,
        }
    }
}

impl From<db::Model> for SensorProfile {
    fn from(model: db::Model) -> Self {
        let geom = Geometry {
            // Check that the model has srid, x, y, z and if not set to None
            srid: model.coord_srid.unwrap_or_default(),
            x: model.coord_x.unwrap_or_default(),
            y: model.coord_y.unwrap_or_default(),
            z: model.coord_z.unwrap_or_default(),
        }
        .to_hashmap(vec![4326]);

        Self {
            id: model.id,
            area_id: model.area_id,
            name: model.name,
            geom,
            data_by_depth_cm: HashMap::new(),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SensorProfileSimple {
    pub id: Uuid,
    pub name: String,
    pub profile_type: ProfileTypeEnum,
    pub geom: HashMap<i32, Geometry>,
    pub average_temperature: HashMap<i32, f64>,
    pub average_moisture: HashMap<i32, f64>,
}

impl From<crate::routes::private::sensors::profile::models::SensorProfile> for SensorProfileSimple {
    fn from(model: crate::routes::private::sensors::profile::models::SensorProfile) -> Self {
        let geom = Geometry {
            srid: model.coord_srid.unwrap_or_default(),
            x: model.coord_x.unwrap_or_default(),
            y: model.coord_y.unwrap_or_default(),
            z: model.coord_z.unwrap_or_default(),
        }
        .to_hashmap(vec![4326]);

        Self {
            id: model.id,
            name: model.name,
            profile_type: model.profile_type,
            average_temperature: HashMap::new(), // Set later in func
            average_moisture: HashMap::new(),    // Set later in func
            geom,
        }
    }
}

/// Public flux data time series response
#[derive(ToSchema, Serialize, Deserialize)]
pub struct FluxDataPoint {
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
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SensorProfileFlux {
    pub id: Uuid,
    pub name: String,
    pub geom: HashMap<i32, Geometry>,
    pub flux_data: Vec<FluxDataPoint>,
}

/// Public redox data time series response
#[derive(ToSchema, Serialize, Deserialize)]
pub struct RedoxDataPoint {
    pub measured_on: DateTime<Utc>,
    pub ch1_5cm_mv: Option<f64>,
    pub ch2_15cm_mv: Option<f64>,
    pub ch3_25cm_mv: Option<f64>,
    pub ch4_35cm_mv: Option<f64>,
    pub temp_c: Option<f64>,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SensorProfileRedox {
    pub id: Uuid,
    pub name: String,
    pub geom: HashMap<i32, Geometry>,
    pub redox_data: Vec<RedoxDataPoint>,
}
