use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use soil_sensor_toolbox::SoilType;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "soil_type_enum")]
pub enum SoilTypeEnum {
    #[sea_orm(string_value = "sand")]
    Sand,
    #[sea_orm(string_value = "loamysanda")]
    LoamySandA,
    #[sea_orm(string_value = "loamysandb")]
    LoamySandB,
    #[sea_orm(string_value = "sandyloama")]
    SandyLoamA,
    #[sea_orm(string_value = "sandyloamb")]
    SandyLoamB,
    #[sea_orm(string_value = "loam")]
    Loam,
    #[sea_orm(string_value = "siltloam")]
    SiltLoam,
    #[sea_orm(string_value = "peat")]
    Peat,
    #[sea_orm(string_value = "water")]
    Water,
    #[sea_orm(string_value = "universal")]
    Universal,
    #[sea_orm(string_value = "sandtms1")]
    SandTMS1,
    #[sea_orm(string_value = "loamysandtms1")]
    LoamySandTMS1,
    #[sea_orm(string_value = "siltloamtms1")]
    SiltLoamTMS1,
}

impl From<SoilType> for SoilTypeEnum {
    fn from(soil_type: SoilType) -> Self {
        match soil_type {
            SoilType::Sand => SoilTypeEnum::Sand,
            SoilType::LoamySandA => SoilTypeEnum::LoamySandA,
            SoilType::LoamySandB => SoilTypeEnum::LoamySandB,
            SoilType::SandyLoamA => SoilTypeEnum::SandyLoamA,
            SoilType::SandyLoamB => SoilTypeEnum::SandyLoamB,
            SoilType::Loam => SoilTypeEnum::Loam,
            SoilType::SiltLoam => SoilTypeEnum::SiltLoam,
            SoilType::Peat => SoilTypeEnum::Peat,
            SoilType::Water => SoilTypeEnum::Water,
            SoilType::Universal => SoilTypeEnum::Universal,
            SoilType::SandTMS1 => SoilTypeEnum::SandTMS1,
            SoilType::LoamySandTMS1 => SoilTypeEnum::LoamySandTMS1,
            SoilType::SiltLoamTMS1 => SoilTypeEnum::SiltLoamTMS1,
        }
    }
}

impl From<SoilTypeEnum> for SoilType {
    fn from(soil_type_enum: SoilTypeEnum) -> Self {
        match soil_type_enum {
            SoilTypeEnum::Sand => SoilType::Sand,
            SoilTypeEnum::LoamySandA => SoilType::LoamySandA,
            SoilTypeEnum::LoamySandB => SoilType::LoamySandB,
            SoilTypeEnum::SandyLoamA => SoilType::SandyLoamA,
            SoilTypeEnum::SandyLoamB => SoilType::SandyLoamB,
            SoilTypeEnum::Loam => SoilType::Loam,
            SoilTypeEnum::SiltLoam => SoilType::SiltLoam,
            SoilTypeEnum::Peat => SoilType::Peat,
            SoilTypeEnum::Water => SoilType::Water,
            SoilTypeEnum::Universal => SoilType::Universal,
            SoilTypeEnum::SandTMS1 => SoilType::SandTMS1,
            SoilTypeEnum::LoamySandTMS1 => SoilType::LoamySandTMS1,
            SoilTypeEnum::SiltLoamTMS1 => SoilType::SiltLoamTMS1,
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, ToSchema,
)]
#[serde(rename_all = "lowercase")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "profile_type_enum")]
pub enum ProfileTypeEnum {
    #[sea_orm(string_value = "tms")]
    Tms,
    #[sea_orm(string_value = "chamber")]
    Chamber,
    #[sea_orm(string_value = "redox")]
    Redox,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "sensorprofile")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub area_id: Uuid,
    pub profile_type: ProfileTypeEnum,
    pub soil_type_vwc: Option<SoilTypeEnum>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
    pub coord_srid: Option<i32>,
    pub volume_ml: Option<f64>,
    pub area_cm2: Option<f64>,
    pub instrument_model: Option<String>,
    pub chamber_id_external: Option<String>,
    pub position: Option<i32>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::routes::private::areas::db::Entity",
        from = "Column::AreaId",
        to = "crate::routes::private::areas::db::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Area,
    #[sea_orm(has_many = "crate::routes::private::sensors::profile::assignment::db::Entity")]
    SensorprofileAssignment,
    #[sea_orm(has_many = "crate::routes::private::sensors::flux_data::db::Entity")]
    FluxData,
    #[sea_orm(has_many = "crate::routes::private::sensors::redox_data::db::Entity")]
    RedoxData,
}

impl Related<crate::routes::private::areas::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl Related<crate::routes::private::sensors::profile::assignment::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SensorprofileAssignment.def()
    }
}

impl Related<crate::routes::private::sensors::flux_data::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FluxData.def()
    }
}

impl Related<crate::routes::private::sensors::redox_data::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RedoxData.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
