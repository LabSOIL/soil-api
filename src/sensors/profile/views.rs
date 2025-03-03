use super::models::SensorProfile;
use crate::common::auth::Role;
use crate::common::models::LowResolution;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get},
};
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use crudcrate::{CRUDResource, routes as crud};
use sea_orm::{DatabaseConnection, DbErr};
use std::sync::Arc;
use uuid::Uuid;

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> Router
where
    SensorProfile: CRUDResource,
{
    let mut mutating_router = Router::new()
        .route(
            "/",
            get(crud::get_all::<SensorProfile>).post(crud::create_one::<SensorProfile>),
        )
        .route(
            "/{id}",
            get(get_one::<SensorProfile>)
                .put(crud::update_one::<SensorProfile>)
                .delete(crud::delete_one::<SensorProfile>),
        )
        .route("/batch", delete(crud::delete_many::<SensorProfile>))
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
            SensorProfile::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}

pub async fn get_one<T>(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Query(query): Query<LowResolution>,
) -> Result<Json<T::ApiModel>, (StatusCode, Json<String>)>
where
    T: CRUDResource,
    <T as CRUDResource>::ApiModel: From<SensorProfile>,
{
    if query.high_resolution {
        match SensorProfile::get_one_high_resolution(&db, id).await {
            Ok(item) => Ok(Json(item.into())),
            Err(DbErr::RecordNotFound(_)) => {
                Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
            }
            Err(e) => {
                println!("Error: {e:?}");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Internal Server Error".to_string()),
                ))
            }
        }
    } else {
        match SensorProfile::get_one_low_resolution(&db, id).await {
            Ok(item) => Ok(Json(item.into())),
            Err(DbErr::RecordNotFound(_)) => {
                Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
            }
            Err(e) => {
                println!("Error: {e:?}");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Internal Server Error".to_string()),
                ))
            }
        }
    }
}
