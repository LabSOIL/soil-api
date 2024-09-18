use sea_orm::FromQueryResult;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Deserialize, Default)]
pub struct FilterOptions {
    pub filter: Option<String>, // JSON-encoded filter
    pub range: Option<String>,  // range in the format "[0,24]"
    pub sort: Option<String>,   // sort in the format '["id", "ASC"]'
}

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct GenericNameAndID {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct ClosestFeature {
    pub id: Uuid,
    pub name: String,
    pub distance: f64,
    pub elevation_difference: f64,
    pub feature_type: String,
}
