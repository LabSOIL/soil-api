use proj4rs::{Proj, transform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Geometry {
    pub srid: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<(i32, f64, f64, f64)> for Geometry {
    fn from(geom: (i32, f64, f64, f64)) -> Self {
        Self {
            srid: geom.0,
            x: geom.1,
            y: geom.2,
            z: geom.3,
        }
    }
}
impl From<Geometry> for (i32, f64, f64, f64) {
    fn from(geom: Geometry) -> Self {
        (geom.srid, geom.x, geom.y, geom.z)
    }
}

impl Geometry {
    pub fn to_hashmap(&self, additional_projections: Vec<i32>) -> HashMap<i32, Self> {
        let mut geom = HashMap::new();

        // Add the original SRID and coordinates
        geom.insert(
            self.srid,
            Geometry {
                srid: self.srid,
                x: self.x,
                y: self.y,
                z: self.z,
            },
        );

        for srid in additional_projections {
            if srid == self.srid {
                continue;
            }
            // Transform coordinates to supply also WGS84 for mapping
            let wgs84: Proj = Proj::from_epsg_code(4326).unwrap();
            let model_srid: Proj = Proj::from_epsg_code(srid.try_into().unwrap()).unwrap();
            let mut coordinates = (self.x, self.y, self.z);

            transform::transform(&model_srid, &wgs84, &mut coordinates).unwrap();

            geom.insert(
                srid,
                Geometry {
                    srid,
                    x: coordinates.0.to_degrees(),
                    y: coordinates.1.to_degrees(),
                    z: coordinates.2,
                },
            );
        }
        geom
    }
}
