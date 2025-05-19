pub mod areas;
pub mod sensors;

use sea_orm::DatabaseConnection;
use utoipa_axum::router::OpenApiRouter;

pub fn router(db: &DatabaseConnection) -> OpenApiRouter {
    OpenApiRouter::new()
        .with_state(db.clone())
        .nest("/areas", areas::views::router(db))
        .nest("/sensors", sensors::views::router(db))
}
