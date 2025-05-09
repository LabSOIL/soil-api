use super::db::Model;
use crate::config::Config;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{ActiveValue, Condition, Order, QueryOrder, QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct SoilClassification {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub depth_upper_cm: i32,
    pub depth_lower_cm: i32,
    #[crudcrate(update_model = false, create_model = false, on_create = chrono::Utc::now())]
    pub created_on: DateTime<Utc>,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now(), on_create = chrono::Utc::now())]
    pub last_updated: DateTime<Utc>,
    pub sample_date: Option<chrono::NaiveDate>,
    pub fe_abundance_g_per_cm3: Option<f64>,
    pub area_id: Uuid,
    pub soil_type_id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    pub coord_srid: Option<i32>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    #[crudcrate(update_model = false, create_model = false, default = None)]
    pub area: Option<crate::routes::private::areas::db::Model>,
    #[crudcrate(update_model = false, create_model = false, default = None)]
    pub soil_type: Option<crate::routes::private::soil::types::db::Model>,
}

impl From<Model> for SoilClassification {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            depth_upper_cm: model.depth_upper_cm,
            depth_lower_cm: model.depth_lower_cm,
            created_on: model.created_on,
            last_updated: model.last_updated,
            sample_date: model.sample_date,
            fe_abundance_g_per_cm3: model.fe_abundance_g_per_cm3,
            area_id: model.area_id,
            soil_type_id: model.soil_type_id,
            coord_srid: model.coord_srid,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
            area: None,
            soil_type: None,
        }
    }
}

#[async_trait]
impl CRUDResource for SoilClassification {
    type EntityType = crate::routes::private::soil::classification::db::Entity;
    type ColumnType = crate::routes::private::soil::classification::db::Column;
    type ActiveModelType = crate::routes::private::soil::classification::db::ActiveModel;
    type CreateModel = SoilClassificationCreate;
    type UpdateModel = SoilClassificationUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "soil classification";
    const RESOURCE_NAME_PLURAL: &'static str = "soil classifications";
    const RESOURCE_DESCRIPTION: &'static str = "A soil classification entry links a soil type, depth and area with Fe abundance to be linked to plot samples.";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self>, DbErr> {
        let classifications = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;

        let mut responses: Vec<SoilClassification> = vec![];
        for classification in classifications {
            let area = classification
                .find_related(crate::routes::private::areas::db::Entity)
                .one(db)
                .await?;
            let soil_type = classification
                .find_related(crate::routes::private::soil::types::db::Entity)
                .one(db)
                .await?;
            let mut obj: SoilClassification = classification.into();
            obj.area = area;
            obj.soil_type = soil_type;
            responses.push(obj);
        }

        Ok(responses)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let classification = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Soil classification with id {id} not found"
            )))?;
        let area = classification
            .find_related(crate::routes::private::areas::db::Entity)
            .one(db)
            .await?;
        let soil_type = classification
            .find_related(crate::routes::private::soil::types::db::Entity)
            .one(db)
            .await?;
        let mut obj: SoilClassification = classification.into();
        obj.area = area;
        obj.soil_type = soil_type;
        Ok(obj)
    }
    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("last_updated", Self::ColumnType::LastUpdated),
            ("created_on", Self::ColumnType::CreatedOn),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", Self::ColumnType::Name),
            ("description", Self::ColumnType::Description),
        ]
    }
}
