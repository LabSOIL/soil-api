use super::models::Sensor;
use crate::common::auth::Role;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use crudcrate::routes as crud;
use crudcrate::CRUDResource;
use sea_orm::{DatabaseConnection, DbErr};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

pub fn router(
    db: DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> Router {
    let mut mutating_router = Router::new()
        .route(
            "/",
            get(crud::get_all::<Sensor>).post(crud::create_one::<Sensor>),
        )
        .route(
            "/{id}",
            get(get_one::<Sensor>)
                .put(crud::update_one::<Sensor>)
                .delete(crud::delete_one::<Sensor>),
        )
        .route("/batch", delete(crud::delete_many::<Sensor>))
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
            Sensor::RESOURCE_NAME_PLURAL
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
    <T as CRUDResource>::ApiModel: From<Sensor>,
{
    println!("High resolution: {}", query.high_resolution);
    if query.high_resolution {
        match T::get_one(&db, id).await {
            Ok(item) => Ok(Json(item)),
            Err(DbErr::RecordNotFound(_)) => {
                Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Internal Server Error".to_string()),
                ))
            }
        }
    } else {
        match Sensor::get_one_low_resolution(&db, id).await {
            Ok(item) => Ok(Json(item.into())),
            Err(DbErr::RecordNotFound(_)) => {
                Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())))
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Internal Server Error".to_string()),
                ))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct LowResolution {
    #[serde(default)]
    pub high_resolution: bool,
}
