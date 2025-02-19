use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, DbBackend, Statement};
use serde_json::Value;

pub async fn get_convex_hull(db: &DatabaseConnection, area_id: Uuid) -> Option<Value> {
    let raw_sql = r#"
    SELECT area.id,
           ST_AsGeoJSON(ST_Transform(ST_Buffer(ST_ConvexHull(ST_Collect(area.geom)), $1), $2)) AS convex_hull
    FROM (
        SELECT area.id AS id,
               ST_Transform(plot.geom, $3) AS geom
        FROM area
        JOIN plot ON area.id = plot.area_id
        UNION ALL
        SELECT area.id AS id,
               ST_Transform(soilprofile.geom, $4) AS geom
        FROM area
        JOIN soilprofile ON area.id = soilprofile.area_id
        UNION ALL
        SELECT area.id AS id,
               ST_Transform(sensorprofile.geom, $5) AS geom
        FROM area
        JOIN sensorprofile ON area.id = sensorprofile.area_id
    ) AS area
    WHERE area.id = $6
    GROUP BY area.id
    "#;

    // Try to execute the query
    if let Ok(result) = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            raw_sql,
            vec![
                10.into(),      // ST_Buffer value
                4326.into(),    // ST_Transform value
                2056.into(),    // ST_Transform value for plot.geom
                2056.into(),    // ST_Transform value for soilprofile.geom
                2056.into(),    // ST_Transform value for sensorprofile.geom
                area_id.into(), // The ID of the area
            ],
        ))
        .await
    {
        if let Some(row) = result {
            if let Ok(convex_hull) = row.try_get::<String>("", "convex_hull") {
                if let Ok(parsed_geojson) = serde_json::from_str(&convex_hull) {
                    return parsed_geojson; // Return the parsed GeoJSON if valid
                }
            }
        }
    }

    // Return none if the query fails
    None
}
