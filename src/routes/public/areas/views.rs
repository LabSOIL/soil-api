use super::models::{Area, Plot};
use crate::config::Config;
use crate::routes::private::areas::db as AreaDB;
use crate::routes::private::plots::db as PlotDB;
use crate::routes::private::samples::models::PlotSample;
use crate::routes::private::{
    samples::db as SampleDB, sensors::profile::db as SensorProfileDB,
    soil::classification::db as SoilClassDB,
};
use crate::routes::public::sensors::models::SensorProfileSimple;
use crate::routes::public::website_access::resolve_website_access;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_response_cache::CacheLayer;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::Statement;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
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
    pub website: String,
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
    let website_access = match resolve_website_access(&db, &params.website).await {
        Ok(Some(a)) => a,
        Ok(None) => return Ok((StatusCode::OK, Json(vec![]))), // unknown slug = empty
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

    // 2) Bulk‐fetch plots
    let plots = PlotDB::Entity::find()
        .filter(PlotDB::Column::AreaId.is_in(area_ids.clone()))
        .all(&db)
        .await
        .unwrap_or_default();
    let mut plots_by_area: HashMap<Uuid, Vec<_>> = HashMap::new();
    for plot in plots {
        plots_by_area.entry(plot.area_id).or_default().push(plot);
    }

    // 3) Bulk‐fetch samples
    let plot_ids: Vec<Uuid> = plots_by_area.values().flatten().map(|p| p.id).collect();
    let samples = SampleDB::Entity::find()
        .filter(SampleDB::Column::PlotId.is_in(plot_ids.clone()))
        .all(&db)
        .await
        .unwrap_or_default();

    // 4) Bulk‐fetch soils and index by ID
    let soil_ids: Vec<Uuid> = samples
        .iter()
        .filter_map(|s| s.soil_classification_id)
        .collect();
    let soil_classes = SoilClassDB::Entity::find()
        .filter(SoilClassDB::Column::Id.is_in(soil_ids.clone()))
        .all(&db)
        .await
        .unwrap_or_default();
    let soil_map: HashMap<Uuid, _> = soil_classes.into_iter().map(|c| (c.id, c)).collect();

    // group samples by plot, enriching with soil
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

    // 5) Bulk‐fetch sensor profiles
    let sensor_models = SensorProfileDB::Entity::find()
        .filter(SensorProfileDB::Column::AreaId.is_in(area_ids.clone()))
        .all(&db)
        .await
        .unwrap_or_default();

    // Compute all average temperatures in one SQL pass
    let avg_temp_map = fetch_all_average_temps(&db, &sensor_models)
        .await
        .unwrap_or_default();

    // Compute all average moisture counts in one SQL pass
    let avg_moisture_map = fetch_all_average_moisture(&db, &sensor_models)
        .await
        .unwrap_or_default();

    // Compute all convex hulls in one SQL pass
    let hull_map = fetch_all_hulls(&db, &area_ids).await.unwrap_or_default();

    // 6) Assemble result
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

/// Helper: fetch all average moisture counts in one SQL query
async fn fetch_all_average_moisture(
    db: &DatabaseConnection,
    profiles: &[crate::routes::private::sensors::profile::db::Model],
) -> Result<HashMap<Uuid, HashMap<i32, f64>>, sea_orm::DbErr> {
    use soil_sensor_toolbox::mc_calc_vwc;
    use std::collections::BTreeMap;

    // Build quoted ID list: 'id1','id2',...
    let ids_csv = profiles
        .iter()
        .map(|p| format!("'{}'", p.id))
        .collect::<Vec<_>>()
        .join(",");

    // Create a map of profile_id -> soil_type for VWC calculations
    // Default to Universal for profiles without a soil type (chamber/redox)
    let profile_soil_types: HashMap<Uuid, soil_sensor_toolbox::SoilType> = profiles
        .iter()
        .map(|p| {
            let soil_type = p
                .soil_type_vwc
                .clone()
                .unwrap_or(crate::routes::private::sensors::profile::db::SoilTypeEnum::Universal)
                .into();
            (p.id, soil_type)
        })
        .collect();

    // SQL: fetch individual readings (not averaged) with temperature
    let sql = format!(
        r"
        WITH depths AS (
          SELECT
            sensorprofile_id,
            depth_cm_moisture   AS depth_cm,
            sensor_id,
            date_from,
            date_to
          FROM sensorprofile_assignment
          WHERE sensorprofile_id IN ({ids_csv})
        )
        SELECT
          d.sensorprofile_id,
          d.depth_cm,
          sd.soil_moisture_count::double precision AS moisture_count,
          sd.temperature_1 AS temperature
        FROM depths AS d
        JOIN sensordata AS sd
          ON sd.sensor_id = d.sensor_id
         AND sd.time_utc >= d.date_from
         AND sd.time_utc <= d.date_to
        ",
    );

    //Run the query, logging any error
    let stmt = Statement::from_sql_and_values(db.get_database_backend(), &sql, vec![]);
    let rows = match db.query_all(stmt).await {
        Ok(r) => r,
        Err(err) => {
            return Err(err);
        }
    };

    // Process rows: apply VWC conversion then group and average
    let mut grouped_vwc: HashMap<Uuid, BTreeMap<i32, Vec<f64>>> = HashMap::new();

    for row in rows {
        let pid: Uuid = row.try_get("", "sensorprofile_id")?;
        let depth: i32 = row.try_get("", "depth_cm")?;
        let moisture_count: f64 = row.try_get("", "moisture_count")?;
        let temperature: f64 = row.try_get("", "temperature")?;

        // Get soil type for this profile
        if let Some(&soil_type) = profile_soil_types.get(&pid) {
            // Calculate VWC using mc_calc_vwc
            let vwc = mc_calc_vwc(moisture_count, temperature, soil_type);

            // Group by profile_id and depth
            grouped_vwc
                .entry(pid)
                .or_default()
                .entry(depth)
                .or_default()
                .push(vwc);
        }
    }

    // Calculate averages for each profile/depth combination
    let mut map: HashMap<Uuid, HashMap<i32, f64>> = HashMap::new();
    for (pid, depths) in grouped_vwc {
        let mut depth_averages = HashMap::new();
        for (depth, vwc_values) in depths {
            if !vwc_values.is_empty() {
                #[allow(clippy::cast_precision_loss)]
                let average = vwc_values.iter().sum::<f64>() / vwc_values.len() as f64;
                depth_averages.insert(depth, average);
            }
        }
        map.insert(pid, depth_averages);
    }

    // Final debug: always printed
    Ok(map)
}

/// Helper: fetch all averages in one SQL query
async fn fetch_all_average_temps(
    db: &DatabaseConnection,
    profiles: &[crate::routes::private::sensors::profile::db::Model],
) -> Result<HashMap<Uuid, HashMap<i32, f64>>, sea_orm::DbErr> {
    let ids = profiles
        .iter()
        .map(|p| p.id.to_string())
        .collect::<Vec<_>>()
        .join("','");
    let sql = format!(
        r"
        WITH depths AS (
          SELECT
            sensorprofile_id,
            unnest(array[depth_cm_sensor1, depth_cm_sensor2, depth_cm_sensor3]) AS depth_cm,
            sensor_id, date_from, date_to
          FROM sensorprofile_assignment
          WHERE sensorprofile_id IN ('{ids}')
        ),
        buckets AS (
          SELECT
            d.sensorprofile_id,
            d.depth_cm,
            sd.temperature_average
          FROM depths d
          JOIN sensordata sd
            ON sd.sensor_id = d.sensor_id
           AND sd.time_utc BETWEEN d.date_from AND d.date_to
        )
        SELECT
          sensorprofile_id,
          depth_cm,
          AVG(temperature_average) AS avg_temp
        FROM buckets
        GROUP BY sensorprofile_id, depth_cm;
    "
    );
    let stmt = Statement::from_sql_and_values(db.get_database_backend(), &sql, vec![]);
    let rows = db.query_all(stmt).await?;
    let mut map: HashMap<Uuid, HashMap<i32, f64>> = HashMap::new();
    for row in rows {
        let pid: Uuid = row.try_get("", "sensorprofile_id")?;
        let depth: i32 = row.try_get("", "depth_cm")?;
        let avg: f64 = row.try_get("", "avg_temp")?;
        map.entry(pid).or_default().insert(depth, avg);
    }
    Ok(map)
}

/// Helper: fetch one convex hull per area in one SQL query
async fn fetch_all_hulls(
    db: &DatabaseConnection,
    area_ids: &[Uuid],
) -> Result<HashMap<Uuid, serde_json::Value>, sea_orm::DbErr> {
    use sea_orm::ConnectionTrait;

    let ids = area_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("','");

    // Union plots, soilprofiles and sensorprofiles (all in EPSG:2056),
    // then buffer, convex‐hull, transform to 4326.
    let sql = format!(
        r"
        SELECT
          id AS area_id,
          ST_AsGeoJSON(
            ST_Transform(
              ST_Buffer(
                ST_ConvexHull(
                  ST_Collect(geom)
                ),
                10
              ),
              4326
            )
          )::json AS hull
        FROM (
          SELECT area.id, ST_Transform(plot.geom, 2056) AS geom
          FROM area
          JOIN plot ON plot.area_id = area.id
          WHERE area.id IN ('{ids}')

          UNION ALL

          SELECT area.id, ST_Transform(soilprofile.geom, 2056) AS geom
          FROM area
          JOIN soilprofile ON soilprofile.area_id = area.id
          WHERE area.id IN ('{ids}')

          UNION ALL

          SELECT area.id, ST_Transform(sensorprofile.geom, 2056) AS geom
          FROM area
          JOIN sensorprofile ON sensorprofile.area_id = area.id
          WHERE area.id IN ('{ids}')
        ) AS all_geoms
        GROUP BY id;
        "
    );

    let stmt = Statement::from_sql_and_values(db.get_database_backend(), &sql, vec![]);
    let rows = db.query_all(stmt).await?;
    let mut map = HashMap::new();
    for row in rows {
        let aid: Uuid = row.try_get("", "area_id")?;
        let hull: serde_json::Value = row.try_get("", "hull")?;
        map.insert(aid, hull);
    }
    Ok(map)
}
