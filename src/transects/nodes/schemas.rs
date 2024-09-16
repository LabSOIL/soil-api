use crate::plots::schemas::PlotSimple;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct TransectNode {
    pub id: Uuid,
    pub name: Option<String>,
    pub order: i32,
    pub plot: PlotSimple,
}
