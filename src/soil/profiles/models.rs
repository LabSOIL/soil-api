use super::db::Model;
use crate::common::crud::traits::CRUDResource;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use sea_orm::{
    entity::prelude::*, query::*, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
pub struct SoilProfile {
    pub id: Uuid,
    pub name: String,
    pub gradient: String,
    pub description_horizon: Option<Value>,
    pub last_updated: chrono::NaiveDateTime,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    pub created_on: Option<NaiveDateTime>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    pub soil_diagram: Option<String>,
    pub photo: Option<String>,
    pub parent_material: Option<f64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_srid: Option<i32>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
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
            latitude: None,
            longitude: None,
            coord_srid: None,
            coord_x: None,
            coord_y: None,
            coord_z: None,
        }
    }
}

impl SoilProfile {
    pub async fn from_area(
        area: &crate::areas::db::Model,
        db: &DatabaseConnection,
    ) -> Vec<SoilProfile> {
        super::db::Entity::find()
            .filter(super::db::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(soilprofile.geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(soilprofile.geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(soilprofile.geom)"), "coord_z")
            .column_as(
                Expr::cust("ST_X(st_transform(soilprofile.geom, 4326))"),
                "longitude",
            )
            .column_as(
                Expr::cust("ST_Y(st_transform(soilprofile.geom, 4326))"),
                "latitude",
            )
            .column_as(Expr::cust("st_srid(soilprofile.geom)"), "coord_srid")
            .into_model::<SoilProfile>()
            .all(db)
            .await
            .unwrap()
    }
    pub async fn from_db(
        soil_profile: crate::soil::profiles::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        crate::soil::profiles::db::Entity::find()
            .filter(crate::soil::profiles::db::Column::Id.eq(soil_profile.id))
            .into_model::<SoilProfile>()
            .one(db)
            .await
            .unwrap()
            .unwrap()
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilProfileBasic {
    pub id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub name: Option<String>,
    pub description_horizon: Option<Value>,
}

impl From<crate::soil::profiles::db::Model> for SoilProfileBasic {
    fn from(soil_profile: crate::soil::profiles::db::Model) -> Self {
        SoilProfileBasic {
            id: soil_profile.id,
            last_updated: soil_profile.last_updated,
            name: Some(soil_profile.name),
            description_horizon: soil_profile.description_horizon,
        }
    }
}

impl SoilProfileBasic {
    pub async fn from_db(
        soil_profile: crate::soil::profiles::db::Model,
        db: &DatabaseConnection,
    ) -> Self {
        let soil_profile = crate::soil::profiles::db::Entity::find()
            .filter(crate::soil::profiles::db::Column::Id.eq(soil_profile.id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        SoilProfileBasic::from(soil_profile)
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct SoilProfileCreate {
    pub name: String,
    pub gradient: String,
    pub description_horizon: Option<Value>,
    pub weather: Option<String>,
    pub topography: Option<String>,
    pub vegetation_type: Option<String>,
    pub aspect: Option<String>,
    pub lythology_surficial_deposit: Option<String>,
    pub soil_type_id: Uuid,
    pub area_id: Uuid,
    pub soil_diagram: Option<String>,
    pub photo: Option<String>,
    pub parent_material: Option<f64>,
}

impl From<SoilProfileCreate> for crate::soil::profiles::db::ActiveModel {
    fn from(soil_profile: SoilProfileCreate) -> Self {
        let now = chrono::Utc::now().naive_utc();

        crate::soil::profiles::db::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
            last_updated: sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc()),
            name: sea_orm::ActiveValue::Set(soil_profile.name),
            gradient: sea_orm::ActiveValue::Set(soil_profile.gradient),
            description_horizon: sea_orm::ActiveValue::Set(soil_profile.description_horizon),
            weather: sea_orm::ActiveValue::Set(soil_profile.weather),
            topography: sea_orm::ActiveValue::Set(soil_profile.topography),
            vegetation_type: sea_orm::ActiveValue::Set(soil_profile.vegetation_type),
            aspect: sea_orm::ActiveValue::Set(soil_profile.aspect),
            lythology_surficial_deposit: sea_orm::ActiveValue::Set(
                soil_profile.lythology_surficial_deposit,
            ),
            created_on: sea_orm::ActiveValue::Set(Some(now)),
            soil_type_id: sea_orm::ActiveValue::Set(soil_profile.soil_type_id),
            area_id: sea_orm::ActiveValue::Set(soil_profile.area_id),
            soil_diagram: sea_orm::ActiveValue::Set(soil_profile.soil_diagram),
            photo: sea_orm::ActiveValue::Set(soil_profile.photo),
            parent_material: sea_orm::ActiveValue::Set(soil_profile.parent_material),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SoilProfileUpdate {
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
    pub gradient: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub description_horizon: Option<Option<serde_json::Value>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub weather: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub topography: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub vegetation_type: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub aspect: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub lythology_surficial_deposit: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub soil_diagram: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub photo: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub parent_material: Option<Option<f64>>,
}

impl SoilProfileUpdate {
    pub fn merge_into_activemodel(
        self,
        mut model: crate::soil::profiles::db::ActiveModel,
    ) -> crate::soil::profiles::db::ActiveModel {
        use sea_orm::ActiveValue;
        if let Some(opt) = self.name {
            model.name = ActiveValue::Set(opt.unwrap_or_default());
        }
        if let Some(opt) = self.gradient {
            model.gradient = ActiveValue::Set(opt.unwrap_or_default());
        }
        if let Some(opt) = self.description_horizon {
            model.description_horizon = ActiveValue::Set(opt);
        }
        if let Some(opt) = self.weather {
            model.weather = ActiveValue::Set(Some(opt.unwrap_or_default()));
        }
        if let Some(opt) = self.topography {
            model.topography = ActiveValue::Set(Some(opt.unwrap_or_default()));
        }
        if let Some(opt) = self.vegetation_type {
            model.vegetation_type = ActiveValue::Set(Some(opt.unwrap_or_default()));
        }
        if let Some(opt) = self.aspect {
            model.aspect = ActiveValue::Set(Some(opt.unwrap_or_default()));
        }
        if let Some(opt) = self.lythology_surficial_deposit {
            model.lythology_surficial_deposit = ActiveValue::Set(Some(opt.unwrap_or_default()));
        }
        if let Some(opt) = self.soil_diagram {
            model.soil_diagram = ActiveValue::Set(opt);
        }
        if let Some(opt) = self.photo {
            model.photo = ActiveValue::Set(opt);
        }
        if let Some(opt) = self.parent_material {
            model.parent_material = ActiveValue::Set(opt);
        }
        // Update the timestamp on every update.
        model.last_updated = ActiveValue::Set(chrono::Utc::now().naive_utc());
        model
    }
}

// ----------------------------------------------------------------------------
// Implement CRUDResource for SoilProfile
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
        let profiles = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .column_as(
                Expr::cust("ST_X(st_transform(soilprofile.geom, 4326))"),
                "longitude",
            )
            .column_as(
                Expr::cust("ST_Y(st_transform(soilprofile.geom, 4326))"),
                "latitude",
            )
            .column_as(Expr::cust("st_srid(soilprofile.geom)"), "coord_srid")
            .into_model::<SoilProfile>()
            .all(db)
            .await?;
        Ok(profiles)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let profile = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .column_as(
                Expr::cust("ST_X(st_transform(soilprofile.geom, 4326))"),
                "longitude",
            )
            .column_as(
                Expr::cust("ST_Y(st_transform(soilprofile.geom, 4326))"),
                "latitude",
            )
            .column_as(Expr::cust("st_srid(soilprofile.geom)"), "coord_srid")
            .into_model::<SoilProfile>()
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Soil profile not found".into()))?;
        Ok(profile)
    }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let active_model: Self::ActiveModelType = create_model.into();
        let inserted = active_model.insert(db).await?;
        Self::get_one(db, inserted.id).await
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
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("area_id", Self::ColumnType::AreaId),
        ]
    }
}
