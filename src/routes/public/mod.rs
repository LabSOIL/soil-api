pub mod areas;
pub mod sensors;
pub mod website_access;

use crate::config::Config;
use crate::services::FallbackIpKeyExtractor;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use utoipa_axum::router::OpenApiRouter;

pub fn router(db: &DatabaseConnection) -> OpenApiRouter {
    let config = Config::from_env();

    let (per_second, burst) = if config.disable_rate_limiting {
        (10_000, 100_000) // effectively unlimited
    } else {
        (config.rate_limit_public_per_second, config.rate_limit_public_burst)
    };

    let limiter = GovernorConfigBuilder::default()
        .key_extractor(FallbackIpKeyExtractor)
        .per_second(per_second)
        .burst_size(burst)
        .finish()
        .expect("Failed to create public rate limiter");

    OpenApiRouter::new()
        .with_state(db.clone())
        .nest("/areas", areas::views::router(db))
        .nest("/sensors", sensors::views::router(db))
        .layer(GovernorLayer {
            config: Arc::new(limiter),
        })
}
