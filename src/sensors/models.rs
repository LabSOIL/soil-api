use chrono::NaiveDateTime;
use sea_orm::ColumnTrait;
use sea_orm::FromQueryResult;
use sea_orm::{query::*, Condition, DatabaseConnection, EntityTrait};
use sea_query::Order;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct SensorWithoutData {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub area: crate::areas::models::AreaBasicWithProject,
}

impl SensorWithoutData {
    pub async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: <super::db::Entity as sea_orm::EntityTrait>::Column,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Vec<Self> {
        let sensors = crate::sensors::db::Entity::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await
            .unwrap();

        let mut sensor_without_data: Vec<SensorWithoutData> = Vec::new();

        for sensor in sensors {
            let area = crate::areas::db::Entity::find()
                .filter(crate::areas::db::Column::Id.eq(sensor.area_id))
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let project = crate::projects::db::Entity::find()
                .filter(crate::projects::db::Column::Id.eq(area.project_id))
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let area_with_project = crate::areas::models::AreaBasicWithProject {
                id: area.id,
                name: area.name,
                project: crate::common::models::GenericNameAndID {
                    id: project.id,
                    name: project.name,
                },
            };

            sensor_without_data.push(SensorWithoutData {
                id: sensor.id,
                name: sensor.name,
                description: sensor.description,
                area: area_with_project,
            });
        }

        sensor_without_data
    }
}

#[derive(ToSchema, Serialize)]
pub struct SensorData {
    pub id: Uuid,
    pub instrument_seq: i32,
    pub time_utc: NaiveDateTime,
    pub temperature_1: Option<f64>,
    pub temperature_2: Option<f64>,
    pub temperature_3: Option<f64>,
    pub soil_moisture_count: Option<f64>,
    pub shake: Option<i32>,
    pub error_flat: Option<i32>,
    pub temperature_average: Option<f64>,
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
