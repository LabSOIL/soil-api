use crate::transects::nodes::schemas::TransectNode;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct Transect {
    pub id: Uuid,
    pub name: Option<String>,
    pub nodes: Vec<TransectNode>,
}
