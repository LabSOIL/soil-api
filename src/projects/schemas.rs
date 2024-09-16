use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Project {
    color: String,
    last_updated: NaiveDateTime,
    description: Option<String>,
    id: Uuid,
    name: String,
}

impl Project {
    pub fn from(obj: crate::projects::models::Model) -> Self {
        Project {
            id: obj.id,
            name: obj.name,
            color: obj.color,
            description: obj.description,
            last_updated: obj.last_updated,
        }
    }
}
