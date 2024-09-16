use crate::models::project::Entity as ProjectDB;
use crate::models::sensor::Entity as SensorDB;
use crate::models::soilprofile::Entity as SoilProfileDB;
use crate::models::transect::Entity as TransectDB;
use crate::models::transectnode::Entity as TransectNodeDB;
use crate::plots::models::Entity as PlotDB;

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::{query::*, DatabaseConnection};
use sea_query::Expr;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(ToSchema, Deserialize, Default)]
pub struct FilterOptions {
    pub filter: Option<String>, // JSON-encoded filter
    pub range: Option<String>,  // range in the format "[0,24]"
    pub sort: Option<String>,   // sort in the format '["id", "ASC"]'
}

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Plot {
    id: Uuid,
    name: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    coord_srid: Option<i32>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
}

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct SoilProfile {
    id: Uuid,
    name: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    coord_srid: Option<i32>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
}

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Sensor {
    id: Uuid,
    name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    coord_srid: Option<i32>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
}

#[derive(ToSchema, Serialize)]
pub struct TransectNode {
    id: Uuid,
    name: Option<String>,
    order: i32,
    plot: Plot,
}
#[derive(ToSchema, Serialize)]
pub struct Transect {
    id: Uuid,
    name: Option<String>,
    nodes: Vec<TransectNode>,
}

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Project {
    color: String,
    last_updated: NaiveDateTime,
    iterator: i32,
    description: Option<String>,
    id: Uuid,
    name: String,
}

#[derive(ToSchema, Serialize)]
pub struct Area {
    id: Uuid,
    last_updated: NaiveDateTime,
    name: Option<String>,
    description: Option<String>,
    project_id: Uuid,
    project: Project,
    soil_profiles: Vec<SoilProfile>,
    plots: Vec<Plot>,
    sensors: Vec<Sensor>,
    transects: Vec<Transect>,
    geom: Option<Value>,
}

// impl Sensor {
//     pub fn from(sensor_db: crate::models::sensor::Model) -> Self {
//         Sensor {
//             id: sensor_db.id,
//             name: sensor_db.name,
//             latitude: sensor_db.latitude,
//             longitude: sensor_db.longitude,
//             coord_srid: sensor_db.coord_srid,
//             coord_x: sensor_db.coord_x,
//             coord_y: sensor_db.coord_y,
//             coord_z: sensor_db.coord_z,
//         }
//     }
// }

impl Area {
    pub async fn from(area: crate::areas::models::Model, db: DatabaseConnection) -> Self {
        // Query for plots with matching area_id
        let plots: Vec<crate::areas::schemas::Plot> = PlotDB::find()
            .filter(crate::plots::models::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(plot.geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(plot.geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(plot.geom)"), "coord_z")
            .column_as(
                Expr::cust("ST_X(st_transform(plot.geom, 4326))"),
                "longitude",
            )
            .column_as(
                Expr::cust("ST_Y(st_transform(plot.geom, 4326))"),
                "latitude",
            )
            .column_as(Expr::cust("st_srid(plot.geom)"), "coord_srid")
            .into_model::<crate::areas::schemas::Plot>()
            .all(&db)
            .await
            .unwrap();

        // Query for sensors with matching area_id
        let sensors: Vec<crate::areas::schemas::Sensor> = SensorDB::find()
            .filter(crate::models::sensor::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(sensor.geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(sensor.geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(sensor.geom)"), "coord_z")
            .column_as(
                Expr::cust("ST_X(st_transform(sensor.geom, 4326))"),
                "longitude",
            )
            .column_as(
                Expr::cust("ST_Y(st_transform(sensor.geom, 4326))"),
                "latitude",
            )
            .column_as(Expr::cust("st_srid(sensor.geom)"), "coord_srid")
            .into_model::<crate::areas::schemas::Sensor>()
            .all(&db)
            .await
            .unwrap();

        // Query for transects with related transect nodes and their corresponding plots
        let transects: Vec<(
            crate::models::transect::Model,
            Vec<crate::models::transectnode::Model>,
        )> = TransectDB::find()
            .filter(crate::models::transect::Column::AreaId.eq(area.id))
            .find_with_related(TransectNodeDB)
            .all(&db)
            .await
            .unwrap();

        let mut transects_with_nodes: Vec<Transect> = Vec::new();
        for (transect, nodes) in transects {
            let mut transect_nodes: Vec<TransectNode> = Vec::new();

            for node in nodes {
                let plot: Plot = PlotDB::find()
                    .filter(crate::plots::models::Column::Id.eq(node.plot_id))
                    .column_as(Expr::cust("ST_X(plot.geom)"), "coord_x")
                    .column_as(Expr::cust("ST_Y(plot.geom)"), "coord_y")
                    .column_as(Expr::cust("ST_Z(plot.geom)"), "coord_z")
                    .column_as(
                        Expr::cust("ST_X(st_transform(plot.geom, 4326))"),
                        "longitude",
                    )
                    .column_as(
                        Expr::cust("ST_Y(st_transform(plot.geom, 4326))"),
                        "latitude",
                    )
                    .column_as(Expr::cust("st_srid(plot.geom)"), "coord_srid")
                    .into_model::<Plot>()
                    .one(&db)
                    .await
                    .unwrap()
                    .unwrap(); // Unwrapping safely assuming plot always exists

                transect_nodes.push(TransectNode {
                    id: node.id,
                    name: None, // `name` doesn't exist in the `transectnode::Model`
                    order: node.order,
                    plot,
                });
            }

            transects_with_nodes.push(Transect {
                id: transect.id,
                name: transect.name,
                nodes: transect_nodes,
            });
        }

        // Query for soil profiles with matching area_id
        let soil_profiles: Vec<crate::areas::schemas::SoilProfile> = SoilProfileDB::find()
            .filter(crate::models::soilprofile::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(soilprofile.geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(soilprofile.geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(soilprofile.geom)"), "coord_z")
            .column_as(
                Expr::cust("ST_X(st_transform(soilprofile.geom, 4326))"),
                "longitude",
            )
            .column_as(
                Expr::cust("ST_Y(st_transform(soilprofile.geom, 4326))"),
                "latitude",
            )
            .column_as(Expr::cust("st_srid(soilprofile.geom)"), "coord_srid")
            .into_model::<crate::areas::schemas::SoilProfile>()
            .all(&db)
            .await
            .unwrap();

        let project: crate::areas::schemas::Project = ProjectDB::find()
            .filter(crate::models::project::Column::Id.eq(area.project_id))
            .into_model::<crate::areas::schemas::Project>()
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        // Fetch convex hull geom for the area
        let geom: Option<Value> = crate::areas::services::get_convex_hull(&db, area.id).await;

        Area {
            id: area.id,
            name: area.name,
            description: area.description,
            project_id: area.project_id,
            last_updated: area.last_updated,
            plots,
            soil_profiles,
            sensors,
            // transects: transects_with_nodes, // Include transects with nodes
            transects: vec![],
            project,
            geom,
        }
    }
}
