use crate::routes::private::areas::db;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    pub geom: Option<Value>,
}

impl From<db::Model> for Area {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            geom: None, // Set later in func
        }
    }
}
