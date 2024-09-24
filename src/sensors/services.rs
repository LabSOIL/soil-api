// use crate::common::models::ClosestFeature;
// use crate::common::models::ClosestFeature;
// use crate::sensors::data::db::Entity as SensorDataDB;
// use crate::sensors::db::Entity as SensorDB;
use lttb::lttb;
// use sea_orm::DbBackend;
// use sea_orm::{query::*, DatabaseConnection};
// use sea_query::Alias;
// use sea_query::Expr;
// use sea_query::Order;
// use serde_json::json;
// use std::cmp::min;
// use uuid::Uuid;

// pub async fn fetch_closest_features(
//     sensor_id: Uuid,
//     area_id: Uuid,
//     db: &DatabaseConnection,
// ) -> Vec<ClosestFeature> {
//     // Fetch plots closest to the sensor
//     let plots = fetch_closest_plots(sensor_id, area_id, db).await;

//     // Fetch soil profiles closest to the sensor
//     let soil_profiles = fetch_closest_soil_profiles(sensor_id, area_id, db).await;

//     // Combine both lists of plots and soil profiles
//     let mut closest_features: Vec<ClosestFeature> = Vec::new();

//     closest_features.extend(plots);
//     closest_features.extend(soil_profiles);

//     // Sort by distance and return the closest 10
//     closest_features.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

//     closest_features.into_iter().take(10).collect()
// }
use lttb::DataPoint;

pub fn simplify_sensor_data_lttb(
    data: Vec<crate::sensors::data::db::Model>,
    target_points: usize,
) -> Vec<crate::sensors::data::db::Model> {
    let len = data.len();

    if len <= target_points {
        return data;
    }

    // Convert the sensor data timestamps and temperature_1 values to DataPoint structs
    let timestamps: Vec<DataPoint> = data
        .iter()
        .enumerate()
        .map(|(i, d)| DataPoint {
            x: i as f64, // or use d.time_utc.and_utc().timestamp() as f64
            y: d.temperature_1.unwrap_or(0.0),
        })
        .collect();

    // Downsample the data using the LTTB algorithm
    let downsampled_points = lttb(timestamps, target_points);

    // Now map the downsampled points back to your original data
    let mut downsampled_data = vec![];

    for point in downsampled_points {
        let original = &data[point.x as usize]; // Map x value back to original index
        let simplified = crate::sensors::data::db::Model {
            temperature_1: Some(point.y),
            ..original.clone()
        };
        downsampled_data.push(simplified);
    }

    downsampled_data
}
// async fn fetch_closest_plots(
//     sensor_id: Uuid,
//     area_id: Uuid,
//     db: &DatabaseConnection,
// ) -> Vec<ClosestFeature> {
//     let stmt_plots = sea_query::Query::select()
//         .columns([crate::plots::db::Column::Id, crate::plots::db::Column::Name])
//         .expr_as(
//             Expr::cust("ST_Distance(sensor.geom, plot.geom)"),
//             Alias::new("distance"),
//         )
//         .expr_as(
//             Expr::cust("ST_Z(sensor.geom) - ST_Z(plot.geom)"),
//             Alias::new("elevation_difference"),
//         )
//         .from(crate::plots::db::Entity)
//         .and_where(Expr::col(crate::plots::db::Column::AreaId).eq(area_id))
//         .and_where(Expr::col(crate::plots::db::Column::Id).eq(sensor_id))
//         .order_by_asc(Expr::cust("ST_Distance(sensor.geom, plot.geom)"))
//         .to_owned();

//     // Execute and fetch closest plots
//     SensorDB::find_by_statement(Statement::from_sql_and_values(
//         DbBackend::Postgres, // Use the appropriate database backend
//         stmt_plots.sql(),    // SQL query
//         stmt_plots.values(), // Bind values
//     ));

//     closest_plots
// }

// async fn fetch_closest_soil_profiles(
//     sensor_id: Uuid,
//     area_id: Uuid,
//     db: &DatabaseConnection,
// ) -> Vec<ClosestFeature> {
//     let stmt_soil_profiles = sea_query::Query::select()
//         .columns([
//             crate::soil::profiles::db::Column::Id,
//             crate::soil::profiles::db::Column::Name,
//         ])
//         .expr_as(
//             Expr::cust("ST_Distance(sensor.geom, soil_profile.geom)"),
//             "distance",
//         )
//         .expr_as(
//             Expr::cust("ST_Z(sensor.geom) - ST_Z(soil_profile.geom)"),
//             "elevation_difference",
//         )
//         .from(crate::soil::profiles::db::Entity)
//         .and_where(Expr::col(crate::soil::profiles::db::Column::AreaId).eq(area_id))
//         .and_where(Expr::col(crate::soil::profiles::db::Column::Id).eq(sensor_id))
//         .order_by(
//             Expr::cust("ST_Distance(sensor.geom, soil_profile.geom)"),
//             Order::Asc,
//         )
//         .to_owned();

//     // Execute and fetch closest soil profiles
//     let closest_soil_profiles: Vec<ClosestFeature> = SensorDB::find_by_statement(stmt_plots).all(db).await.unwrap()

//     closest_soil_profiles
// }
