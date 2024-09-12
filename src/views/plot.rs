use crate::models::{self};
use crate::schemas::plot::FilterOptions;
use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    http::header::HeaderMap,
    routing, Json, Router,
};
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use models::area::Entity as AreaDB;
use models::plot::Entity as PlotDB;
use models::sea_orm_active_enums::Gradientchoices;
use sea_orm::sqlx::Result;
use sea_orm::Condition;
use sea_orm::{query::*, DatabaseConnection};
use sea_orm::{EntityTrait, FromQueryResult};
use sea_query::{Alias, Expr};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

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

impl From<models::area::Model> for Area {
    fn from(area_db: models::area::Model) -> Self {
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

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_plots))
        .with_state(db)
}

#[utoipa::path(get, path = "", responses((status = OK, body = Plots)))]
pub async fn get_plots(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    // Default values for range and sorting
    // let default_limit = 10;
    // let default_offset = 0;
    let default_sort_column = "id";
    let default_sort_order = "ASC";

    // 1. Parse the filter, range, and sort parameters
    let filters: HashMap<String, String> = if let Some(filter) = params.filter {
        serde_json::from_str(&filter).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let (offset, limit) = if let Some(range) = params.range {
        let range_vec: Vec<u64> = serde_json::from_str(&range).unwrap_or(vec![0, 24]); // Default to [0, 24]
        let start = range_vec.get(0).copied().unwrap_or(0);
        let end = range_vec.get(1).copied().unwrap_or(24);
        let limit = (end - start + 1).min(25); // Calculate limit as `end - start + 1`, but cap at 25 if needed
        (start, limit) // Offset is `start`, limit is the number of documents to fetch
    } else {
        (0, 25) // Default to 25 documents starting at 0
    };

    let (sort_column, sort_order) = if let Some(sort) = params.sort {
        let sort_vec: Vec<String> = serde_json::from_str(&sort).unwrap_or(vec![
            default_sort_column.to_string(),
            default_sort_order.to_string(),
        ]);
        (
            sort_vec
                .get(0)
                .cloned()
                .unwrap_or(default_sort_column.to_string()),
            sort_vec
                .get(1)
                .cloned()
                .unwrap_or(default_sort_order.to_string()),
        )
    } else {
        (
            default_sort_column.to_string(),
            default_sort_order.to_string(),
        )
    };

    // 2. Apply the filters to your query
    let mut condition = Condition::all();
    for (key, mut value) in filters {
        println!("Key: {}, Value: {}", key, value);
        value = value.trim().to_string();

        // Check if the value is a valid UUID
        if let Ok(uuid) = Uuid::parse_str(&value) {
            // If the value is a valid UUID, filter it as a UUID
            condition = condition.add(Expr::col(Alias::new(&key)).eq(uuid));
        } else {
            // Otherwise, treat it as a regular string filter
            condition = condition.add(Expr::col(Alias::new(&key)).eq(value));
        }
    }

    // 3. Fetch the data from the database with filtering, sorting, and range (pagination)
    let order_direction = if sort_order == "ASC" {
        Order::Asc
    } else {
        Order::Desc
    };
    let order_column = match sort_column.as_str() {
        "id" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Id,
        "name" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Name,
        "plot_iterator" => <models::plot::Entity as sea_orm::EntityTrait>::Column::PlotIterator,
        "area_id" => <models::plot::Entity as sea_orm::EntityTrait>::Column::AreaId,
        "gradient" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Gradient,
        "vegetation_type" => <models::plot::Entity as sea_orm::EntityTrait>::Column::VegetationType,
        "topography" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Topography,
        "aspect" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Aspect,
        "created_on" => <models::plot::Entity as sea_orm::EntityTrait>::Column::CreatedOn,
        "weather" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Weather,
        "lithology" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Lithology,
        "iterator" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Iterator,
        "last_updated" => <models::plot::Entity as sea_orm::EntityTrait>::Column::LastUpdated,
        "image" => <models::plot::Entity as sea_orm::EntityTrait>::Column::Image,
        _ => <models::plot::Entity as sea_orm::EntityTrait>::Column::Id,
    };

    let objs = PlotDB::find()
        .filter(condition)
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .column_as(Expr::cust("ST_X(plot.geom)"), "coord_x")
        .column_as(Expr::cust("ST_Y(plot.geom)"), "coord_y")
        .column_as(Expr::cust("ST_Z(plot.geom)"), "coord_z")
        // .column_as(Expr::cust("ST_AsEWKT(plot.geom)"), "geom")
        .find_also_related(AreaDB)
        .into_model::<PlotWithCoords, Area>()
        // .into_json()
        .all(&db)
        .await
        .unwrap();

    // Map the results from the database models to the Plot struct
    let plots: Vec<Plot> = objs
        .into_iter()
        .map(|(plot, area)| Plot::from((plot, area)))
        .collect();

    let total_plots: u64 = PlotDB::find().count(&db).await.unwrap();
    let max_offset_limit = (offset + limit).min(total_plots);
    let content_range = format!("plots {}-{}/{}", offset, max_offset_limit - 1, total_plots);

    // Return the Content-Range as a header
    let mut headers = HeaderMap::new();
    headers.insert("Content-Range", content_range.parse().unwrap());
    (headers, Json(json!(plots)))
}
