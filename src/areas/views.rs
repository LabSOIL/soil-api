use crate::areas::models::Area;
use crate::common::auth::Role;
use axum::{
    Router,
    routing::{delete, get},
};
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use crudcrate::{CRUDResource, routes as crud};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> Router
where
    Area: CRUDResource,
{
    let mut mutating_router = Router::new()
        .route(
            "/",
            get(crud::get_all::<Area>).post(crud::create_one::<Area>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Area>)
                .put(crud::update_one::<Area>)
                .delete(crud::delete_one::<Area>),
        )
        .route("/batch", delete(crud::delete_many::<Area>))
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
            Area::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}
