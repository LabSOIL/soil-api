use crate::areas::models::Area;
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
            get(crud::get_all::<Area>).post(crud::create_one::<Area>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Area>)
                .put(crud::update_one::<Area>)
                .delete(crud::delete_one::<Area>),
        )
        .route("/batch", delete(crud::delete_many::<Area>))
        .with_state(db)
}
