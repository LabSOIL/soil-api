use super::db::{ActiveModel, Model};
use sea_orm::{entity::prelude::*, ColumnTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilType {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description: String,
    pub image: Option<String>,
}

impl From<Model> for SoilType {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: Some(model.name),
            description: model.description,
            image: model.image,
        }
    }
}

impl From<Model> for SoilTypeBasic {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: Some(model.name),
            description: model.description,
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeBasic {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description: String,
}

impl SoilType {
    pub async fn from_db(
        soil_type: crate::soil::types::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let soil_type = crate::soil::types::db::Entity::find()
            .filter(crate::soil::types::db::Column::Id.eq(soil_type.id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        SoilType::from(soil_type)
    }
}

impl SoilTypeBasic {
    pub async fn from_db(
        soil_type: crate::soil::types::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let soil_type = crate::soil::types::db::Entity::find()
            .filter(crate::soil::types::db::Column::Id.eq(soil_type.id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        SoilTypeBasic::from(soil_type)
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeCreate {
    pub name: String,
    pub description: String,
    pub image: Option<String>,
}

impl From<SoilTypeCreate> for crate::soil::types::db::ActiveModel {
    fn from(soil_type: SoilTypeCreate) -> Self {
        crate::soil::types::db::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
            last_updated: sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc()),
            name: sea_orm::ActiveValue::Set(soil_type.name),
            description: sea_orm::ActiveValue::Set(soil_type.description),
            image: sea_orm::ActiveValue::Set(soil_type.image),
            iterator: sea_orm::ActiveValue::NotSet,
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilTypeUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}
