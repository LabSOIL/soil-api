use crate::plots::db::Entity as PlotDB;
use crate::plots::models::PlotSimple;
use crate::projects::db::Entity as ProjectDB;
use crate::projects::models::Project;
use crate::sensors::db::Entity as SensorDB;
use crate::soil::profiles::db::Entity as SoilProfileDB;
use crate::transects::db::Entity as TransectDB;
use crate::transects::models::Transect;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::{query::*, DatabaseConnection};
use sea_query::Expr;
use serde::Serialize;
use serde_json::Value;
use tracing_subscriber::registry::Data;
use utoipa::ToSchema;
use uuid::Uuid;

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
pub struct Area {
    id: Uuid,
    last_updated: NaiveDateTime,
    name: Option<String>,
    description: Option<String>,
    project_id: Uuid,
    project: Project,
    soil_profiles: Vec<SoilProfile>,
    plots: Vec<PlotSimple>,
    sensors: Vec<Sensor>,
    transects: Vec<Transect>,
    geom: Option<Value>,
}

#[derive(ToSchema, Serialize)]
pub struct AreaBasicWithProject {
    pub id: Uuid,
    pub name: Option<String>,
    pub project: crate::common::models::GenericNameAndID,
}

impl AreaBasicWithProject {
    pub async fn from(area_id: Uuid, db: DatabaseConnection) -> Self {
        let area = super::db::Entity::find()
            .filter(crate::areas::db::Column::Id.eq(area_id))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        let project = ProjectDB::find()
            .filter(crate::projects::db::Column::Id.eq(area.project_id))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        AreaBasicWithProject {
            id: area_id,
            name: area.name,
            project: crate::common::models::GenericNameAndID {
                id: project.id,
                name: project.name,
            },
        }
    }
}

// impl Sensor {
//     pub fn from(sensor_db: crate::db::sensor::Model) -> Self {
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
    pub async fn from(area: crate::areas::db::Model, db: DatabaseConnection) -> Self {
        // Query for plots with matching area_id
        let plots: Vec<PlotSimple> = PlotDB::find()
            .filter(crate::plots::db::Column::AreaId.eq(area.id))
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
            .into_model::<PlotSimple>()
            .all(&db)
            .await
            .unwrap();

        // Query for sensors with matching area_id
        let sensors: Vec<crate::areas::models::Sensor> = SensorDB::find()
            .filter(crate::sensors::db::Column::AreaId.eq(area.id))
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
            .into_model::<crate::areas::models::Sensor>()
            .all(&db)
            .await
            .unwrap();

        // Query for transects with related transect nodes and their corresponding plots
        let transects: Vec<crate::transects::db::Model> = TransectDB::find()
            .filter(crate::transects::db::Column::AreaId.eq(area.id))
            // .find_with_related(TransectNodeDB)
            .all(&db)
            .await
            .unwrap();

        let mut transects_with_nodes: Vec<Transect> = Vec::new();
        for transect in transects {
            transects_with_nodes.push(
                Transect::get_one(transect.id, &db)
                    .await
                    .expect("Transect not found"),
            );
        }

        // Query for soil profiles with matching area_id
        let soil_profiles: Vec<crate::areas::models::SoilProfile> = SoilProfileDB::find()
            .filter(crate::soil::profiles::db::Column::AreaId.eq(area.id))
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
            .into_model::<crate::areas::models::SoilProfile>()
            .all(&db)
            .await
            .unwrap();

        let project: crate::areas::models::Project = ProjectDB::find()
            .filter(crate::projects::db::Column::Id.eq(area.project_id))
            .into_model::<crate::areas::models::Project>()
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
            transects: transects_with_nodes, // Include transects with nodes
            project,
            geom,
        }
    }
}
