use crate::soil::types::models::SoilType;
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
            get(crud::get_all::<SoilType>).post(crud::create_one::<SoilType>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<SoilType>)
                .put(crud::update_one::<SoilType>)
                .delete(crud::delete_one::<SoilType>),
        )
        .route("/batch", delete(crud::delete_many::<SoilType>))
        .with_state(db)
}
