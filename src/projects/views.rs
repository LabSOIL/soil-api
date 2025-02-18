use crate::projects::models::Project;
use axum::{
    routing::{delete, get},
    Router,
};
use crudcrate::routes as crud;
use sea_orm::DatabaseConnection;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<Project>).post(crud::create_one::<Project>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Project>)
                .put(crud::update_one::<Project>)
                .delete(crud::delete_one::<Project>),
        )
        .route("/batch", delete(crud::delete_many::<Project>))
        .with_state(db)
}
