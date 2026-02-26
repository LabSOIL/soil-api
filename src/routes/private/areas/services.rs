use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, DbBackend, Statement};
use serde_json::{Value, json};

pub async fn get_convex_hull(db: &DatabaseConnection, area_id: Uuid) -> Option<Value> {
    let raw_sql = r"SELECT ST_AsGeoJSON(hull_geom) AS convex_hull
                     FROM area WHERE id = $1 AND hull_geom IS NOT NULL";

    if let Ok(result) = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            raw_sql,
            vec![area_id.into()],
        ))
        .await
    {
        let Some(row) = result else {
            return Some(json!({"type": "FeatureCollection", "features": []}));
        };
        let convex_hull = row.try_get::<String>("", "convex_hull").unwrap();
        if let Ok(parsed_geojson) = serde_json::from_str(&convex_hull) {
            return Some(parsed_geojson);
        }
    }
    None
}
