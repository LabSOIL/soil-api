// use crate::common::crud::routes as crud;
// use crate::sensors::models::Sensor;
// use axum::{
//     routing::{delete, get, put},
//     Router,
// };
// use sea_orm::DatabaseConnection;

// pub fn router(db: DatabaseConnection) -> Router {
//     Router::new()
//         .route(
//             "/",
//             get(crud::get_all::<Sensor>).post(crud::create_one::<Sensor>),
//         )
//         .route(
//             "/{id}",
//             get(crud::get_one::<Sensor>)
//                 .put(crud::update_one::<Sensor>)
//                 .delete(crud::delete_one::<Sensor>),
//         )
//         .route("/batch", delete(crud::delete_many::<Sensor>))
//         .with_state(db)
// }
