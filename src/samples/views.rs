use crate::samples::models::PlotSample;
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
            get(crud::get_all::<PlotSample>).post(crud::create_one::<PlotSample>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<PlotSample>)
                .put(crud::update_one::<PlotSample>)
                .delete(crud::delete_one::<PlotSample>),
        )
        .route("/batch", delete(crud::delete_many::<PlotSample>))
        .with_state(db)
}
