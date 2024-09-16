use crate::areas;
use crate::plots::models::Gradientchoices;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Default)]
pub struct FilterOptions {
    pub filter: Option<String>, // JSON-encoded filter
    pub range: Option<String>,  // range in the format "[0,24]"
    pub sort: Option<String>,   // sort in the format '["id", "ASC"]'
}

#[derive(ToSchema, Serialize)]
pub struct Plot {
    id: Uuid,
    name: String,
    plot_iterator: i32,
    area_id: Uuid,
    gradient: Gradientchoices,
    vegetation_type: Option<String>,
    topography: Option<String>,
    aspect: Option<String>,
    created_on: Option<NaiveDate>,
    weather: Option<String>,
    lithology: Option<String>,
    iterator: i32,
    last_updated: NaiveDateTime,
    image: Option<String>,
    area: Area,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
}

#[derive(FromQueryResult, Serialize)]
pub struct PlotWithCoords {
    id: Uuid,
    name: String,
    plot_iterator: i32,
    area_id: Uuid,
    gradient: Gradientchoices,
    vegetation_type: Option<String>,
    topography: Option<String>,
    aspect: Option<String>,
    created_on: Option<NaiveDate>,
    weather: Option<String>,
    lithology: Option<String>,
    iterator: i32,
    last_updated: NaiveDateTime,
    image: Option<String>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
}
#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Area {
    id: Uuid,
    name: String,
    description: Option<String>,
}

impl From<areas::models::Model> for Area {
    fn from(area_db: areas::models::Model) -> Self {
        Area {
            id: area_db.id,
            name: area_db.name,
            description: area_db.description,
        }
    }
}
impl From<(PlotWithCoords, Option<Area>)> for Plot {
    fn from((plot_db, area_db_vec): (PlotWithCoords, Option<Area>)) -> Self {
        let area = area_db_vec.into_iter().next().map_or(
            Area {
                id: Uuid::nil(),
                name: "Unknown".to_string(),
                description: None,
            },
            Area::from,
        );

        Plot {
            id: plot_db.id,
            name: plot_db.name,
            plot_iterator: plot_db.plot_iterator,
            area_id: plot_db.area_id,
            gradient: plot_db.gradient,
            vegetation_type: plot_db.vegetation_type,
            topography: plot_db.topography,
            aspect: plot_db.aspect,
            created_on: plot_db.created_on,
            weather: plot_db.weather,
            lithology: plot_db.lithology,
            iterator: plot_db.iterator,
            last_updated: plot_db.last_updated,
            image: plot_db.image,
            coord_x: plot_db.coord_x,
            coord_y: plot_db.coord_y,
            coord_z: plot_db.coord_z,
            area,
        }
    }
}
