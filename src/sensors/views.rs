use super::models::Sensor;
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
use crudcrate::CRUDResource;
use crudcrate::routes as crud;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;
use uuid::Uuid;

pub fn router(
    db: &DatabaseConnection,
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
        .route("/{id}/data", delete(delete_data))
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
    if query.high_resolution {
        match T::get_one(&db, id).await {
            Ok(item) => Ok(Json(item)),
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
        match Sensor::get_one_low_resolution(&db, id).await {
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

pub async fn delete_data(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<String>)> {
    crate::sensors::data::db::Entity::delete_many()
        .filter(crate::sensors::data::db::Column::SensorId.eq(id))
        .exec(&db)
        .await
        .map_err(|e| {
            println!("Error: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal Server Error".to_string()),
            )
        })?;

    Ok(StatusCode::OK)
}
