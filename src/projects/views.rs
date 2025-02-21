use crate::common::auth::Role;
use crate::projects::models::Project;
use axum::{
    routing::{delete, get},
    Router,
};
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use crudcrate::{routes as crud, CRUDResource};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub fn router(
    db: DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> Router
where
    Project: CRUDResource,
{
    let mut mutating_router = Router::new()
        .route(
            "/",
            get(crud::get_all::<Project>).post(crud::create_one::<Project>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Project>)
                .put(crud::update_one::<Project>)
                .delete(crud::delete_one::<Project>),
        )
        .route("/batch", delete(crud::delete_many::<Project>))
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
            Project::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}
