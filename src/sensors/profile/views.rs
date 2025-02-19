use super::models::SensorProfile;
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
            get(crud::get_all::<SensorProfile>).post(crud::create_one::<SensorProfile>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<SensorProfile>)
                .put(crud::update_one::<SensorProfile>)
                .delete(crud::delete_one::<SensorProfile>),
        )
        .route("/batch", delete(crud::delete_many::<SensorProfile>))
        .with_state(db)
}
