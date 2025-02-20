use super::models::SensorProfileAssignment;
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
            get(crud::get_all::<SensorProfileAssignment>)
                .post(crud::create_one::<SensorProfileAssignment>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<SensorProfileAssignment>)
                .put(crud::update_one::<SensorProfileAssignment>)
                .delete(crud::delete_one::<SensorProfileAssignment>),
        )
        .route(
            "/batch",
            delete(crud::delete_many::<SensorProfileAssignment>),
        )
        .with_state(db)
}
