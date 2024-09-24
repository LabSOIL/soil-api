use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct SoilType {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description: String,
    pub image: Option<String>,
}

#[derive(ToSchema, Serialize)]
pub struct SoilTypeBasic {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description: String,
}
