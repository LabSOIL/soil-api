use super::db::Model;
use crate::areas;
use crate::plots::db::Gradientchoices;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use sea_orm::{
    entity::prelude::*, query::*, sea_query::Expr, ColumnTrait, DatabaseConnection, EntityTrait,
    FromQueryResult,
};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct PlotSimple {
    pub id: Uuid,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_srid: Option<i32>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
}

impl From<Model> for PlotSimple {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            latitude: None,
            longitude: None,
            coord_srid: None,
            coord_x: None,
            coord_y: None,
            coord_z: None,
        }
    }
}

impl PlotSimple {
    pub async fn from_db(plot: super::db::Model, db: &DatabaseConnection) -> Self {
        let plot = super::db::Entity::find()
            .filter(super::db::Column::Id.eq(plot.id))
            .column_as(Expr::cust("ST_X(geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
            .column_as(Expr::cust("ST_SRID(geom)"), "coord_srid")
            .column_as(Expr::cust("ST_X(st_transform(geom, 4326))"), "longitude")
            .column_as(Expr::cust("ST_Y(st_transform(geom, 4326))"), "latitude")
            .into_model::<PlotSimple>()
            .one(db)
            .await
            .unwrap()
            .unwrap();

        PlotSimple {
            id: plot.id,
            name: plot.name,
            latitude: plot.latitude,
            longitude: plot.longitude,
            coord_srid: plot.coord_srid,
            coord_x: plot.coord_x,
            coord_y: plot.coord_y,
            coord_z: plot.coord_z,
        }
    }

    pub async fn from_area(area: &crate::areas::db::Model, db: &DatabaseConnection) -> Vec<Self> {
        super::db::Entity::find()
            .filter(super::db::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
            .column_as(Expr::cust("ST_SRID(geom)"), "coord_srid")
            .column_as(Expr::cust("ST_X(st_transform(geom, 4326))"), "longitude")
            .column_as(Expr::cust("ST_Y(st_transform(geom, 4326))"), "latitude")
            .into_model::<PlotSimple>()
            .all(db)
            .await
            .unwrap()
    }
}
#[derive(ToSchema, Serialize)]
pub struct PlotBasicWithAreaAndProject {
    pub id: Uuid,
    pub name: String,
    pub area: crate::areas::models::AreaBasicWithProject,
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
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
    area: Area,
}

#[derive(ToSchema, FromQueryResult, Serialize)]
pub struct PlotWithCoords {
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
#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Area {
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
}

impl From<areas::db::Model> for Area {
    fn from(area_db: areas::db::Model) -> Self {
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
                name: None,
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
