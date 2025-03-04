use super::models::{Plot, PlotCreate, PlotUpdate};
use crate::common::auth::Role;
// use axum::{
//     Router,
//     routing::{delete, get},
// };
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crudcrate::{CRUDResource, crud_handlers};
use utoipa_axum::{router::OpenApiRouter, routes};
// get_one!(Plot);
// create_one!(Plot, PlotCreate);
// update_one!(Plot, PlotUpdate);
// get_all!(Plot);
// delete_many!(Plot);
// delete_one!(Plot);

crud_handlers!(Plot, PlotUpdate, PlotCreate);

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    Plot: CRUDResource,
{
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one_handler))
        // .routes(routes!(get_all_handler))
        .routes(routes!(create_one_handler))
        // .routes(routes!(update_one_handler))
        // .routes(routes!(delete_one_handler))
        // .routes(routes!(delete_many_handler))
        .with_state(db.clone());

    if let Some(instance) = keycloak_auth_instance {
        mutating_router = mutating_router.layer(
            KeycloakAuthLayer::<Role>::builder()
                .instance(instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![Role::Administrator])
                .build(),
        );
    } else {
        println!(
            "Warning: Mutating routes of {} router are not protected",
            Plot::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}
