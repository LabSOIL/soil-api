use crate::plots::models::Gradientchoices;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Deserialize, Default)]
pub struct FilterOptions {
    pub filter: Option<String>, // JSON-encoded filter
    pub range: Option<String>,  // range in the format "[0,24]"
    pub sort: Option<String>,   // sort in the format '["id", "ASC"]'
}

#[derive(ToSchema, Serialize)]
pub struct Area {
    id: Uuid,
    last_updated: NaiveDateTime,
    name: String,
    description: Option<String>,
    project_id: Uuid,
    // project: Project,
    // soil_profiles: Vec<SoilProfile>,
    // plots: Vec<Plot>,
    // sensors: Vec<Sensor>,
    // transects: Vec<Transect>,
    // geom: Option<String>,
}

#[derive(ToSchema, FromQueryResult, Serialize)]
pub struct AreaWithBoundary {
    // Represents the model of the query for get all plots with the extra
    // coordinate fields
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

// impl From<areas::models::Model> for Area {
//     fn from(area_db: areas::models::Model) -> Self {
//         Area {
//             id: area_db.id,
//             name: area_db.name,
//             description: area_db.description,
//         }
//     }
// }
// impl From<(PlotWithCoords, Option<Area>)> for Plot {
//     fn from((plot_db, area_db_vec): (PlotWithCoords, Option<Area>)) -> Self {
//         let area = area_db_vec.into_iter().next().map_or(
//             Area {
//                 id: Uuid::nil(),
//                 name: "Unknown".to_string(),
//                 description: None,
//             },
//             Area::from,
//         );

//         Plot {
//             id: plot_db.id,
//             name: plot_db.name,
//             plot_iterator: plot_db.plot_iterator,
//             area_id: plot_db.area_id,
//             gradient: plot_db.gradient,
//             vegetation_type: plot_db.vegetation_type,
//             topography: plot_db.topography,
//             aspect: plot_db.aspect,
//             created_on: plot_db.created_on,
//             weather: plot_db.weather,
//             lithology: plot_db.lithology,
//             iterator: plot_db.iterator,
//             last_updated: plot_db.last_updated,
//             image: plot_db.image,
//             coord_x: plot_db.coord_x,
//             coord_y: plot_db.coord_y,
//             coord_z: plot_db.coord_z,
//             area,
//         }
//     }
// }

// impl From<areas::models::Model> for Area {
//     fn from(area_db: areas::models::Model) -> Self {
//         Area {
//             id: area_db.id,
//             name: area_db.name,
//             description: area_db.description,
//         }
//     }
// }
impl From<crate::areas::models::Model> for Area {
    fn from(area_db: crate::areas::models::Model) -> Self {
        Area {
            id: area_db.id,
            name: area_db.name,
            description: area_db.description,
            project_id: area_db.project_id,
            // geom: area_db.geom,
            last_updated: area_db.last_updated,
        }
    }
}
