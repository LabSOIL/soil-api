use crate::plots::models::PlotSimple;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct TransectNode {
    pub id: Uuid,
    pub order: i32,
    pub plot: PlotSimple,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct TransectNodeAsPlotWithOrder {
    // A transect node is really a plot with an order value, this is similar to
    // the PlotSimple struct but with an additional order field
    pub id: Uuid,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_srid: i32,
    pub coord_x: f64,
    pub coord_y: f64,
    pub coord_z: f64,
    pub order: i32,
}
