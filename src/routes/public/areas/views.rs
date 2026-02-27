use super::models::{Area, Plot};
use crate::routes::private::areas::db as AreaDB;
use crate::routes::private::plots::db as PlotDB;
use crate::routes::private::samples::models::PlotSample;
use crate::routes::private::{
    samples::db as SampleDB, sensors::profile::db as SensorProfileDB,
    soil::classification::db as SoilClassDB,
};
use crate::routes::public::sensors::models::SensorProfileSimple;
use crate::routes::public::website_access::{resolve_website_access, validate_slug};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::Statement;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::sea_query::{ArrayType, Value};
use serde::Deserialize;
use std::collections::HashMap;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

pub fn router(db: &DatabaseConnection) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_all_areas))
        .with_state(db.clone())
}

#[derive(Deserialize)]
pub struct AreaQueryParams {
    pub website: Option<String>,
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
pub async fn get_all_areas(
    State(db): State<DatabaseConnection>,
    Query(params): Query<AreaQueryParams>,
) -> impl IntoResponse {
    // 1) Resolve website access from slug
    let Some(website_slug) = params.website else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Missing required query parameter: 'website'".to_string()),
        ));
    };

    if !validate_slug(&website_slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid website slug".to_string()),
        ));
    }

    let website_access = match resolve_website_access(&db, &website_slug).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json("Website not found".to_string()),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal server error".to_string()),
            ));
        }
    };

    // Load areas assigned to this website
    if website_access.area_ids.is_empty() {
        return Ok((StatusCode::OK, Json(vec![])));
    }

    let Ok(areas) = AreaDB::Entity::find()
        .filter(AreaDB::Column::Id.is_in(website_access.area_ids.iter().copied()))
        .all(&db)
        .await
    else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal server error".to_string()),
        ));
    };
    let area_ids: Vec<Uuid> = areas.iter().map(|a| a.id).collect();

    // Run three independent query chains in parallel
    let (sensor_chain, hull_result, data_chain) = tokio::join!(
        // Chain 1: sensor profiles → sensor averages
        async {
            let models = SensorProfileDB::Entity::find()
                .filter(SensorProfileDB::Column::AreaId.is_in(area_ids.clone()))
                .all(&db)
                .await
                .unwrap_or_default();
            let profile_ids: Vec<Uuid> = models.iter().map(|m| m.id).collect();
            let avgs = fetch_all_sensor_averages(&db, &profile_ids)
                .await
                .unwrap_or_default();
            (models, avgs)
        },
        // Chain 2: convex hulls
        async { fetch_all_hulls(&db, &area_ids).await },
        // Chain 3: plots → samples → soil
        async {
            let plots = PlotDB::Entity::find()
                .filter(PlotDB::Column::AreaId.is_in(area_ids.clone()))
                .all(&db)
                .await
                .unwrap_or_default();
            let mut plots_by_area: HashMap<Uuid, Vec<PlotDB::Model>> = HashMap::new();
            for plot in &plots {
                plots_by_area
                    .entry(plot.area_id)
                    .or_default()
                    .push(plot.clone());
            }

            let plot_ids: Vec<Uuid> = plots.iter().map(|p| p.id).collect();
            let samples = SampleDB::Entity::find()
                .filter(SampleDB::Column::PlotId.is_in(plot_ids))
                .all(&db)
                .await
                .unwrap_or_default();

            let soil_ids: Vec<Uuid> = samples
                .iter()
                .filter_map(|s| s.soil_classification_id)
                .collect();
            let soil_classes = SoilClassDB::Entity::find()
                .filter(SoilClassDB::Column::Id.is_in(soil_ids))
                .all(&db)
                .await
                .unwrap_or_default();
            let soil_map: HashMap<Uuid, SoilClassDB::Model> =
                soil_classes.into_iter().map(|c| (c.id, c)).collect();

            let mut samples_by_plot: HashMap<Uuid, Vec<PlotSample>> = HashMap::new();
            for s in samples {
                let Some(soil_id) = s.soil_classification_id else {
                    continue;
                };
                let Some(soil) = soil_map.get(&soil_id) else {
                    continue;
                };
                let enriched = PlotSample::from((s.clone(), Some(soil.clone())));
                samples_by_plot
                    .entry(enriched.plot_id)
                    .or_default()
                    .push(enriched);
            }

            (plots_by_area, samples_by_plot)
        },
    );

    let (sensor_models, (avg_temp_map, avg_moisture_map)): (
        Vec<SensorProfileDB::Model>,
        (
            HashMap<Uuid, HashMap<i32, f64>>,
            HashMap<Uuid, HashMap<i32, f64>>,
        ),
    ) = sensor_chain;
    let hull_map = hull_result.unwrap_or_default();
    let (mut plots_by_area, mut samples_by_plot): (
        HashMap<Uuid, Vec<PlotDB::Model>>,
        HashMap<Uuid, Vec<PlotSample>>,
    ) = data_chain;

    // Assemble result
    let mut area_models = Vec::with_capacity(areas.len());
    for area in areas {
        let mut am: Area = area.clone().into();

        // plots → attach samples, aggregate, convert
        am.plots = plots_by_area
            .remove(&area.id)
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                let mut private_plot = crate::routes::private::plots::models::Plot::from((
                    p.clone(),
                    area.clone(),
                    samples_by_plot.remove(&p.id).unwrap_or_default(),
                    vec![],
                    vec![],
                ));
                private_plot.aggregated_samples = private_plot.aggregate_samples();
                Plot::from(private_plot)
            })
            .collect();

        // sensors → convert via SensorProfile then into Simple, inject averages
        am.sensors = sensor_models
            .iter()
            .filter(|m| m.area_id == area.id)
            .map(|m| {
                let full: crate::routes::private::sensors::profile::models::SensorProfile =
                    m.clone().into();
                let mut simple: SensorProfileSimple = full.into();
                simple.average_temperature = avg_temp_map.get(&m.id).cloned().unwrap_or_default();
                simple.average_moisture = avg_moisture_map.get(&m.id).cloned().unwrap_or_default();
                simple
            })
            .collect();

        // Apply website exclusions
        am.plots
            .retain(|p| !website_access.excluded_plot_ids.contains(&p.id));
        am.sensors
            .retain(|s| !website_access.excluded_sensor_ids.contains(&s.id));

        // convex hull
        am.geom = hull_map.get(&area.id).cloned();

        area_models.push(am);
    }
    Ok((StatusCode::OK, Json(area_models)))
}

