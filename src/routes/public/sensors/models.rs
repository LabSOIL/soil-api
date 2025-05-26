use crate::common::geometry::Geometry;
use crate::routes::private::sensors::profile::db;
use crate::routes::private::sensors::profile::models::DepthAverageData;
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
    pub average_temperature_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
    pub average_moisture_by_depth_cm: HashMap<i32, Vec<DepthAverageData>>,
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
            average_temperature_by_depth_cm: model.average_temperature_by_depth_cm,
            average_moisture_by_depth_cm: model.average_moisture_by_depth_cm,
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
            average_temperature_by_depth_cm: HashMap::new(), // Set later in func
            average_moisture_by_depth_cm: HashMap::new(),    // Set later in func
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SensorProfileSimple {
    pub id: Uuid,
    pub name: String,
    pub geom: HashMap<i32, Geometry>,
    pub average_temperature: HashMap<i32, f64>,
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
            average_temperature: HashMap::new(), // Set later in func
            geom,
        }
    }
}
