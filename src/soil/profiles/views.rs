use crate::common::crud::routes as crud;
use crate::soil::profiles::models::SoilProfile;
use axum::{
    routing::{delete, get},
    Router,
};
use sea_orm::DatabaseConnection;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route(
            "/",
            get(crud::get_all::<SoilProfile>).post(crud::create_one::<SoilProfile>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<SoilProfile>)
                .put(crud::update_one::<SoilProfile>)
                .delete(crud::delete_one::<SoilProfile>),
        )
        .route("/batch", delete(crud::delete_many::<SoilProfile>))
        .with_state(db)
}
