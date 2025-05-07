pub mod areas;

use sea_orm::DatabaseConnection;
use utoipa_axum::router::OpenApiRouter;

pub fn router(db: &DatabaseConnection) -> OpenApiRouter {
    OpenApiRouter::new()
        .route("/", axum::routing::get(|| async { "hello" }))
        .with_state(db.clone())
        .nest("/areas", areas::views::router(db))
}
