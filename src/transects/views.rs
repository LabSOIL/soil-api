use super::models::Transect;
use crate::common::auth::Role;
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
    Transect: CRUDResource,
{
    let mut mutating_router = Router::new()
        .route(
            "/",
            get(crud::get_all::<Transect>).post(crud::create_one::<Transect>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Transect>)
                .put(crud::update_one::<Transect>)
                .delete(crud::delete_one::<Transect>),
        )
        .route("/batch", delete(crud::delete_many::<Transect>))
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
            Transect::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}
