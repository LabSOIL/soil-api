use crate::common::auth::Role;
use crate::gnss::models::Gnss;
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
    Gnss: CRUDResource,
{
    let mut mutating_router = Router::new()
        .route(
            "/",
            get(crud::get_all::<Gnss>).post(crud::create_one::<Gnss>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<Gnss>)
                .put(crud::update_one::<Gnss>)
                .delete(crud::delete_one::<Gnss>),
        )
        .route("/batch", delete(crud::delete_many::<Gnss>))
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
            Gnss::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}
