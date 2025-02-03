// use crate::common::{
//     crud::models::FilterOptions,
//     filter::{apply_filters, parse_range},
//     pagination::calculate_content_range,
//     sort::generic_sort,
// };
// use axum::{
//     extract::{Path, Query, State},
//     http::StatusCode,
//     response::IntoResponse,
//     routing, Json, Router,
// };
// use sea_orm::{
//     query::*, ActiveModelTrait, DatabaseConnection, DeleteResult, EntityTrait, ModelTrait, SqlErr,
// };
// use uuid::Uuid;

// const RESOURCE_NAME: &str = "areas";

// pub fn router(db: DatabaseConnection) -> Router {
//     Router::new()
//         .route("/", routing::get(get_all))
//         .route("/{id}", routing::get(get_one))
//         .route("/", routing::post(create_one))
//         .route("/{id}", routing::put(update_one).delete(delete_one))
//         .route("/batch", routing::delete(delete_many))
//         .with_state(db.clone())
// }

// pub async fn get_all(
//     Query(params): Query<FilterOptions>,
//     State(db): State<DatabaseConnection>,
// ) -> impl IntoResponse {
//     let (offset, limit) = parse_range(params.range.clone());

//     let condition = apply_filters(params.filter.clone(), &[("name", super::db::Column::Name)]);

//     let (order_column, order_direction) = generic_sort(
//         params.sort.clone(),
//         &[
//             ("id", super::db::Column::Id),
//             ("name", super::db::Column::Name),
//         ],
//         super::db::Column::Id,
//     );
//     let areas = super::models::Area::get_all(
//         db.clone(),
//         condition.clone(),
//         order_column.clone(),
//         order_direction.clone(),
//         offset,
//         limit,
//     )
//     .await;

//     let total_count: u64 = <super::db::Entity>::find()
//         .filter(condition.clone())
//         .select_only()
//         .column(super::db::Column::Id)
//         .count(&db)
//         .await
//         .unwrap_or(0);

//     let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);
//     (headers, Json(areas))
// }

// pub async fn get_one(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
// ) -> Result<Json<super::models::Area>, (StatusCode, Json<String>)> {
//     let area = super::models::Area::get_one(id, db).await;
//     println!("{:}", area.id);

//     Ok(Json(area))
// }

// pub async fn create_one(
//     State(db): State<DatabaseConnection>,
//     Json(payload): Json<super::models::AreaCreate>,
// ) -> Result<(StatusCode, Json<super::models::Area>), (StatusCode, Json<String>)> {
//     let new_obj: super::db::ActiveModel = payload.into();

//     match super::db::Entity::insert(new_obj).exec(&db).await {
//         Ok(insert_result) => {
//             let response_obj: super::models::Area =
//                 get_one(State(db.clone()), Path(insert_result.last_insert_id))
//                     .await
//                     .unwrap()
//                     .0;

//             Ok((StatusCode::CREATED, Json(response_obj)))
//         }
//         Err(err) => match err.sql_err() {
//             Some(SqlErr::UniqueConstraintViolation(_)) => {
//                 Err((StatusCode::CONFLICT, Json("Duplicate entry".to_string())))
//             }
//             Some(_) => Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json("Error adding object".to_string()),
//             )),
//             _ => Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json("Server error".to_string()),
//             )),
//         },
//     }
// }

// pub async fn update_one(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
//     Json(payload): Json<super::models::AreaUpdate>,
// ) -> impl IntoResponse {
//     let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
//         .one(&db)
//         .await
//         .unwrap()
//         .expect("Failed to find object")
//         .into();

//     let updated_obj: super::db::ActiveModel = payload.merge_into_activemodel(db_obj);
//     let response_obj = updated_obj.update(&db).await.unwrap();

//     // Assert response is ok
//     assert_eq!(response_obj.id, id);

//     // Return the new object
//     let obj = get_one(State(db.clone()), Path(id.clone()))
//         .await
//         .unwrap()
//         .0;

//     Json(obj)
// }

// pub async fn delete_one(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
// ) -> Result<(StatusCode, Json<Uuid>), (StatusCode, Json<String>)> {
//     let obj = match super::db::Entity::find_by_id(id.clone()).one(&db).await {
//         Ok(Some(obj)) => obj,
//         Ok(None) => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
//         _ => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
//     };

//     let res: DeleteResult = obj.delete(&db).await.map_err(|_| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json("Failed to delete object".to_string()),
//         )
//     })?;

//     if res.rows_affected == 0 {
//         return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
//     }

//     Ok((StatusCode::NO_CONTENT, Json(id)))
// }

// pub async fn delete_many(
//     State(db): State<DatabaseConnection>,
//     Json(ids): Json<Vec<Uuid>>,
// ) -> Result<(StatusCode, Json<Vec<Uuid>>), (StatusCode, Json<String>)> {
//     let mut deleted_ids = Vec::new();
//     for id in ids {
//         let obj = match super::db::Entity::find_by_id(id.clone()).one(&db).await {
//             Ok(Some(obj)) => obj,
//             Ok(None) => continue,
//             Err(_) => {
//                 return Err((
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     Json("Failed to delete objects".to_string()),
//                 ))
//             }
//         };

//         let res: DeleteResult = obj.delete(&db).await.map_err(|_| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json("Failed to delete object".to_string()),
//             )
//         })?;

//         if res.rows_affected > 0 {
//             deleted_ids.push(id);
//         }
//     }

//     Ok((StatusCode::NO_CONTENT, Json(deleted_ids)))
// }

use crate::areas::models::Area;
use crate::common::crud::routes as crud;
use axum::{
    routing::{delete, get},
    Router,
};
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
