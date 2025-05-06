use super::models::{Sensor, SensorCreate, SensorUpdate};
use crate::common::auth::Role;
use crate::common::models::LowResolution;
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use crudcrate::{CRUDResource, crud_handlers};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

crud_handlers!(Sensor, SensorUpdate, SensorCreate);

#[utoipa::path(
    get,
    path = "/{id}",
    responses(
        (status = 200, description = "Sensor found", body = Sensor),
        (status = 404, description = "Sensor not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, description = "Sensor ID"),
        ("high_resolution" = bool, Query, description = "High resolution data flag")
    ),
    summary = format!("Get one {}", Sensor::RESOURCE_NAME_SINGULAR),
    description = format!("Retrieves one {} by its ID.\n\n{}", Sensor::RESOURCE_NAME_SINGULAR, Sensor::RESOURCE_DESCRIPTION)
)]
pub async fn get_one_sensor(
    axum::extract::State(db): axum::extract::State<sea_orm::DatabaseConnection>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(query): axum::extract::Query<LowResolution>,
) -> Result<Json<Sensor>, (axum::http::StatusCode, axum::Json<String>)> {
    if query.high_resolution {
        match <Sensor as CRUDResource>::get_one(&db, id).await {
            Ok(item) => Ok(Json(item)),
            Err(DbErr::RecordNotFound(_)) => Err((
                axum::http::StatusCode::NOT_FOUND,
                Json("Not Found".to_string()),
            )),
            Err(_) => Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal Server Error".to_string()),
            )),
        }
    } else {
        match Sensor::get_one_low_resolution(&db, id).await {
            Ok(item) => Ok(Json(item)),
            Err(DbErr::RecordNotFound(_)) => Err((
                axum::http::StatusCode::NOT_FOUND,
                Json("Not Found".to_string()),
            )),
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

#[utoipa::path(
    delete,
    path = "/{id}/data",
    responses(
        (status = 200, description = "Sensor data deleted"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, description = "Sensor ID")
    ),
    summary = "Delete sensor data",
    description = "Deletes all data for a sensor by its given ID."
)]
pub async fn delete_sensor_data(
    axum::extract::State(db): axum::extract::State<sea_orm::DatabaseConnection>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, axum::Json<String>)> {
    crate::routes::sensors::data::db::Entity::delete_many()
        .filter(crate::routes::sensors::data::db::Column::SensorId.eq(id))
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

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    Sensor: CRUDResource,
{
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one_sensor))
        .routes(routes!(get_all_handler))
        .routes(routes!(create_one_handler))
        .routes(routes!(update_one_handler))
        .routes(routes!(delete_one_handler))
        .routes(routes!(delete_many_handler))
        .routes(routes!(delete_sensor_data))
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
