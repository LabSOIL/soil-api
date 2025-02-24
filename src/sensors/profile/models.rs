use super::db::Model;
use crate::config::Config;
use async_trait::async_trait;
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct SensorProfile {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now().naive_utc(), on_create = chrono::Utc::now().naive_utc())]
    pub last_updated: chrono::NaiveDateTime,
    pub name: String,
    pub description: Option<String>,
    pub area_id: Uuid,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    pub coord_srid: Option<i32>,
    #[crudcrate(update_model = false, create_model = false)]
    pub assignments: Vec<crate::sensors::profile::assignment::models::SensorProfileAssignment>,
    #[crudcrate(non_db_attr = true, default = vec![])]
    pub data: Vec<crate::sensors::data::models::SensorData>,
}

impl From<Model> for SensorProfile {
    fn from(model: Model) -> Self {
        Self::from_with_assignments(model, vec![])
    }
}

impl SensorProfile {
    fn from_with_assignments(
        model: Model,
        assignments: Vec<crate::sensors::profile::assignment::models::SensorProfileAssignment>,
    ) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: model.name,
            description: model.description,
            area_id: model.area_id,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
            coord_srid: model.coord_srid,
            assignments,
            data: vec![],
        }
    }
}

#[async_trait]
impl CRUDResource for SensorProfile {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = SensorProfile;
    type CreateModel = SensorProfileCreate;
    type UpdateModel = SensorProfileUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "sensorprofile";
    const RESOURCE_NAME_PLURAL: &'static str = "sensorprofiles";

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

        let assignments: Vec<super::assignment::models::SensorProfileAssignment> = models
            .load_many(super::assignment::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .into_iter()
            .map(|(assignment)| assignment.into())
            .collect();

        let mut sensor_profiles: Vec<SensorProfile> = Vec::new();
        for model in models {
            let sensor_profile = SensorProfile::from_with_assignments(model, assignments.clone());
            sensor_profiles.push(sensor_profile);
        }
        Ok(sensor_profiles)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let mut model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .all(db)
            .await?;

        let assignments: Vec<super::assignment::models::SensorProfileAssignment> = model
            .load_many(super::assignment::db::Entity, db)
            .await?
            .pop()
            .unwrap()
            .into_iter()
            .map(|(assignment)| assignment.into())
            .collect();

        let model = model.pop().ok_or(DbErr::RecordNotFound(
            format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
        ))?;

        let sensor_profile = SensorProfile::from_with_assignments(model, assignments);
        Ok(sensor_profile)
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
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", Self::ColumnType::Name),
            ("description", Self::ColumnType::Description),
        ]
    }
}
