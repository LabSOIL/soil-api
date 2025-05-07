use crate::routes::private::areas::db;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Area {
    pub id: Uuid,
    pub name: Option<String>,
}

impl From<db::Model> for Area {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
        }
    }
}
