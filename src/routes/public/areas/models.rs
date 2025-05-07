use crate::routes::private::areas::db;
use proj4rs::{Proj, transform};
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
}

impl From<db::Model> for Area {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            geom: None,    // Set later in func
            plots: vec![], // Set later in func
        }
    }
}
#[derive(ToSchema, Serialize, Deserialize)]
pub struct Geometry {
    pub srid: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Plot {
    pub id: Uuid,
    pub name: String,
    pub geom: HashMap<i32, Geometry>,
}

impl From<crate::routes::private::plots::db::Model> for Plot {
    fn from(model: crate::routes::private::plots::db::Model) -> Self {
        let mut geom = HashMap::new();

        // Original SRID and coordinates
        geom.insert(
            model.coord_srid,
            Geometry {
                srid: model.coord_srid,
                x: model.coord_x,
                y: model.coord_y,
                z: model.coord_z,
            },
        );

        // Transform coordinates to supply also WGS84 for mapping
        let wgs84: Proj = Proj::from_epsg_code(4326).unwrap();
        let model_srid: Proj = Proj::from_epsg_code(model.coord_srid.try_into().unwrap()).unwrap();
        let mut coordinates = (model.coord_x, model.coord_y, model.coord_z);

        transform::transform(&model_srid, &wgs84, &mut coordinates).unwrap();

        geom.insert(
            4326,
            Geometry {
                srid: 4326,
                x: coordinates.0.to_degrees(),
                y: coordinates.1.to_degrees(),
                z: coordinates.2,
            },
        );

        Self {
            id: model.id,
            name: model.name,
            geom,
        }
    }
}
