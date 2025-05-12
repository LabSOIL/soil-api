use super::models::{Area, Plot};
use crate::routes::private::areas::db as AreaDB;
use crate::routes::private::areas::services::get_convex_hull;
use crate::routes::private::plots::db as PlotDB;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::DatabaseConnection;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
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
    summary = "Get all areas (public)",
    description = "Returns a list of all available areas with associated properties to display in the public UI. If no public areas exist, an empty list is returned.",
    operation_id = "get_all_areas_public",
)]
pub async fn get_all_areas(State(db): State<DatabaseConnection>) -> impl IntoResponse {
    match AreaDB::Entity::find()
        .filter(AreaDB::Column::IsPublic.eq(true))
        .all(&db)
        .await
    {
        Ok(objs) => {
            // Add geometry for each area
            let mut areas: Vec<Area> = Vec::new();
            for obj in objs {
                let plot = obj
                    .find_related(PlotDB::Entity)
                    .all(&db)
                    .await
                    .unwrap_or_default();
                let mut area: Area = obj.into();
                area.plots = plot.into_iter().map(Plot::from).collect();
                area.geom = get_convex_hull(&db, area.id).await;
                areas.push(area);
            }

            Ok((StatusCode::OK, Json(areas)))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal server error".to_string()),
        )),
    }
}
