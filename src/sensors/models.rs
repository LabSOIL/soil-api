use sea_orm::{
    entity::prelude::*, query::*, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult,
};
use serde::Serialize;
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
    // pub async fn get_all(
    //     db: &DatabaseConnection,
    //     condition: Condition,
    //     order_column: <super::db::Entity as sea_orm::EntityTrait>::Column,
    //     order_direction: Order,
    //     offset: u64,
    //     limit: u64,
    // ) -> Vec<Self> {
    //     let sensors = crate::sensors::db::Entity::find()
    //         .filter(condition)
    //         .order_by(order_column, order_direction)
    //         .offset(offset)
    //         .limit(limit)
    //         .all(db)
    //         .await
    //         .unwrap();

    //     let mut sensor_without_data: Vec<SensorSimple> = Vec::new();

    //     for sensor in sensors {
    //         let area = crate::areas::db::Entity::find()
    //             .filter(crate::areas::db::Column::Id.eq(sensor.area_id))
    //             .one(db)
    //             .await
    //             .unwrap()
    //             .unwrap();

    //         let project = crate::projects::db::Entity::find()
    //             .filter(crate::projects::db::Column::Id.eq(area.project_id))
    //             .one(db)
    //             .await
    //             .unwrap()
    //             .unwrap();

    //         let area_with_project = crate::areas::models::AreaBasicWithProject {
    //             id: area.id,
    //             name: area.name,
    //             project: crate::common::models::GenericNameAndID {
    //                 id: project.id,
    //                 name: project.name,
    //             },
    //         };

    //         sensor_without_data.push(SensorSimple {
    //             id: sensor.id,
    //             name: sensor.name,
    //             description: sensor.description,
    //             area_id: sensor.area_id,
    //             manufacturer: sensor.manufacturer,
    //             serial_number: sensor.serial_number,
    //             latitude: sensor.latitude,
    //             longitude: sensor.longitude,
    //             coord_srid: sensor.coord_srid,
    //             coord_x: sensor.coord_x,
    //             coord_y: sensor.coord_y,
    //             coord_z: sensor.coord_z,
    //         });
    //     }

    //     sensor_without_data
    // }
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
