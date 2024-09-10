// use crate::models::plot::PlotRead;
// use crate::models::plot::Model as Plot;
// use crate::schemas::plot::FilterOptions;
use crate::models;
use crate::schemas::plot::FilterOptions;
use axum::{
    extract::{Query, State},
    Json,
};
use models::plot::Entity as Plot;
use sea_orm::DbConn;
use sea_orm::EntityTrait;
use sea_orm::{entity::*, query::*};
use serde_json;
use serde_json::{json, Value};
pub async fn get_plots(
    opts: Option<Query<FilterOptions>>,
    State(db): State<DbConn>,
) -> Json<Value> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let page = opts.page.unwrap_or(1);
    let offset = (page - 1) * limit;
    let plots = Plot::find().into_json().all(&db).await.unwrap();
    Json(json!(plots))
}
