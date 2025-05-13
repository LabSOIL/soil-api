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
        Ok(areas) => {
            // Add geometry for each area
            let mut area_models: Vec<Area> = Vec::new();
            for area in areas {
                let plots = area
                    .find_related(PlotDB::Entity)
                    .all(&db)
                    .await
                    .unwrap_or_default();
                let mut plot_models: Vec<Plot> = Vec::new();
                for plot in plots {
                    let samples = crate::routes::private::samples::db::Entity::find()
                        .filter(crate::routes::private::samples::db::Column::PlotId.eq(plot.id))
                        .all(&db)
                        .await
                        .unwrap();
                    let mut sample_objs: Vec<crate::routes::private::samples::models::PlotSample> =
                        Vec::new();
                    // Get the soil classification for each sample
                    for sample in samples {
                        let soil_classification =
                            crate::routes::private::soil::classification::db::Entity::find()
                                .filter(
                                    crate::routes::private::soil::classification::db::Column::Id
                                        .eq(sample.soil_classification_id),
                                )
                                .one(&db)
                                .await
                                .unwrap();

                        let updated_sample =
                            crate::routes::private::samples::models::PlotSample::from((
                                sample.clone(),
                                soil_classification,
                            ));
                        sample_objs.push(updated_sample);
                    }
                    // Use the private route Plot to get all the samples and aggregated samples
                    let mut plot: crate::routes::private::plots::models::Plot =
                        (plot, area.clone(), sample_objs, vec![], vec![]).into();
                    plot.aggregated_samples = plot.aggregate_samples();

                    plot_models.push(plot.into());
                }

                let mut area: Area = area.into();
                area.plots = plot_models;
                area.geom = get_convex_hull(&db, area.id).await;
                area_models.push(area);
            }
            println!("Areas: {:?}", area_models.len());

            Ok((StatusCode::OK, Json(area_models)))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal server error".to_string()),
        )),
    }
}
