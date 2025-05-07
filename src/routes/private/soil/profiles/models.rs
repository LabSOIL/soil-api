use super::db::Model;
use crate::config::Config;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult,
    entity::prelude::*,
    query::{Condition, Order, QueryOrder, QuerySelect},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    ToSchema, Serialize, Deserialize, FromQueryResult, ToCreateModel, ToUpdateModel, Clone,
)]
#[active_model = "super::db::ActiveModel"]
pub struct SoilProfile {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub gradient: String,
    pub description_horizon: Option<Value>,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now(), on_create = chrono::Utc::now())]
    pub last_updated: chrono::DateTime<Utc>,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_create = chrono::Utc::now())]
    pub created_on: Option<DateTime<Utc>>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    pub soil_diagram: Option<String>,
    pub photo: Option<String>,
    pub parent_material: Option<f64>,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    pub coord_srid: i32,
    pub coord_x: f64,
    pub coord_y: f64,
    pub coord_z: f64,
}

impl From<Model> for SoilProfile {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
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
            coord_srid: model.coord_srid,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
        }
    }
}

#[async_trait]
impl CRUDResource for SoilProfile {
    type EntityType = crate::routes::private::soil::profiles::db::Entity;
    type ColumnType = crate::routes::private::soil::profiles::db::Column;
    type ActiveModelType = crate::routes::private::soil::profiles::db::ActiveModel;
    type CreateModel = SoilProfileCreate;
    type UpdateModel = SoilProfileUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "soil profile";
    const RESOURCE_NAME_PLURAL: &'static str = "soil profiles";
    const RESOURCE_DESCRIPTION: &'static str = "Soil profiles are a collection of soil horizons that describe the soil at a specific location.";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self>, DbErr> {
        let profiles: Vec<SoilProfile> = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?
            .into_iter()
            .map(std::convert::Into::into)
            .collect();
        Ok(profiles)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let profile: SoilProfile = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Soil profile not found".into()))?
            .into();
        Ok(profile)
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_data: Self::UpdateModel,
    ) -> Result<Self, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Soil profile not found".into()))?
            .into();
        let updated_model = update_data.merge_into_activemodel(existing);
        let updated = updated_model.update(db).await?;
        Self::get_one(db, updated.id).await
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
            ("gradient", Self::ColumnType::Gradient),
            ("weather", Self::ColumnType::Weather),
            ("topography", Self::ColumnType::Topography),
            ("vegetation_type", Self::ColumnType::VegetationType),
            ("aspect", Self::ColumnType::Aspect),
            (
                "lythology_surficial_deposit",
                Self::ColumnType::LythologySurficialDeposit,
            ),
        ]
    }
}
