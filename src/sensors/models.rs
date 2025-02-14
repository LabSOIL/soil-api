use super::db::{ActiveModel, Model};
use crate::areas::models::AreaBasicWithProject;
use crate::common::crud::traits::CRUDResource;
use crate::sensors::data::models::SensorData;
use crate::sensors::db;
use async_trait::async_trait;
use sea_orm::sea_query::Expr;

use sea_orm::{
    entity::prelude::*, query::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition,
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, Order,
};
use sea_orm::{NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// #[derive(ToSchema, Serialize)]
// pub struct Sensor {
//     id: Uuid,
//     name: Option<String>,
//     serial_number: Option<String>,
//     manufacturer: Option<String>,
//     description: Option<String>,
//     area_id: Uuid,
//     latitude: Option<f64>,
//     longitude: Option<f64>,
//     coord_srid: Option<i32>,
//     coord_x: Option<f64>,
//     coord_y: Option<f64>,
//     coord_z: Option<f64>,
//     data: Option<Vec<crate::sensors::data::models::SensorData>>,
//     area: Option<crate::areas::models::AreaBasicWithProject>,
// }

// impl From<Model> for Sensor {
//     fn from(model: Model) -> Self {
//         Self {
//             id: model.id,
//             name: model.name,
//             serial_number: model.serial_number,
//             manufacturer: model.manufacturer,
//             description: model.description,
//             area_id: model.area_id,
//             latitude: None,
//             longitude: None,
//             coord_srid: None,
//             coord_x: None,
//             coord_y: None,
//             coord_z: None,
//             data: None,
//             area: None,
//         }
//     }
// }

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
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

// impl SensorSimple {
//     pub async fn from_area(area: &crate::areas::db::Model, db: &DatabaseConnection) -> Vec<Self> {
//         super::db::Entity::find()
//             .filter(super::db::Column::AreaId.eq(area.id))
//             .column_as(Expr::cust("ST_X(geom)"), "coord_x")
//             .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
//             .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
//             .column_as(Expr::cust("ST_SRID(geom)"), "coord_srid")
//             .column_as(Expr::cust("ST_X(st_transform(geom, 4326))"), "longitude")
//             .column_as(Expr::cust("ST_Y(st_transform(geom, 4326))"), "latitude")
//             .into_model::<SensorSimple>()
//             .all(db)
//             .await
//             .unwrap()
//     }
// }

// #[derive(Serialize, ToSchema)]
// pub struct SensorWithData {
//     pub id: Uuid,
//     pub name: Option<String>,
//     pub description: Option<String>,
//     pub data: Vec<crate::sensors::data::db::Model>,
//     pub coord_x: Option<f64>,
//     pub coord_y: Option<f64>,
//     pub coord_z: Option<f64>,
//     // pub closest_features: Vec<crate::common::models::ClosestFeature>,
// }
// #[derive(Serialize, ToSchema, FromQueryResult)]
// pub struct SensorWithCoords {
//     pub id: Uuid,
//     pub name: Option<String>,
//     pub description: Option<String>,
//     pub coord_x: Option<f64>,
//     pub coord_y: Option<f64>,
//     pub coord_z: Option<f64>,
// }

// #[derive(ToSchema, Deserialize)]
// pub struct SensorUpdate {
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub name: Option<Option<String>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub description: Option<Option<String>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub manufacturer: Option<Option<String>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub serial_number: Option<Option<String>>,
//     pub area_id: Option<Uuid>,
// }

// impl From<SensorUpdate> for ActiveModel {
//     fn from(update: SensorUpdate) -> Self {
//         // If the field is Some(None), update the field to None, if None,
//         // do not update the field (double option)

//         Self {
//             name: match update.name {
//                 Some(Some(name)) => Set(Some(name)),
//                 Some(_) => NotSet,
//                 _ => NotSet,
//             },
//             description: match update.description {
//                 Some(Some(description)) => Set(Some(description)),
//                 Some(_) => Set(None),
//                 _ => NotSet,
//             },
//             manufacturer: match update.manufacturer {
//                 Some(Some(manufacturer)) => Set(Some(manufacturer)),
//                 Some(_) => Set(None),
//                 _ => NotSet,
//             },
//             serial_number: match update.serial_number {
//                 Some(Some(serial_number)) => Set(Some(serial_number)),
//                 Some(_) => Set(None),
//                 _ => NotSet,
//             },
//             area_id: match update.area_id {
//                 Some(area_id) => Set(area_id),
//                 _ => NotSet,
//             },
//             last_updated: Set(chrono::Utc::now().naive_utc()),
//             comment: NotSet,
//             id: NotSet,
//         }
//     }
// }
// impl SensorUpdate {
//     pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
//         // If the field is Some(None), update the field to None, if None,
//         // do not update the field (double option)

//         model.name = match self.name {
//             Some(Some(ref name)) => Set(Some(name.clone())),
//             Some(_) => NotSet,
//             _ => NotSet,
//         };

//         model.description = match self.description {
//             Some(Some(ref description)) => Set(Some(description.clone())),
//             Some(_) => Set(None),
//             _ => NotSet,
//         };
//         model.last_updated = Set(chrono::Utc::now().naive_utc());

//         model
//     }
// }

// /// The creation model for sensors
// #[derive(Serialize, Deserialize, ToSchema)]
// pub struct SensorCreate {
//     pub name: Option<String>,
//     pub serial_number: Option<String>,
//     pub manufacturer: Option<String>,
//     pub description: Option<String>,
//     /// Optional comment that is stored in the DB (even if not returned in the API model)
//     pub comment: Option<String>,
//     pub area_id: Uuid,
// }

// /// Convert a SensorCreate into a DB active model.
// impl From<SensorCreate> for db::ActiveModel {
//     fn from(create: SensorCreate) -> Self {
//         let now = chrono::Utc::now().naive_utc();
//         db::ActiveModel {
//             id: ActiveValue::Set(Uuid::new_v4()),
//             name: ActiveValue::Set(create.name),
//             serial_number: ActiveValue::Set(create.serial_number),
//             manufacturer: ActiveValue::Set(create.manufacturer),
//             description: ActiveValue::Set(create.description),
//             comment: ActiveValue::Set(create.comment),
//             area_id: ActiveValue::Set(create.area_id),
//             last_updated: ActiveValue::Set(now),
//         }
//     }
// }

// /// The update model is already defined as SensorUpdate (with double‑option types)
// /// and you have a merge_into_activemodel() method on it.

// /// Implement the CRUDResource trait for Sensor
// #[async_trait]
// impl CRUDResource for Sensor {
//     type EntityType = db::Entity;
//     type ColumnType = db::Column;
//     type ModelType = db::Model;
//     type ActiveModelType = db::ActiveModel;
//     type ApiModel = Sensor;
//     type CreateModel = SensorCreate;
//     type UpdateModel = SensorUpdate;

//     const RESOURCE_NAME_SINGULAR: &'static str = "sensor";
//     const RESOURCE_NAME_PLURAL: &'static str = "sensors";

//     async fn get_all(
//         db: &DatabaseConnection,
//         condition: Condition,
//         order_column: Self::ColumnType,
//         order_direction: Order,
//         offset: u64,
//         limit: u64,
//     ) -> Result<Vec<Self::ApiModel>, DbErr> {
//         // Query sensors with custom SQL to load coordinate values.
//         // We also join the related area (so we can later build AreaBasicWithProject).
//         let tuples = Self::EntityType::find()
//             .filter(condition)
//             .order_by(order_column, order_direction)
//             // Adjust the following expressions if your sensor table actually has a geom column.
//             .column_as(Expr::cust("ST_X(sensor.geom)"), "coord_x")
//             .column_as(Expr::cust("ST_Y(sensor.geom)"), "coord_y")
//             .column_as(Expr::cust("ST_Z(sensor.geom)"), "coord_z")
//             .find_also_related(crate::areas::db::Entity)
//             .into_model::<SensorSimple, crate::areas::db::Model>()
//             .all(db)
//             .await?;

//         let sensors: Vec<Sensor> = tuples
//             .into_iter()
//             .map(|(simple, area_vec)| {
//                 let area_opt = match area_vec.into_iter().next() {
//                     Some(area_db) => {
//                         Some(crate::areas::models::AreaBasicWithProject::from(area_db))
//                     }
//                     None => None,
//                 };
//                 // Construct the API model manually. (In this example we copy fields from SensorSimple.)
//                 Sensor {
//                     id: simple.id,
//                     name: simple.name,
//                     serial_number: simple.serial_number,
//                     manufacturer: simple.manufacturer,
//                     description: simple.description,
//                     area_id: simple.area_id,
//                     latitude: simple.latitude,
//                     longitude: simple.longitude,
//                     coord_srid: simple.coord_srid,
//                     coord_x: simple.coord_x,
//                     coord_y: simple.coord_y,
//                     coord_z: simple.coord_z,
//                     data: None, // Sensor data can be fetched by a separate endpoint
//                     area: area_opt,
//                 }
//             })
//             .collect();
//         Ok(sensors)
//     }

//     async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
//         let tuple_opt = Self::EntityType::find()
//             .filter(db::Column::Id.eq(id))
//             .column_as(Expr::cust("ST_X(sensor.geom)"), "coord_x")
//             .column_as(Expr::cust("ST_Y(sensor.geom)"), "coord_y")
//             .column_as(Expr::cust("ST_Z(sensor.geom)"), "coord_z")
//             .find_also_related(crate::areas::db::Entity)
//             .into_model::<SensorSimple>()
//             .one(db)
//             .await?
//             .map(|(simple, area_vec)| (simple, area_vec.into_iter().next()))
//             .await?;
//         if let Some((simple, area_vec)) = tuple_opt {
//             let area_opt = area_vec
//                 .into_iter()
//                 .next()
//                 .map(|area_db| crate::areas::models::AreaBasicWithProject::from(area_db));
//             Ok(Sensor {
//                 id: simple.id,
//                 name: simple.name,
//                 serial_number: simple.serial_number,
//                 manufacturer: simple.manufacturer,
//                 description: simple.description,
//                 area_id: simple.area_id,
//                 latitude: simple.latitude,
//                 longitude: simple.longitude,
//                 coord_srid: simple.coord_srid,
//                 coord_x: simple.coord_x,
//                 coord_y: simple.coord_y,
//                 coord_z: simple.coord_z,
//                 data: None,
//                 area: area_opt,
//             })
//         } else {
//             Err(DbErr::RecordNotFound("Sensor not found".into()))
//         }
//     }

//     async fn create(
//         db: &DatabaseConnection,
//         create_model: Self::CreateModel,
//     ) -> Result<Self::ApiModel, DbErr> {
//         let active_model: Self::ActiveModelType = create_model.into();
//         let inserted = active_model.insert(db).await?;
//         Self::get_one(db, inserted.id).await
//     }

//     async fn update(
//         db: &DatabaseConnection,
//         id: Uuid,
//         update_model: Self::UpdateModel,
//     ) -> Result<Self::ApiModel, DbErr> {
//         let existing: Self::ActiveModelType = Self::EntityType::find()
//             .filter(db::Column::Id.eq(id))
//             .one(db)
//             .await?
//             .ok_or(DbErr::RecordNotFound("Sensor not found".into()))?
//             .into();
//         let updated_model = update_model.merge_into_activemodel(existing);
//         let updated = updated_model.update(db).await?;
//         Self::get_one(db, updated.id).await
//     }

//     async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
//         let res = Self::EntityType::delete_by_id(id).exec(db).await?;
//         Ok(res.rows_affected as usize)
//     }

//     async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr> {
//         Self::EntityType::delete_many()
//             .filter(db::Column::Id.is_in(ids.clone()))
//             .exec(db)
//             .await?;
//         Ok(ids)
//     }

//     async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64 {
//         Self::EntityType::find()
//             .filter(condition)
//             .count(db)
//             .await
//             .unwrap_or(0)
//     }

//     fn default_index_column() -> Self::ColumnType {
//         db::Column::Id
//     }

//     fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
//         &[("id", db::Column::Id), ("name", db::Column::Name)]
//     }

//     fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
//         &[("id", db::Column::Id), ("name", db::Column::Name)]
//     }
// }
