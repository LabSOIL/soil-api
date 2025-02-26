use super::db::Model;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct GNSS {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(
        update_model = false,
        create_model = false,
        on_update = Utc::now(),
        on_create = Utc::now()
    )]
    pub last_updated: DateTime<Utc>,
    pub time: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub original_filename: Option<String>,
    pub elevation_gps: Option<f64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_srid: Option<i32>,
}

impl From<Model> for GNSS {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: model.name,
            time: model.time,
            comment: model.comment,
            original_filename: model.original_filename,
            elevation_gps: model.elevation_gps,
            latitude: model.latitude,
            longitude: model.longitude,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_srid: model.coord_srid,
        }
    }
}

#[async_trait]
impl CRUDResource for GNSS {
    type EntityType = crate::gnss::db::Entity;
    type ColumnType = crate::gnss::db::Column;
    type ModelType = crate::gnss::db::Model;
    type ActiveModelType = crate::gnss::db::ActiveModel;
    type ApiModel = GNSS;
    type CreateModel = GNSSCreate;
    type UpdateModel = GNSSUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "gnss";
    const RESOURCE_NAME_PLURAL: &'static str = "gnss";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        let models = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        Ok(models.into_iter().map(GNSS::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))?;
        Ok(GNSS::from(model))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))?
            .into();

        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let obj = Self::get_one(&db, response_obj.id).await?;
        Ok(obj)
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("last_updated", Self::ColumnType::LastUpdated),
            ("coord_x", Self::ColumnType::CoordX),
            ("coord_y", Self::ColumnType::CoordY),
            ("coord_srid", Self::ColumnType::CoordSrid),
            ("elevation_gps", Self::ColumnType::ElevationGps),
            ("latitude", Self::ColumnType::Latitude),
            ("longitude", Self::ColumnType::Longitude),
            ("original_filename", Self::ColumnType::OriginalFilename),
            ("time", Self::ColumnType::Time),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", Self::ColumnType::Name),
            ("original_filename", Self::ColumnType::OriginalFilename),
            ("comment", Self::ColumnType::Comment),
        ]
    }
}
