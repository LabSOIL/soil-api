use crate::common::crud::routes as crud;
use axum::{
    routing::{delete, get},
    Router,
};
use sea_orm::DatabaseConnection;

use super::models::Transect;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<Transect>).post(crud::create_one::<Transect>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Transect>)
                .put(crud::update_one::<Transect>)
                .delete(crud::delete_one::<Transect>),
        )
        .route("/batch", delete(crud::delete_many::<Transect>))
        .with_state(db)
}
