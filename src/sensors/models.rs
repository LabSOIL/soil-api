use super::db::ActiveModel;
use sea_orm::{
    entity::prelude::*, query::*, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult,
};
use sea_orm::{NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct Sensor {
    id: Uuid,
    name: Option<String>,
    serial_number: Option<String>,
    manufacturer: Option<String>,
    description: Option<String>,
    area_id: Uuid,
    latitude: Option<f64>,
    longitude: Option<f64>,
    coord_srid: Option<i32>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
    data: Option<Vec<crate::sensors::data::models::SensorData>>,
    area: Option<crate::areas::models::AreaBasicWithProject>,
}

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct SensorSimple {
    id: Uuid,
    name: Option<String>,
    serial_number: Option<String>,
    manufacturer: Option<String>,
    description: Option<String>,
    area_id: Uuid,
    latitude: Option<f64>,
    longitude: Option<f64>,
    coord_srid: Option<i32>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
}

impl From<super::db::Model> for SensorSimple {
    fn from(model: super::db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            serial_number: model.serial_number,
            manufacturer: model.manufacturer,
            description: model.description,
            area_id: model.area_id,
            latitude: None,
            longitude: None,
            coord_srid: None,
            coord_x: None,
            coord_y: None,
            coord_z: None,
        }
    }
}

impl SensorSimple {
    pub async fn from_area(area: &crate::areas::db::Model, db: &DatabaseConnection) -> Vec<Self> {
        super::db::Entity::find()
            .filter(super::db::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
            .column_as(Expr::cust("ST_SRID(geom)"), "coord_srid")
            .column_as(Expr::cust("ST_X(st_transform(geom, 4326))"), "longitude")
            .column_as(Expr::cust("ST_Y(st_transform(geom, 4326))"), "latitude")
            .into_model::<SensorSimple>()
            .all(db)
            .await
            .unwrap()
    }
}

#[derive(Serialize, ToSchema)]
pub struct SensorWithData {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub data: Vec<crate::sensors::data::db::Model>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    // pub closest_features: Vec<crate::common::models::ClosestFeature>,
}
#[derive(Serialize, ToSchema, FromQueryResult)]
pub struct SensorWithCoords {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
}

#[derive(ToSchema, Deserialize)]
pub struct SensorUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub description: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub manufacturer: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub serial_number: Option<Option<String>>,
    pub area_id: Option<Uuid>,
}

impl From<SensorUpdate> for ActiveModel {
    fn from(update: SensorUpdate) -> Self {
        // If the field is Some(None), update the field to None, if None,
        // do not update the field (double option)

        Self {
            name: match update.name {
                Some(Some(name)) => Set(Some(name)),
                Some(_) => NotSet,
                _ => NotSet,
            },
            description: match update.description {
                Some(Some(description)) => Set(Some(description)),
                Some(_) => Set(None),
                _ => NotSet,
            },
            manufacturer: match update.manufacturer {
                Some(Some(manufacturer)) => Set(Some(manufacturer)),
                Some(_) => Set(None),
                _ => NotSet,
            },
            serial_number: match update.serial_number {
                Some(Some(serial_number)) => Set(Some(serial_number)),
                Some(_) => Set(None),
                _ => NotSet,
            },
            area_id: match update.area_id {
                Some(area_id) => Set(area_id),
                _ => NotSet,
            },
            last_updated: Set(chrono::Utc::now().naive_utc()),
            iterator: NotSet,
            comment: NotSet,
            id: NotSet,
        }
    }
}
impl SensorUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        // If the field is Some(None), update the field to None, if None,
        // do not update the field (double option)

        model.name = match self.name {
            Some(Some(ref name)) => Set(Some(name.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.description = match self.description {
            Some(Some(ref description)) => Set(Some(description.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };
        model.last_updated = Set(chrono::Utc::now().naive_utc());

        model
    }
}
