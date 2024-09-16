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
struct Plot {
    id: Uuid,
    name: String,
}

#[derive(ToSchema, Serialize)]
struct SoilProfile {
    id: Uuid,
    name: String,
}

#[derive(ToSchema, Serialize)]
struct Sensor {
    id: Uuid,
    name: Option<String>,
}

#[derive(ToSchema, Serialize)]
struct Transect {
    id: Uuid,
    name: Option<String>,
}

#[derive(ToSchema, Serialize)]
pub struct Area {
    id: Uuid,
    last_updated: NaiveDateTime,
    name: String,
    description: Option<String>,
    project_id: Uuid,
    // project: Project,
    soil_profiles: Vec<SoilProfile>,
    plots: Vec<Plot>,
    sensors: Vec<Sensor>,
    transects: Vec<Transect>,
    // geom: Option<String>,
}

impl Area {
    pub fn from(
        area_db: crate::areas::models::Model,
        plots_db: Vec<crate::plots::models::Model>,
        soilprofiles_db: Vec<crate::models::soilprofile::Model>,
        sensors_db: Vec<crate::models::sensor::Model>,
        transects_db: Vec<crate::models::transect::Model>,
    ) -> Self {
        let plots = plots_db
            .into_iter()
            .map(|plot_db| Plot {
                id: plot_db.id,
                name: plot_db.name,
            })
            .collect();

        let soil_profiles = soilprofiles_db
            .into_iter()
            .map(|soilprofile_db| SoilProfile {
                id: soilprofile_db.id,
                name: soilprofile_db.name,
            })
            .collect();

        let sensors = sensors_db
            .into_iter()
            .map(|sensor_db| Sensor {
                id: sensor_db.id,
                name: sensor_db.name,
            })
            .collect();

        let transects = transects_db
            .into_iter()
            .map(|transect_db| Transect {
                id: transect_db.id,
                name: transect_db.name,
            })
            .collect();

        Area {
            id: area_db.id,
            name: area_db.name,
            description: area_db.description,
            project_id: area_db.project_id,
            last_updated: area_db.last_updated,
            plots: plots,
            soil_profiles: soil_profiles,
            sensors: sensors,
            transects: transects,
            // geom: area_db.geom,
        }
    }
}
