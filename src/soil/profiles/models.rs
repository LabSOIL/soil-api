use super::db::Model;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, query::*, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait,
    FromQueryResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult, ToCreateModel, ToUpdateModel)]
#[active_model = "super::db::ActiveModel"]
pub struct SoilProfile {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub gradient: String,
    pub description_horizon: Option<Value>,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now().naive_utc(), on_create = chrono::Utc::now().naive_utc())]
    pub last_updated: chrono::NaiveDateTime,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_create = chrono::Utc::now().naive_utc())]
    pub created_on: Option<NaiveDateTime>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    pub soil_diagram: Option<String>,
    pub photo: Option<String>,
    pub parent_material: Option<f64>,
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
    type EntityType = crate::soil::profiles::db::Entity;
    type ColumnType = crate::soil::profiles::db::Column;
    type ModelType = crate::soil::profiles::db::Model;
    type ActiveModelType = crate::soil::profiles::db::ActiveModel;
    type ApiModel = SoilProfile;
    type CreateModel = SoilProfileCreate;
    type UpdateModel = SoilProfileUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "soilprofile";
    const RESOURCE_NAME_PLURAL: &'static str = "soilprofiles";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        let profiles: Vec<SoilProfile> = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?
            .into_iter()
            .map(|model| model.into())
            .collect();
        Ok(profiles)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
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
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Soil profile not found".into()))?
            .into();
        let updated_model = update_model.merge_into_activemodel(existing);
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
