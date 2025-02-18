use crate::gnss::models::GNSS;
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
            get(crud::get_all::<GNSS>).post(crud::create_one::<GNSS>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<GNSS>)
                .put(crud::update_one::<GNSS>)
                .delete(crud::delete_one::<GNSS>),
        )
        .route("/batch", delete(crud::delete_many::<GNSS>))
        .with_state(db)
}
