use crate::routes::private::areas::db;
use crate::routes::private::areas::services::get_convex_hull;
use crate::routes::public::areas::models::Area;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::DatabaseConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router(db: &DatabaseConnection) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_all_areas))
        .with_state(db.clone())
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "List of all areas. Returns an empty list if no areas are found.", body = Vec<Area>),
        (status = 500, description = "Internal server error")
    ),
    summary = "Get all areas",
    description = "Returns a list of all available areas. If no public areas exist, an empty list is returned."
)]
pub async fn get_all_areas(State(db): State<DatabaseConnection>) -> impl IntoResponse {
    match db::Entity::find()
        .filter(db::Column::IsPublic.eq(true))
        .all(&db)
        .await
    {
        Ok(objs) => {
            let mut areas: Vec<Area> = objs.into_iter().map(From::from).collect();

            // Add geometry for each area
            for area in &mut areas {
                area.geom = get_convex_hull(&db, area.id).await;
            }

            Ok((StatusCode::OK, Json(areas)))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal server error".to_string()),
        )),
    }
}
