use crate::transects::nodes::models::TransectNode;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct Transect {
    pub id: Uuid,
    pub name: Option<String>,
    pub nodes: Vec<TransectNode>,
}
