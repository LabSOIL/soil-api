use crate::plots::models::Plot;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct TransectNode {
    pub order: i32,
    pub plot_id: Uuid,
    #[schema(no_recursion)]
    pub plot: Option<Plot>,
}
