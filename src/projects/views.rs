// use crate::common::models::FilterOptions;
// use crate::projects::db::Entity as ProjectDB;
// use crate::projects::models::Project;
// use axum::response::IntoResponse;
// use axum::{
//     extract::{Query, State},
//     http::header::HeaderMap,
//     routing, Json, Router,
// };
// use sea_orm::prelude::*;
// use sea_orm::Condition;
// use sea_orm::EntityTrait;
// use sea_orm::{query::*, DatabaseConnection};
// use sea_query::{Alias, Expr};
// use serde_json::json;
// use std::collections::HashMap;
// use uuid::Uuid;

// pub fn router(db: DatabaseConnection) -> Router {
//     Router::new()
//         .route("/", routing::get(get_all))
//         .with_state(db)
// }

// #[utoipa::path(get, path = "/api/projects", responses((status = OK, body = PlotWithCoords)))]
// pub async fn get_all(
//     Query(params): Query<FilterOptions>,
//     State(db): State<DatabaseConnection>,
// ) -> impl IntoResponse {
//     // Default values for range and sorting
//     let default_sort_column = "id";
//     let default_sort_order = "ASC";

//     // 1. Parse the filter, range, and sort parameters
//     let filters: HashMap<String, String> = if let Some(filter) = params.filter {
//         serde_json::from_str(&filter).unwrap_or_default()
//     } else {
//         HashMap::new()
//     };

//     let (offset, limit) = if let Some(range) = params.range {
//         let range_vec: Vec<u64> = serde_json::from_str(&range).unwrap_or(vec![0, 24]); // Default to [0, 24]
//         let start = range_vec.get(0).copied().unwrap_or(0);
//         let end = range_vec.get(1).copied().unwrap_or(24);
//         let limit = end - start + 1;
//         (start, limit) // Offset is `start`, limit is the number of documents to fetch
//     } else {
//         (0, 25) // Default to 25 documents starting at 0
//     };

//     let (sort_column, sort_order) = if let Some(sort) = params.sort {
//         let sort_vec: Vec<String> = serde_json::from_str(&sort).unwrap_or(vec![
//             default_sort_column.to_string(),
//             default_sort_order.to_string(),
//         ]);
//         (
//             sort_vec
//                 .get(0)
//                 .cloned()
//                 .unwrap_or(default_sort_column.to_string()),
//             sort_vec
//                 .get(1)
//                 .cloned()
//                 .unwrap_or(default_sort_order.to_string()),
//         )
//     } else {
//         (
//             default_sort_column.to_string(),
//             default_sort_order.to_string(),
//         )
//     };

//     // Apply filters
//     let mut condition = Condition::all();
//     for (key, mut value) in filters {
//         value = value.trim().to_string();

//         // Check if the value is a valid UUID
//         if let Ok(uuid) = Uuid::parse_str(&value) {
//             // If the value is a valid UUID, filter it as a UUID
//             condition = condition.add(Expr::col(Alias::new(&key)).eq(uuid));
//         } else {
//             // Otherwise, treat it as a regular string filter
//             condition = condition.add(Expr::col(Alias::new(&key)).eq(value));
//         }
//     }

//     // Query with filtering, sorting, and pagination
//     let order_direction = if sort_order == "ASC" {
//         Order::Asc
//     } else {
//         Order::Desc
//     };
//     let order_column = match sort_column.as_str() {
//         "id" => <ProjectDB as sea_orm::EntityTrait>::Column::Id,
//         "name" => <ProjectDB as sea_orm::EntityTrait>::Column::Name,
//         "description" => <ProjectDB as sea_orm::EntityTrait>::Column::Description,
//         "color" => <ProjectDB as sea_orm::EntityTrait>::Column::Color,
//         "iterator" => <ProjectDB as sea_orm::EntityTrait>::Column::Iterator,
//         "last_updated" => <ProjectDB as sea_orm::EntityTrait>::Column::LastUpdated,
//         _ => <ProjectDB as sea_orm::EntityTrait>::Column::Id,
//     };

