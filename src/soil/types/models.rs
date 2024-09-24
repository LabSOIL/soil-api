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

impl From<crate::soil::types::db::Model> for SoilTypeBasic {
    fn from(soil_type: crate::soil::types::db::Model) -> Self {
        SoilTypeBasic {
            id: soil_type.id,
            last_updated: soil_type.last_updated,
            name: Some(soil_type.name),
            description: soil_type.description,
        }
    }
}

impl From<crate::soil::types::db::Model> for SoilType {
    fn from(soil_type: crate::soil::types::db::Model) -> Self {
        SoilType {
            id: soil_type.id,
            last_updated: soil_type.last_updated,
            name: Some(soil_type.name),
            description: soil_type.description,
            image: soil_type.image,
        }
    }
}
