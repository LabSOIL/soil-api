use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilType {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description: String,
    pub image: Option<String>,
}

#[derive(ToSchema, Serialize, Deserialize)]
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

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeCreate {
    pub name: String,
    pub description: String,
    pub image: Option<String>,
}

impl From<SoilTypeCreate> for crate::soil::types::db::ActiveModel {
    fn from(soil_type: SoilTypeCreate) -> Self {
        crate::soil::types::db::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
            last_updated: sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc()),
            name: sea_orm::ActiveValue::Set(soil_type.name),
            description: sea_orm::ActiveValue::Set(soil_type.description),
            image: sea_orm::ActiveValue::Set(soil_type.image),
            iterator: sea_orm::ActiveValue::NotSet,
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}
