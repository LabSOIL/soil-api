//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "gradientchoices")]
pub enum Gradientchoices {
    #[sea_orm(string_value = "flat")]
    Flat,
    #[sea_orm(string_value = "slope")]
    Slope,
}
