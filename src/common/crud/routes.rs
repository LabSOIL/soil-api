use crate::common::crud::models::FilterOptions;
use crate::common::crud::traits::CRUDResource;
use crate::common::filter::{apply_filters, parse_range};
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use hyper::HeaderMap;
use sea_orm::{DatabaseConnection, DbErr, SqlErr};
use uuid::Uuid;

const RESOURCE_NAME: &str = "projects";

// #[utoipa::path(
//     get,
//     path = format!("/api/{}", <T as ApiResource>::RESOURCE_NAME),
//     responses((status = 200, body = Vec<T::ApiModel>))
// )]
// #[axum::debug_handler] // Helps with better compile-time error messages
pub async fn get_all<T>(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> Result<(HeaderMap, Json<Vec<T::ApiModel>>), (StatusCode, String)>
where
    T: CRUDResource,
{
    println!("Getting all {}", T::RESOURCE_NAME_PLURAL);
    let (offset, limit) = parse_range(params.range.clone());

    let condition = apply_filters(params.filter.clone(), T::filterable_columns());

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &T::sortable_columns(),
        T::default_index_column(),
    );

    let items = match T::get_all(
        &db,
        condition.clone(),
        order_column,
        order_direction,
        offset,
        limit,
    )
    .await
    {
        Ok(items) => items,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    };

    let total_count = T::total_count(&db, condition).await;

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);
    Ok((headers, Json(items)))
}

// Generic get_one function
// #[utoipa::path(
//     get,
//     path = format!("/api/{}/{{id}}", RESOURCE_NAME),
//     responses((status = 200, body = Project), (status = 404, body = String))
// )]
// #[axum::debug_handler] // Helps with better compile-time error messages
pub async fn get_one<T>(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<T::ApiModel>, (StatusCode, Json<String>)>
where
    T: CRUDResource,
{
    println!("Getting one {}", T::RESOURCE_NAME_PLURAL);
    match T::get_one(&db, id).await {
        Ok(item) => Ok(Json(item)),
        Err(DbErr::RecordNotFound(_)) => {
            Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal Server Error".to_string()),
        )),
    }
}

// Generic create_one function
pub async fn create_one<T>(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<T::CreateModel>,
) -> Result<(StatusCode, Json<T::ApiModel>), (StatusCode, Json<String>)>
where
    T: CRUDResource,
{
    println!("Creating one {}", T::RESOURCE_NAME_PLURAL);
    match T::create(&db, payload).await {
        Ok(created_item) => Ok((StatusCode::CREATED, Json(created_item))),
        Err(err) => match err.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                Err((StatusCode::CONFLICT, Json("Duplicate entry".to_string())))
            }
            Some(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error adding object".to_string()),
            )),
            _ => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Server error".to_string()),
            )),
        },
    }
}

// Generic update_one function
pub async fn update_one<T>(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<T::UpdateModel>,
) -> Result<Json<T::ApiModel>, (StatusCode, Json<String>)>
where
    T: CRUDResource,
{
    println!("Updating one {}", T::RESOURCE_NAME_PLURAL);
    match T::update(&db, id, payload).await {
        Ok(updated_item) => Ok(Json(updated_item)),
        Err(DbErr::RecordNotFound(_)) => {
            Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Error updating item".to_string()),
        )),
    }
}

// Generic delete_one function
pub async fn delete_one<T>(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Uuid>), (StatusCode, Json<String>)>
where
    T: CRUDResource,
{
    println!("Deleting one {}", T::RESOURCE_NAME_PLURAL);
    match T::delete(&db, id).await {
        Ok(rows_affected) if rows_affected > 0 => Ok((StatusCode::NO_CONTENT, Json(id))),
        Ok(_) => Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Error deleting item".to_string()),
        )),
    }
}

// Generic delete_many function
pub async fn delete_many<T>(
    State(db): State<DatabaseConnection>,
    Json(ids): Json<Vec<Uuid>>,
) -> Result<(StatusCode, Json<Vec<Uuid>>), (StatusCode, Json<String>)>
where
    T: CRUDResource,
{
    println!("Deleting many {}", T::RESOURCE_NAME_PLURAL);
    match T::delete_many(&db, ids.clone()).await {
        Ok(deleted_ids) => Ok((StatusCode::NO_CONTENT, Json(deleted_ids))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Error deleting items".to_string()),
        )),
    }
}
