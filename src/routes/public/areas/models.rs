use crate::common::geometry::Geometry;
use crate::routes::private::areas::db;
use crate::routes::public::sensors::models::SensorProfileSimple;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    pub geom: Option<Value>,
    pub plots: Vec<Plot>,
    pub sensors: Vec<SensorProfileSimple>,
}

impl From<db::Model> for Area {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            geom: None,      // Set later in func
            plots: vec![],   // Set later in func
            sensors: vec![], // Set later in func
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Plot {
    pub id: Uuid,
    pub name: String,
    pub geom: HashMap<i32, Geometry>,
    pub aggregated_samples:
        HashMap<i32, crate::routes::private::plots::models::SampleReplicateAggregate>,
}

impl From<crate::routes::private::plots::models::Plot> for Plot {
    fn from(model: crate::routes::private::plots::models::Plot) -> Self {
        let geom = Geometry {
            srid: model.coord_srid,
            x: model.coord_x,
            y: model.coord_y,
            z: model.coord_z,
        }
        .to_hashmap(vec![4326]);

        Plot {
            id: model.id,
            name: model.name,
            geom,
            aggregated_samples: model.aggregated_samples,
        }
    }
}
