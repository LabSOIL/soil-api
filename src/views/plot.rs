use crate::models::{self, area};
use crate::schemas::plot::FilterOptions;

use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    http::{
        header::{self, HeaderMap, HeaderName},
        StatusCode, Uri,
    },
    routing, Json, Router,
};

use geo_types::Point;
use models::area::Entity as AreaDB;
use models::plot::Entity as PlotDB;
use models::sea_orm_active_enums::Gradientchoices;
use sea_orm::sqlx::Result;
use sea_orm::{query::*, DatabaseConnection};
use sea_orm::{DbConn, EntityTrait};
use sea_query::{Alias, Expr, Func};
use serde_json::{json, Value};
use tracing_subscriber::util::SubscriberInitExt;
use wkt::types::Coord;
// use axum::{Json, Router};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
// use utoipa_axum::router::OpenApiRouter;
// use utoipa_axum::routes;
use std::sync::Arc;
use uuid::Uuid;
use wkt::Wkt;
#[derive(OpenApi)]
#[openapi(components(schemas(Plot)))]
pub struct PlotApi;

impl Serialize for Gradientchoices {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let gradient = match self {
            Gradientchoices::Flat => "Flat",
            Gradientchoices::Slope => "Slope",
        };
        serializer.serialize_str(gradient)
    }
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
    created_on: Option<String>,
    weather: Option<String>,
    lithology: Option<String>,
    iterator: i32,
    last_updated: String,
    image: Option<String>,
    // coord_x: Option<f64>,
    // coord_y: Option<f64>,
    // coord_z: Option<f64>,
    area: Area,
}

#[derive(ToSchema, Serialize)]
pub struct Area {
    id: Uuid,
    name: String,
    description: Option<String>,
}

impl From<models::area::Model> for Area {
    fn from(area_db: models::area::Model) -> Self {
        Area {
            id: area_db.id,
            name: area_db.name,
            description: area_db.description,
        }
    }
}
impl
    From<(
        models::plot::Model,
        Vec<models::area::Model>,
        // Option<f64>,
        // Option<f64>,
        // Option<f64>,
    )> for Plot
{
    fn from(
        (
            plot_db,
            area_db_vec,
            //  coord_x, coord_y, coord_z
        ): (
            models::plot::Model,
            Vec<models::area::Model>,
            // Option<f64>,
            // Option<f64>,
            // Option<f64>,
        ),
    ) -> Self {
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
            created_on: plot_db.created_on.map(|d| d.to_string()),
            weather: plot_db.weather,
            lithology: plot_db.lithology,
            iterator: plot_db.iterator,
            last_updated: plot_db.last_updated.to_string(),
            image: plot_db.image,
            // coord_x, // Manually setting coord_x
            // coord_y, // Manually setting coord_y
            // coord_z, // Manually setting coord_z
            area,
        }
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_plots))
        .with_state(db)
}

#[utoipa::path(get, path = "", responses((status = OK, body = Plots)))]
pub async fn get_plots(
    opts: Option<Query<FilterOptions>>,
    State(db): State<DbConn>,
) -> impl IntoResponse {
    let Query(opts) = opts.unwrap_or_default();
    println!("Requested all plots");
    let limit: u64 = opts.limit.unwrap_or(10);
    let offset: u64 = opts.offset.unwrap_or(0);

    let objs = PlotDB::find()
        .column_as(Expr::cust("ST_X(plot.geom)"), "coord_x")
        .column_as(Expr::cust("ST_Y(plot.geom)"), "coord_y")
        .column_as(Expr::cust("ST_Z(plot.geom)"), "coord_z")
        .column_as(Expr::cust("ST_AsEWKT(plot.geom)"), "geom")
        .find_with_related(AreaDB)
        .all(&db)
        .await
        .unwrap(); // Add error handling as needed

    // Map the results from the database models to the Plot struct
    let plots: Vec<Plot> = objs
        .into_iter()
        .map(|(plot, areas)| {
            // // Extract virtual columns
            // let coord_x = plot.get("coord_x").and_then(|v| v.as_f64());
            // let coord_y = plot.get("coord_y").and_then(|v| v.as_f64());
            // let coord_z = plot.get("coord_z").and_then(|v| v.as_f64());

            Plot::from((
                plot, areas,
                //  coord_x, coord_y, coord_z
            ))
        })
        .collect();

    let total_plots: u64 = PlotDB::find().count(&db).await.unwrap();
    let content_range: String;
    let max_offset_limit = if total_plots > offset + limit {
        offset + limit
    } else {
        total_plots
    };
    content_range = format!("plots {}-{}/{}", offset, max_offset_limit, total_plots);

    // // Return the Content-Range as a header
    let mut headers = HeaderMap::new();
    headers.insert("Content-Range", content_range.parse().unwrap());
    // // Return JSON response
    (headers, Json(json!(plots)))
}
