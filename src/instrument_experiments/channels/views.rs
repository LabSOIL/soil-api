use axum::{
    routing::{delete, get},
    Router,
};
use crudcrate::routes as crud;
use sea_orm::DatabaseConnection;

use super::models::InstrumentExperimentChannel;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<InstrumentExperimentChannel>)
                .post(crud::create_one::<InstrumentExperimentChannel>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<InstrumentExperimentChannel>)
                .put(crud::update_one::<InstrumentExperimentChannel>)
                .delete(crud::delete_one::<InstrumentExperimentChannel>),
        )
        .route(
            "/batch",
            delete(crud::delete_many::<InstrumentExperimentChannel>),
        )
        .with_state(db)
}
