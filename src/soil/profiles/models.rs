use super::db::Model;
use chrono::NaiveDateTime;
use sea_orm::{
    entity::prelude::*, query::*, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
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
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_srid: Option<i32>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
}

impl From<Model> for SoilProfile {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            profile_iterator: model.profile_iterator,
            gradient: model.gradient,
            description_horizon: model.description_horizon,
            last_updated: model.last_updated,
            weather: model.weather,
            topography: model.topography,
            vegetation_type: model.vegetation_type,
            aspect: model.aspect,
            lythology_surficial_deposit: model.lythology_surficial_deposit,
            created_on: model.created_on,
            soil_type_id: model.soil_type_id,
            area_id: model.area_id,
            soil_diagram: model.soil_diagram,
            photo: model.photo,
            parent_material: model.parent_material,
            latitude: None,
            longitude: None,
            coord_srid: None,
            coord_x: None,
            coord_y: None,
            coord_z: None,
        }
    }
}

impl SoilProfile {
    pub async fn from_area(
        area: &crate::areas::db::Model,
        db: &DatabaseConnection,
    ) -> Vec<SoilProfile> {
        super::db::Entity::find()
            .filter(super::db::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(soilprofile.geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(soilprofile.geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(soilprofile.geom)"), "coord_z")
            .column_as(
                Expr::cust("ST_X(st_transform(soilprofile.geom, 4326))"),
                "longitude",
            )
            .column_as(
                Expr::cust("ST_Y(st_transform(soilprofile.geom, 4326))"),
                "latitude",
            )
            .column_as(Expr::cust("st_srid(soilprofile.geom)"), "coord_srid")
            .into_model::<SoilProfile>()
            .all(db)
            .await
            .unwrap()
    }
    pub async fn from_db(
        soil_profile: crate::soil::profiles::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        crate::soil::profiles::db::Entity::find()
            .filter(crate::soil::profiles::db::Column::Id.eq(soil_profile.id))
            .into_model::<SoilProfile>()
            .one(db)
            .await
            .unwrap()
            .unwrap()
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
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

impl SoilProfileBasic {
    pub async fn from_db(
        soil_profile: crate::soil::profiles::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let soil_profile = crate::soil::profiles::db::Entity::find()
            .filter(crate::soil::profiles::db::Column::Id.eq(soil_profile.id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        SoilProfileBasic::from(soil_profile)
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilProfileCreate {
    pub name: String,
    pub profile_iterator: i32,
    pub gradient: String,
    pub description_horizon: Option<Value>,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    pub soil_diagram: Option<String>,
    pub photo: Option<String>,
    pub parent_material: Option<f64>,
}

impl From<SoilProfileCreate> for crate::soil::profiles::db::ActiveModel {
    fn from(soil_profile: SoilProfileCreate) -> Self {
        let now = chrono::Utc::now().naive_utc();

        crate::soil::profiles::db::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
            last_updated: sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc()),
            name: sea_orm::ActiveValue::Set(soil_profile.name),
            profile_iterator: sea_orm::ActiveValue::Set(soil_profile.profile_iterator),
            gradient: sea_orm::ActiveValue::Set(soil_profile.gradient),
            description_horizon: sea_orm::ActiveValue::Set(soil_profile.description_horizon),
            weather: sea_orm::ActiveValue::Set(soil_profile.weather),
            topography: sea_orm::ActiveValue::Set(soil_profile.topography),
            vegetation_type: sea_orm::ActiveValue::Set(soil_profile.vegetation_type),
            aspect: sea_orm::ActiveValue::Set(soil_profile.aspect),
            lythology_surficial_deposit: sea_orm::ActiveValue::Set(
                soil_profile.lythology_surficial_deposit,
            ),
            created_on: sea_orm::ActiveValue::Set(Some(now)),
            soil_type_id: sea_orm::ActiveValue::Set(soil_profile.soil_type_id),
            area_id: sea_orm::ActiveValue::Set(soil_profile.area_id),
            soil_diagram: sea_orm::ActiveValue::Set(soil_profile.soil_diagram),
            photo: sea_orm::ActiveValue::Set(soil_profile.photo),
            parent_material: sea_orm::ActiveValue::Set(soil_profile.parent_material),
            iterator: sea_orm::ActiveValue::NotSet,
        }
    }
}
