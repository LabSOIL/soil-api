#[macro_export]
macro_rules! generate_router {
    (
        resource_name: $resource_name:expr,
        db_entity: $db_entity:ty,
        db_model: $db_model:ty,
        db_columns: $db_columns:ty,
        get_one_response_model: $get_one_response_model:ty,
        get_all_response_model: $get_all_response_model:ty,
        order_column_logic: $order_column_logic:expr,
        searchable_columns: $searchable_columns:expr // New argument for searchable fields
    ) => {
        use crate::common::models::FilterOptions;
        use axum::extract::Path;
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        use sea_query::{extension::postgres::PgExpr};
        use axum::{
            extract::{Query, State},
            http::header::HeaderMap,
            routing, Json, Router,
        };
        use sea_orm::query::*;
        use sea_orm::ColumnTrait;
        use sea_orm::{Condition, DatabaseConnection, EntityTrait};
        use sea_query::{Alias, Expr, Order};
        use std::collections::HashMap;
        use std::iter::Iterator;
        use uuid::Uuid;

        pub fn router(db: DatabaseConnection) -> Router {
            Router::new()
                .route("/", routing::get(get_all))
                .route("/:id", routing::get(get_one))
                .with_state(db)
        }

        #[utoipa::path(get, path = format!("/v1/{}", $resource_name), responses((status = 200, body = $get_all_response_model)))]
        pub async fn get_all(
            Query(params): Query<FilterOptions>,
            State(db): State<DatabaseConnection>,
        ) -> impl IntoResponse {
            // Default sorting and range values
            let default_sort_column = "id";
            let default_sort_order = "ASC";

            // Parse filter, range, and sort parameters
            let filters: HashMap<String, String> = if let Some(filter) = params.filter {
                serde_json::from_str(&filter).unwrap_or_default()
            } else {
                HashMap::new()
            };

            let (offset, limit) = if let Some(range) = params.range {
                let range_vec: Vec<u64> = serde_json::from_str(&range).unwrap_or(vec![0, 24]);
                let start = range_vec.get(0).copied().unwrap_or(0);
                let end = range_vec.get(1).copied().unwrap_or(24);
                let limit = end - start + 1;
                (start, limit)
            } else {
                (0, 25)
            };

            let (sort_column, sort_order) = if let Some(sort) = params.sort {
                let sort_vec: Vec<String> = serde_json::from_str(&sort).unwrap_or(vec![
                    default_sort_column.to_string(),
                    default_sort_order.to_string(),
                ]);
                (
                    sort_vec
                        .get(0)
                        .cloned()
                        .unwrap_or(default_sort_column.to_string()),
                    sort_vec
                        .get(1)
                        .cloned()
                        .unwrap_or(default_sort_order.to_string()),
                )
            } else {
                (
                    default_sort_column.to_string(),
                    default_sort_order.to_string(),
                )
            };

            // Apply filters
            let mut condition = Condition::all();

            if let Some(q_value) = filters.get("q") {
                // Free-text search across specified columns
                let mut or_conditions = Condition::any();
                for (_col_name, col) in $searchable_columns {
                    or_conditions = or_conditions.add(
                        Expr::col(col).ilike(format!("%{}%", q_value)));
                }
                condition = condition.add(or_conditions);
            } else {
                for (key, mut value) in filters {
                    value = value.trim().to_string();

                    // Check if the value is a UUID, otherwise treat it as a string filter
                    if let Ok(uuid) = Uuid::parse_str(&value) {
                        condition = condition.add(Expr::col(Alias::new(&key)).eq(uuid));
                    } else {
                        condition = condition.add(Expr::col(Alias::new(&key)).ilike(format!("%{}%", value)));
                    }
                }
            }

            // Sorting and pagination
            let order_direction = if sort_order == "ASC" {
                Order::Asc
            } else {
                Order::Desc
            };

            let order_column = $order_column_logic
                .iter()
                .find(|&&(col_name, _)| col_name == sort_column)
                .map(|&(_, col)| col)
                .unwrap_or(<$db_columns>::Id);

            let objs: Vec<$db_model> = <$db_entity>::find()
                .filter(condition.clone())
                .order_by(order_column, order_direction)
                .offset(offset)
                .limit(limit)
                .all(&db)
                .await
                .unwrap();

            // Maps the database model to the response model
            let objs: Vec<$get_all_response_model> = objs
                .into_iter()
                .map(Into::into)
                .collect();

            // Get total count for content range header
            let total_count: u64 = <$db_entity>::find()
                .filter(condition.clone())
                .count(&db)
                .await
                .unwrap();
            let max_offset_limit = (offset + limit-1).min(total_count);
            let content_range = format!(
                "{} {}-{}/{}",
                $resource_name,
                offset,
                max_offset_limit,
                total_count
            );

            // Return Content-Range as a header
            let mut headers = HeaderMap::new();
            headers.insert("Content-Range", content_range.parse().unwrap());
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

            let response_obj: $get_one_response_model = obj.unwrap().into();

            (StatusCode::OK, Json(response_obj))

        }
    };
}
