use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct Plot {
    pub id: Uuid,
    pub name: String,
    pub plot_iterator: Option<i32>,
    pub area_id: Uuid,
    pub gradient: Option<String>,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    // pub created_on: Option<NaiveDateTime>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    pub iterator: Option<i32>,
    // pub last_updated: Option<NaiveDateTime>,
    pub image: Option<String>,
    // pub geom: Option<String>,
}
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct AreaRead {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct PlotRead {
    pub id: Uuid,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub name: Option<String>,
    pub area_id: Uuid,
    pub gradient: Option<String>,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    pub created_on: Option<NaiveDate>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
    pub image: Option<String>,
    pub iterator: Option<i32>,
    pub plot_iterator: Option<i32>,
    pub area_name: Option<String>, // Include area in PlotRead
    pub area_description: Option<String>,
    pub area_project_id: Option<Uuid>,
    // pub area: Option<Vec<AreaRead>>, // Include area in PlotRead
}
