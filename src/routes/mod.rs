mod areas;
mod gnss;
mod instrument_experiments;
mod plots;
mod projects;
mod samples;
mod sensors;
mod soil;
mod transects;

use crate::config::Config;
use axum::{Router, extract::DefaultBodyLimit};
use axum_keycloak_auth::{Url, instance::KeycloakAuthInstance, instance::KeycloakConfig};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

pub fn build_router(db: &DatabaseConnection) -> Router {
    #[derive(OpenApi)]
    #[openapi()]
    struct ApiDoc;

    let config: Config = Config::from_env();

    let keycloak_instance: Arc<KeycloakAuthInstance> = Arc::new(KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse(&config.keycloak_url).unwrap())
            .realm(String::from(&config.keycloak_realm))
            .build(),
    ));

    // Build the router with routes from the plots module
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(crate::common::views::router(db)) // Root routes
        .nest(
            "/api/plots",
            plots::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/areas",
            areas::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/projects",
            projects::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/gnss",
            gnss::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/plot_samples",
            samples::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/sensors",
            sensors::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/sensor_profiles",
            sensors::profile::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/sensor_profile_assignments",
            sensors::profile::assignment::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/transects",
            transects::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/instruments",
            instrument_experiments::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/instrument_channels",
            instrument_experiments::channels::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/soil_types",
            soil::types::views::router(db, Some(keycloak_instance.clone())),
        )
        .nest(
            "/api/soil_profiles",
            soil::profiles::views::router(db, Some(keycloak_instance.clone())),
        )
        .layer(DefaultBodyLimit::max(30 * 1024 * 1024))
        .split_for_parts();

    router.merge(Scalar::with_url("/api/docs", api))
}
