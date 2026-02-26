use super::models::{Area, Plot};
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
            let avgs = fetch_all_sensor_averages(&db, &models)
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

/// Fetch all sensor temperature and moisture averages using the
/// `sensordata_daily` TimescaleDB continuous aggregate (~100x fewer rows
/// than raw sensordata, with <0.1% error for VWC due to daily pre-averaging).
async fn fetch_all_sensor_averages(
    db: &DatabaseConnection,
    profiles: &[crate::routes::private::sensors::profile::db::Model],
) -> Result<
    (
        HashMap<Uuid, HashMap<i32, f64>>,
        HashMap<Uuid, HashMap<i32, f64>>,
    ),
    sea_orm::DbErr,
> {
    if profiles.is_empty() {
        return Ok((HashMap::new(), HashMap::new()));
    }

    let ids_csv = profiles
        .iter()
        .map(|p| format!("'{}'", p.id))
        .collect::<Vec<_>>()
        .join(",");

    // Temperature: weighted average from daily continuous aggregate
    let temp_sql = format!(
        r"
        WITH depths AS (
          SELECT
            sensorprofile_id,
            unnest(array[depth_cm_sensor1, depth_cm_sensor2, depth_cm_sensor3]) AS depth_cm,
            sensor_id, date_from, date_to
          FROM sensorprofile_assignment
          WHERE sensorprofile_id IN ({ids_csv})
        )
        SELECT
          d.sensorprofile_id,
          d.depth_cm,
          SUM(sd.avg_temp * sd.sample_count) / SUM(sd.sample_count) AS avg_temp
        FROM depths d
        JOIN sensordata_daily sd
          ON sd.sensor_id = d.sensor_id
         AND sd.bucket >= d.date_from AND sd.bucket <= d.date_to
        GROUP BY d.sensorprofile_id, d.depth_cm
        "
    );

    // Moisture: VWC formula applied to daily-averaged mc and temp_1,
    // then weighted-averaged by sample_count.
    //
    // VWC formula (matching vwc.rs):
    //   vwc0 = a*mc² + b*mc + c
    //   tcor = mc + (24.0 - temp) * (1.911327 - 1.270247 * vwc0)
    //   vwc  = clamp(a*tcor² + b*tcor + c, 0, 1)
    let coeffs_values = profiles
        .iter()
        .map(|p| {
            let soil: soil_sensor_toolbox::SoilType = p
                .soil_type_vwc
                .clone()
                .unwrap_or(crate::routes::private::sensors::profile::db::SoilTypeEnum::Universal)
                .into();
            let (a, b, c) = match soil {
                soil_sensor_toolbox::SoilType::Sand => (-3.00e-09, 0.000_161_192, -0.109_956_5),
                soil_sensor_toolbox::SoilType::LoamySandA => {
                    (-1.90e-08, 0.000_265_610, -0.154_089_3)
                }
                soil_sensor_toolbox::SoilType::LoamySandB => {
                    (-2.30e-08, 0.000_282_473, -0.167_211_2)
                }
                soil_sensor_toolbox::SoilType::SandyLoamA => {
                    (-3.80e-08, 0.000_339_449, -0.214_921_8)
                }
                soil_sensor_toolbox::SoilType::SandyLoamB => {
                    (-9.00e-10, 0.000_261_847, -0.158_618_3)
                }
                soil_sensor_toolbox::SoilType::Loam => (-5.10e-08, 0.000_397_984, -0.291_046_4),
                soil_sensor_toolbox::SoilType::SiltLoam => (1.70e-08, 0.000_118_119, -0.101_168_5),
                soil_sensor_toolbox::SoilType::Peat => (1.23e-07, -0.000_144_644, 0.202_927_9),
                soil_sensor_toolbox::SoilType::Water => (0.00e+00, 0.000_306_700, -0.134_927_9),
                soil_sensor_toolbox::SoilType::Universal => {
                    (-1.34e-08, 0.000_249_622, -0.157_888_8)
                }
                soil_sensor_toolbox::SoilType::SandTMS1 => (0.00e+00, 0.000_260_000, -0.133_040_0),
                soil_sensor_toolbox::SoilType::LoamySandTMS1 => {
                    (0.00e+00, 0.000_330_000, -0.193_890_0)
                }
                soil_sensor_toolbox::SoilType::SiltLoamTMS1 => {
                    (0.00e+00, 0.000_380_000, -0.294_270_0)
                }
            };
            format!("('{}'::uuid, {}::double precision, {}::double precision, {}::double precision)", p.id, a, b, c)
        })
        .collect::<Vec<_>>()
        .join(",\n             ");

    let moisture_sql = format!(
        r"
        WITH soil_coeffs(sensorprofile_id, a, b, c) AS (
          VALUES {coeffs_values}
        ),
        assignments AS (
          SELECT sa.sensorprofile_id, sa.depth_cm_moisture AS depth_cm,
                 sa.sensor_id, sa.date_from, sa.date_to,
                 sc.a, sc.b, sc.c
          FROM sensorprofile_assignment sa
          JOIN soil_coeffs sc ON sc.sensorprofile_id = sa.sensorprofile_id
          WHERE sa.sensorprofile_id IN ({ids_csv})
            AND sa.depth_cm_moisture IS NOT NULL
        )
        SELECT a.sensorprofile_id, a.depth_cm,
          SUM(
            GREATEST(0.0::double precision, LEAST(1.0::double precision,
              a.a * vwc.tcor * vwc.tcor + a.b * vwc.tcor + a.c
            )) * sd.sample_count
          ) / SUM(sd.sample_count) AS avg_vwc
        FROM assignments a
        JOIN sensordata_daily sd
          ON sd.sensor_id = a.sensor_id
         AND sd.bucket >= a.date_from AND sd.bucket <= a.date_to
        CROSS JOIN LATERAL (
          SELECT sd.avg_moisture_count + (24.0 - sd.avg_temp_1)
                       * (1.911327 - 1.270247 * (a.a * sd.avg_moisture_count * sd.avg_moisture_count + a.b * sd.avg_moisture_count + a.c))
                 AS tcor
        ) vwc
        GROUP BY a.sensorprofile_id, a.depth_cm
        "
    );

    let (temp_rows, moisture_rows) = tokio::join!(
        async {
            let stmt =
                Statement::from_sql_and_values(db.get_database_backend(), &temp_sql, vec![]);
            db.query_all(stmt).await
        },
        async {
            let stmt =
                Statement::from_sql_and_values(db.get_database_backend(), &moisture_sql, vec![]);
            db.query_all(stmt).await
        },
    );
    let temp_rows = temp_rows?;
    let moisture_rows = moisture_rows?;

    let mut temp_map: HashMap<Uuid, HashMap<i32, f64>> = HashMap::new();
    for row in temp_rows {
        let pid: Uuid = row.try_get("", "sensorprofile_id")?;
        let depth: i32 = row.try_get("", "depth_cm")?;
        let avg: f64 = row.try_get("", "avg_temp")?;
        temp_map.entry(pid).or_default().insert(depth, avg);
    }

    let mut moisture_map: HashMap<Uuid, HashMap<i32, f64>> = HashMap::new();
    for row in moisture_rows {
        let pid: Uuid = row.try_get("", "sensorprofile_id")?;
        let depth: i32 = row.try_get("", "depth_cm")?;
        let avg: f64 = row.try_get("", "avg_vwc")?;
        moisture_map.entry(pid).or_default().insert(depth, avg);
    }

    Ok((temp_map, moisture_map))
}

/// Helper: fetch precomputed convex hulls from the area table
async fn fetch_all_hulls(
    db: &DatabaseConnection,
    area_ids: &[Uuid],
) -> Result<HashMap<Uuid, serde_json::Value>, sea_orm::DbErr> {
    let ids = area_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("','");

    let sql = format!(
        r"SELECT id AS area_id, ST_AsGeoJSON(hull_geom)::json AS hull
          FROM area
          WHERE id IN ('{ids}') AND hull_geom IS NOT NULL"
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