/// Fetch precomputed sensor temperature and moisture averages from
/// the `sensorprofile_averages` table (maintained by DB triggers).
async fn fetch_all_sensor_averages(
    db: &DatabaseConnection,
    profile_ids: &[Uuid],
) -> Result<
    (
        HashMap<Uuid, HashMap<i32, f64>>,
        HashMap<Uuid, HashMap<i32, f64>>,
    ),
    sea_orm::DbErr,
> {
    if profile_ids.is_empty() {
        return Ok((HashMap::new(), HashMap::new()));
    }

    let uuid_values: Vec<Value> = profile_ids
        .iter()
        .map(|&id| Value::Uuid(Some(Box::new(id))))
        .collect();
    let array_param = Value::Array(ArrayType::Uuid, Some(Box::new(uuid_values)));

    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT sensorprofile_id, depth_cm, avg_temp, avg_vwc
         FROM sensorprofile_averages
         WHERE sensorprofile_id = ANY($1)",
        vec![array_param],
    );

    let rows = db.query_all(stmt).await?;

    let mut temp_map: HashMap<Uuid, HashMap<i32, f64>> = HashMap::new();
    let mut moisture_map: HashMap<Uuid, HashMap<i32, f64>> = HashMap::new();
    for row in rows {
        let pid: Uuid = row.try_get("", "sensorprofile_id")?;
        let depth: i32 = row.try_get("", "depth_cm")?;
        if let Ok(avg_temp) = row.try_get::<f64>("", "avg_temp") {
            temp_map.entry(pid).or_default().insert(depth, avg_temp);
        }
        if let Ok(avg_vwc) = row.try_get::<f64>("", "avg_vwc") {
            moisture_map.entry(pid).or_default().insert(depth, avg_vwc);
        }
    }

    Ok((temp_map, moisture_map))
}

/// Helper: fetch precomputed convex hulls from the area table
async fn fetch_all_hulls(
    db: &DatabaseConnection,
    area_ids: &[Uuid],
) -> Result<HashMap<Uuid, serde_json::Value>, sea_orm::DbErr> {
    let uuid_values: Vec<Value> = area_ids
        .iter()
        .map(|&id| Value::Uuid(Some(Box::new(id))))
        .collect();
    let array_param = Value::Array(ArrayType::Uuid, Some(Box::new(uuid_values)));

    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT id AS area_id, ST_AsGeoJSON(hull_geom)::json AS hull
         FROM area WHERE id = ANY($1) AND hull_geom IS NOT NULL",
        vec![array_param],
    );

    let rows = db.query_all(stmt).await?;
    let mut map = HashMap::new();
    for row in rows {
        let aid: Uuid = row.try_get("", "area_id")?;
        let hull: serde_json::Value = row.try_get("", "hull")?;
        map.insert(aid, hull);
    }
    Ok(map)
}
