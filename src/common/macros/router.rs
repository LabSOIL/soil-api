#[macro_export]
macro_rules! generate_router {
    (
        resource_name: $resource_name:expr,
        db_entity: $db_entity:ty,
        db_model: $db_model:ty,
        active_model: $active_model:ty,
        db_columns: $db_columns:ty,
        get_one_response_model: $get_one_response_model:ty,
        get_all_response_model: $get_all_response_model:ty,
        create_one_request_model: $create_one_request_model:ty,
        order_column_logic: $order_column_logic:expr,
        searchable_columns: $searchable_columns:expr
    ) => {
        use crate::common::models::FilterOptions;
        use crate::common::sort::generic_sort;
        use crate::common::filter::{parse_range, apply_filters};
        use crate::common::pagination::calculate_content_range;
        use axum::extract::Path;
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        use axum::{
            extract::{Query, State},
            routing, Json, Router,
        };
        use sea_orm::query::*;
        use sea_orm::ColumnTrait;
        use sea_orm::{DatabaseConnection, EntityTrait,ActiveModelTrait};
        use std::iter::Iterator;
        use uuid::Uuid;
        use futures::StreamExt;

        pub fn router(db: DatabaseConnection) -> Router {
            Router::new()
                .route("/", routing::get(get_all).post(create_one))
                .route("/:id", routing::get(get_one))
                .with_state(db)
        }

        #[utoipa::path(get, path = format!("/v1/{}", $resource_name), responses((status = 200, body = $get_all_response_model)))]
        pub async fn get_all(
            Query(params): Query<FilterOptions>,
            State(db): State<DatabaseConnection>,
        ) -> impl IntoResponse {

            let (offset, limit) = parse_range(params.range.clone());

            // Apply filters
            let condition = apply_filters(params.filter.clone(), &$searchable_columns);

            // Apply sorting
            let (order_column, order_direction) = generic_sort(
                params.sort.clone(),
                &$order_column_logic[..],
                <$db_columns>::Id,
            );

            // Do the query
            let objs: Vec<$db_model> = <$db_entity>::find()
                .filter(condition.clone())
                .order_by(order_column, order_direction)
                .offset(offset)
                .limit(limit)
                .all(&db)
                .await
                .unwrap();

            // Maps the database model to the response model
            let objs: Vec<$get_all_response_model> = futures::stream::iter(objs)
                .then(|obj| <$get_all_response_model>::from_db(obj, &db) )
                .collect()
                .await;

            // Get total count for content range header
            let total_count: u64 = <$db_entity>::find()
                .filter(condition.clone())
                .count(&db)
                .await
                .unwrap_or(0);

            let headers = calculate_content_range(
                offset,
                limit,
                total_count,
                $resource_name,
            );

            (headers, Json(objs))
        }

        #[utoipa::path(get, path = concat!("/v1/",$resource_name, "/{id}"), responses((status = 200, body = $get_one_response_model)))]
        pub async fn get_one(
            State(db): State<DatabaseConnection>,
            Path(id): Path<Uuid>,
        ) -> impl IntoResponse {
            let obj: Option<$db_model> = <$db_entity>::find()
                .filter(<$db_columns>::Id.eq(id))
                .one(&db)
                .await
                .unwrap();

            let response_obj: $get_one_response_model = <$get_one_response_model>::from_db(obj.unwrap(), &db).await;

            (StatusCode::OK, Json(response_obj))

        }

        #[utoipa::path(post, path = format!("/v1/{}", $resource_name), responses((status = 201, body = $get_one_response_model)))]
        pub async fn create_one(
            State(db): State<DatabaseConnection>,
            Json(payload): Json<$create_one_request_model>,
        ) -> impl IntoResponse {
            let db_obj: $active_model = <$active_model>::from_create(payload);
            let response = db_obj.insert(&db).await.unwrap();
            let response_obj = <$get_one_response_model>::from_db(response, &db).await;

            (StatusCode::CREATED, Json(response_obj))
        }
    };
}
