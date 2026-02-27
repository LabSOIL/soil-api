use crate::common::geometry::Geometry;
use crate::routes::private::sensors::profile::db::ProfileTypeEnum;
use crate::routes::private::sensors::profile::models::DepthAverageData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SensorRef {
    pub id: Uuid,
    pub name: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct ParameterData {
    pub name: String,
    pub depth_cm: i32,
    pub units: String,
    pub values: Vec<Option<f64>>,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SensorProfile {
    pub sensor: SensorRef,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub resolution: String,
    pub times: Vec<DateTime<Utc>>,
    pub parameters: Vec<ParameterData>,
}

impl SensorProfile {
    /// Build a `SensorProfile` response from a depth→timeseries map.
    /// Aligns all depths to a common time axis with `None` for missing values.
    pub fn from_depth_map(
        id: Uuid,
        name: &str,
        resolution: &str,
        units: &str,
        depth_map: HashMap<i32, Vec<DepthAverageData>>,
    ) -> Self {
        // Collect all unique timestamps across all depths, sorted
        let mut all_times = BTreeSet::new();
        for series in depth_map.values() {
            for d in series {
                all_times.insert(d.time_utc);
            }
        }
        let times: Vec<DateTime<Utc>> = all_times.into_iter().collect();

        // Build time→index lookup
        let time_index: HashMap<DateTime<Utc>, usize> =
            times.iter().enumerate().map(|(i, t)| (*t, i)).collect();

        // Sort depths numerically
        let mut depths: Vec<i32> = depth_map.keys().copied().collect();
        depths.sort_unstable();

        // Build parameters aligned to the times axis
        let parameters: Vec<ParameterData> = depths
            .into_iter()
            .map(|depth_cm| {
                let mut values: Vec<Option<f64>> = vec![None; times.len()];
                if let Some(series) = depth_map.get(&depth_cm) {
                    for d in series {
                        if let Some(&idx) = time_index.get(&d.time_utc) {
                            values[idx] = Some(d.y);
                        }
                    }
                }
                ParameterData {
                    name: format!("{depth_cm}cm"),
                    depth_cm,
                    units: units.to_string(),
                    values,
                }
            })
            .collect();

        let start = times.first().copied();
        let end = times.last().copied();

        Self {
            sensor: SensorRef {
                id,
                name: name.to_string(),
            },
            start,
            end,
            resolution: resolution.to_string(),
            times,
            parameters,
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