//     let objs = ProjectDB::find()
//         .filter(condition)
//         .order_by(order_column, order_direction)
//         .offset(offset)
//         .limit(limit)
//         .all(&db)
//         .await
//         .unwrap();

//     let projects: Vec<Project> = objs
//         .iter()
//         .map(|project| Project::from(project.clone()))
//         .collect::<Vec<Project>>();

//     let total_count: u64 = ProjectDB::find().count(&db).await.unwrap();
//     let max_offset_limit = (offset + limit).min(total_count);
//     let content_range = format!(
//         "projects {}-{}/{}",
//         offset,
//         max_offset_limit - 1,
//         total_count
//     );

//     // Return the Content-Range as a header
//     let mut headers = HeaderMap::new();
//     headers.insert("Content-Range", content_range.parse().unwrap());
//     (headers, Json(json!(projects)))
// }

// use crate::generate_router;

// generate_router!(
//     resource_name: "projects",
//     db_entity: crate::projects::db::Entity,
//     db_model: crate::projects::db::Model,
//     active_model: crate::projects::db::ActiveModel,
//     db_columns: crate::projects::db::Column,
//     get_one_response_model: crate::projects::models::Project,
//     get_all_response_model: crate::projects::models::Project,
//     create_one_request_model: crate::projects::models::ProjectCreate,
//     update_one_request_model: crate::projects::models::ProjectUpdate,
//     order_column_logic: [
//         ("id", crate::projects::db::Column::Id),
//         ("name", crate::projects::db::Column::Name),
//         ("description", crate::projects::db::Column::Description),
//         ("color", crate::projects::db::Column::Color),
//         ("iterator", crate::projects::db::Column::Iterator),
//         ("last_updated", crate::projects::db::Column::LastUpdated),
//     ],
//     searchable_columns: [
//         ("name", crate::projects::db::Column::Name),
//         ("description", crate::projects::db::Column::Description),
//         ("color", crate::projects::db::Column::Color),
//         ("iterator", crate::projects::db::Column::Iterator),
//     ]
// );

use crate::common::auth::Role;
use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use crate::common::traits::ApiResource;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use itertools::izip;
use sea_orm::{
    query::*, ActiveModelTrait, DatabaseConnection, DbBackend, DeleteResult, EntityTrait,
    LoaderTrait, ModelTrait, SqlErr,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use uuid::Uuid;

use crate::projects::models::Project;

const RESOURCE_NAME: &str = "projects";

// Generic get_all function
#[utoipa::path(
    get,
    path = format!("/api/{}", RESOURCE_NAME),
    responses((status = 200, body = Vec<Project>))
)]
pub async fn get_all<T: ApiResource>(
    Query(params): Query<FilterOptions>,
    State(db): State<Arc<DatabaseConnection>>,
) -> impl IntoResponse {
    let (offset, limit) = parse_range(params.range.clone());

    let condition = apply_filters(params.filter.clone(), T::filterable_columns());

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &T::sortable_columns(),
        T::default_sort_column(),
    );

    let items = T::get_all(&db, condition, order_column, order_direction, offset, limit).await;

    let total_count = T::EntityType::find()
        .filter(condition.clone())
        .count(&db)
        .await
        .unwrap_or(0);

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);
    (headers, Json(items))
}

// Generic get_one function
#[utoipa::path(
    get,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = 200, body = Project), (status = 404, body = String))
)]
pub async fn get_one<T: ApiResource>(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<Uuid>,
) -> Result<Json<T::ApiModel>, (StatusCode, Json<String>)> {
    match T::get_one(&db, id).await {
        Some(item) => Ok(Json(item)),
        None => Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
    }
}

// Similarly implement create_one, update_one, delete_one, delete_many using T: ApiResource

// Router setup
pub fn router(db: DatabaseConnection) -> Router {
    let db = Arc::new(db);
    Router::new()
        .route(
            "/",
            routing::get(get_all::<Project>).post(create_one::<Project>),
        )
        .route(
            "/:id",
            routing::get(get_one::<Project>)
                .put(update_one::<Project>)
                .delete(delete_one::<Project>),
        )
        .route("/batch", routing::delete(delete_many::<Project>))
        .with_state(db)
}
