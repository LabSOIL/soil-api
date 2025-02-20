use super::models::Sensor;
// use crate::filter::{apply_filters, parse_range};
// use crate::models::FilterOptions;
// use crate::pagination::calculate_content_range;
// use crate::sort::generic_sort;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum::{
    routing::{delete, get},
    Router,
};
use crudcrate::routes as crud;
use crudcrate::CRUDResource;
use hyper::HeaderMap;
use sea_orm::{DatabaseConnection, DbErr, SqlErr};
use serde::Deserialize;
use uuid::Uuid;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<Sensor>).post(crud::create_one::<Sensor>),
        )
        .route(
            "/{id}",
            get(get_one::<Sensor>)
                .put(crud::update_one::<Sensor>)
                .delete(crud::delete_one::<Sensor>),
        )
        .route("/batch", delete(crud::delete_many::<Sensor>))
        .with_state(db)
}
pub async fn get_one<T>(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Query(query): Query<LowResolution>,
) -> Result<Json<T::ApiModel>, (StatusCode, Json<String>)>
where
    T: CRUDResource,
    <T as CRUDResource>::ApiModel: From<Sensor>,
{
    println!("High resolution: {}", query.high_resolution);
    if query.high_resolution {
        match T::get_one(&db, id).await {
            Ok(item) => Ok(Json(item)),
            Err(DbErr::RecordNotFound(_)) => {
                Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Internal Server Error".to_string()),
                ))
            }
        }
    } else {
        match Sensor::get_one_low_resolution(&db, id).await {
            Ok(item) => Ok(Json(item.into())),
            Err(DbErr::RecordNotFound(_)) => {
                Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Internal Server Error".to_string()),
                ))
            }
        }
    }
}
#[derive(Deserialize)]
pub struct LowResolution {
    #[serde(default)]
    pub high_resolution: bool,
}
