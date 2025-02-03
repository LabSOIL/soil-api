use crate::common::crud::routes as crud;
use crate::plots::models::Plot;
use axum::{
    routing::{delete, get},
    Router,
};
use sea_orm::DatabaseConnection;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<Plot>).post(crud::create_one::<Plot>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Plot>)
                .put(crud::update_one::<Plot>)
                .delete(crud::delete_one::<Plot>),
        )
        .route("/batch", delete(crud::delete_many::<Plot>))
        .with_state(db)
}
