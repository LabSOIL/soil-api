use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "gnss")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub time: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub original_filename: Option<String>,
    pub elevation_gps: Option<f64>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_srid: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
