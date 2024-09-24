use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct SoilProfile {
    pub id: Uuid,
    pub name: String,
    pub profile_iterator: i32,
    pub gradient: String,
    pub description_horizon: Option<Value>,
    pub last_updated: chrono::NaiveDateTime,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    pub created_on: Option<NaiveDateTime>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    pub soil_diagram: Option<String>,
    pub photo: Option<String>,
    pub parent_material: Option<f64>,
}

#[derive(ToSchema, Serialize)]
pub struct SoilProfileBasic {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description_horizon: Option<Value>,
}

impl From<crate::soil::profiles::db::Model> for SoilProfileBasic {
    fn from(soil_profile: crate::soil::profiles::db::Model) -> Self {
        SoilProfileBasic {
            id: soil_profile.id,
            last_updated: soil_profile.last_updated,
            name: Some(soil_profile.name),
            description_horizon: soil_profile.description_horizon,
        }
    }
}

impl From<crate::soil::profiles::db::Model> for SoilProfile {
    fn from(soil_profile: crate::soil::profiles::db::Model) -> Self {
        SoilProfile {
            id: soil_profile.id,
            name: soil_profile.name,
            profile_iterator: soil_profile.profile_iterator,
            gradient: soil_profile.gradient,
            description_horizon: soil_profile.description_horizon,
            last_updated: soil_profile.last_updated,
            weather: soil_profile.weather,
            topography: soil_profile.topography,
            vegetation_type: soil_profile.vegetation_type,
            aspect: soil_profile.aspect,
            lythology_surficial_deposit: soil_profile.lythology_surficial_deposit,
            created_on: soil_profile.created_on,
            soil_type_id: soil_profile.soil_type_id,
            area_id: soil_profile.area_id,
            soil_diagram: soil_profile.soil_diagram,
            photo: soil_profile.photo,
            parent_material: soil_profile.parent_material,
        }
    }
}
