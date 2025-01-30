use crate::common::crud::models::FilterOptions;
use crate::samples::db::Entity as PlotSampleDB;
use crate::samples::models::PlotSample;
use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    http::header::HeaderMap,
    routing, Json, Router,
};
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::{
    query::*,
    sea_query::{Alias, Expr},
    DatabaseConnection,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .with_state(db)
}

#[utoipa::path(get, path = "/api/plot_samples", responses((status = OK, body = PlotSample)))]
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    // Default values for range and sorting
    let default_sort_column = "id";
    let default_sort_order = "ASC";

    // 1. Parse the filter, range, and sort parameters
    let filters: HashMap<String, String> = if let Some(filter) = params.filter {
        serde_json::from_str(&filter).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let (offset, limit) = if let Some(range) = params.range {
        let range_vec: Vec<u64> = serde_json::from_str(&range).unwrap_or(vec![0, 24]); // Default to [0, 24]
        let start = range_vec.get(0).copied().unwrap_or(0);
        let end = range_vec.get(1).copied().unwrap_or(24);
        let limit = end - start + 1;
        (start, limit) // Offset is `start`, limit is the number of documents to fetch
    } else {
        (0, 25) // Default to 25 documents starting at 0
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
    for (key, mut value) in filters {
        value = value.trim().to_string();

        // Check if the value is a valid UUID
        if let Ok(uuid) = Uuid::parse_str(&value) {
            // If the value is a valid UUID, filter it as a UUID
            condition = condition.add(Expr::col(Alias::new(&key)).eq(uuid));
        } else {
            // Otherwise, treat it as a regular string filter
            condition = condition.add(Expr::col(Alias::new(&key)).eq(value));
        }
    }

    // Query with filtering, sorting, and pagination
    let order_direction = if sort_order == "ASC" {
        Order::Asc
    } else {
        Order::Desc
    };
    let order_column: <PlotSampleDB as sea_orm::EntityTrait>::Column = match sort_column.as_str() {
        "id" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Id,
        "name" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Name,
        "last_updated" => <PlotSampleDB as sea_orm::EntityTrait>::Column::LastUpdated,
        "upper_depth_cm" => <PlotSampleDB as sea_orm::EntityTrait>::Column::UpperDepthCm,
        "lower_depth_cm" => <PlotSampleDB as sea_orm::EntityTrait>::Column::LowerDepthCm,
        "plot_id" => <PlotSampleDB as sea_orm::EntityTrait>::Column::PlotId,
        "sample_weight" => <PlotSampleDB as sea_orm::EntityTrait>::Column::SampleWeight,
        "subsample_weight" => <PlotSampleDB as sea_orm::EntityTrait>::Column::SubsampleWeight,
        "ph" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Ph,
        "rh" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Rh,
        "loi" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Loi,
        "mfc" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Mfc,
        "c" => <PlotSampleDB as sea_orm::EntityTrait>::Column::C,
        "n" => <PlotSampleDB as sea_orm::EntityTrait>::Column::N,
        "cn" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Cn,
        "clay_percent" => <PlotSampleDB as sea_orm::EntityTrait>::Column::ClayPercent,
        "silt_percent" => <PlotSampleDB as sea_orm::EntityTrait>::Column::SiltPercent,
        "sand_percent" => <PlotSampleDB as sea_orm::EntityTrait>::Column::SandPercent,
        "fe_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::FeUgPerG,
        "na_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::NaUgPerG,
        "al_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::AlUgPerG,
        "k_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::KUgPerG,
        "ca_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::CaUgPerG,
        "mg_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::MgUgPerG,
        "mn_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::MnUgPerG,
        "s_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::SUgPerG,
        "cl_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::ClUgPerG,
        "p_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::PUgPerG,
        "si_ug_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::SiUgPerG,
        "subsample_replica_weight" => {
            <PlotSampleDB as sea_orm::EntityTrait>::Column::SubsampleReplicaWeight
        }
        "fungi_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::FungiPerG,
        "bacteria_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::BacteriaPerG,
        "archea_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::ArcheaPerG,
        "methanogens_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::MethanogensPerG,
        "methanotrophs_per_g" => <PlotSampleDB as sea_orm::EntityTrait>::Column::MethanotrophsPerG,
        "replicate" => <PlotSampleDB as sea_orm::EntityTrait>::Column::Replicate,
        _ => <PlotSampleDB as sea_orm::EntityTrait>::Column::Id,
    };

    let plot_samples: Vec<super::models::PlotSample> =
        PlotSample::get_all(&db, condition, order_column, order_direction, offset, limit).await;

    let total_samples: u64 = PlotSampleDB::find().count(&db).await.unwrap();
    let max_offset_limit = (offset + limit).min(total_samples);
    let content_range = format!(
        "plotsamples {}-{}/{}",
        offset,
        max_offset_limit - 1,
        total_samples
    );

    // Return the Content-Range as a header
    let mut headers = HeaderMap::new();
    headers.insert("Content-Range", content_range.parse().unwrap());
    (headers, Json(json!(plot_samples)))
}
