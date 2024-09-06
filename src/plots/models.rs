use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct Plot {
    pub id: Uuid,
    pub name: String,
    pub plot_iterator: Option<i32>,
    pub area_id: Uuid,
    pub gradient: Option<String>, // Now treated as a string
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    pub created_on: Option<NaiveDate>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
    pub image: Option<String>,
    pub geom: Option<String>, // Geometry as WKT String
}

#[derive(Serialize, Debug)]
pub struct PlotRead {
    pub id: Uuid,
    pub geom: Option<String>, // WKT Geometry String
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    pub coord_srid: Option<i32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub name: Option<String>,
    pub area: Option<String>,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    pub created_on: Option<NaiveDate>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
    pub image: Option<String>,
}
