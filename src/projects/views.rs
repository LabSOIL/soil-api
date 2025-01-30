use crate::common::crud::routes as crud;
use crate::projects::models::Project;
use axum::{routing::get, Router};
use sea_orm::DatabaseConnection;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<Project>),
            // .post(create_one::<Project>)
        )
        .route(
            "/{id}",
            get(crud::get_one::<Project>),
            // .put(update_one::<Project>)
            // .delete(delete_one::<Project>),
        )
        // .route("/batch", delete(delete_many::<Project>))
        .with_state(db)
}
