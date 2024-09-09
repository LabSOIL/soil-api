// use crate::models::plot::PlotRead;
// use crate::models::plot::Model as Plot;
// use crate::schemas::plot::FilterOptions;
use crate::models;
use axum::{
    // extract::{Query, State},
    extract::State,
    // http::{HeaderMap, HeaderValue, StatusCode},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use models::plot::Entity as Plot;
use sea_orm::DbConn;
use sea_orm::EntityTrait;
use serde::ser;
use serde_json;
use sqlx::PgPool; // Add this import

pub async fn get_plots(
    // opts: Option<Query<FilterOptions>>,
    State(db): State<DbConn>,
    // Return the plots
) -> impl IntoResponse {
    // let plots: Vec<models::plot::Model> = Plot::find().into_json().all(&db).await.unwrap();
    let plots = Plot::find().into_json().all(&db).await.unwrap();
    println!("{:?}", plots);
    // Ok(Json(plots))
}
