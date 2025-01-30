use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use crate::common::traits::ApiResource;
use crate::projects::models::Project;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{
        // delete,
        get,
        //  post, put}, // Import necessary routing methods
    },
    Json, Router,
};
use hyper::HeaderMap;
use sea_orm::{DatabaseConnection, DbErr};
use uuid::Uuid;

const RESOURCE_NAME: &str = "projects";

#[utoipa::path(get,path = format!("/api/{}", RESOURCE_NAME),responses((status = 200, body = Vec<Project>)))]
// #[axum::debug_handler] // Helps with better compile-time error messages
pub async fn get_all<T>(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> Result<(HeaderMap, Json<Vec<T::ApiModel>>), (StatusCode, String)>
where
    T: ApiResource,
{
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
#[utoipa::path(
    get,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = 200, body = Project), (status = 404, body = String))
)]
// #[axum::debug_handler] // Helps with better compile-time error messages
pub async fn get_one<T>(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<T::ApiModel>, (StatusCode, Json<String>)>
where
    T: ApiResource,
{
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

// Similarly implement create_one, update_one, delete_one, delete_many using T: ApiResource

// Router setup
pub fn router(db: DatabaseConnection) -> Router {
    // let db = Arc::new(db);
    Router::new()
        .route(
            "/",
            get(get_all::<Project>), // .post(create_one::<Project>) // Uncomment and implement as needed
        )
        .route(
            "/{id}",
            get(get_one::<Project>), // .put(update_one::<Project>) // Uncomment and implement as needed
                                     // .delete(delete_one::<Project>), // Uncomment and implement as needed
        )
        // .route("/batch", delete(delete_many::<Project>)) // Uncomment and implement as needed
        .with_state(db)
}
