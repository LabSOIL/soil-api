use super::models::{SensorProfile, SensorProfileCreate, SensorProfileUpdate};
use crate::common::auth::Role;
use crate::common::models::DateRangeQuery;
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use crudcrate::{CRUDResource, crud_handlers};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

crud_handlers!(SensorProfile, SensorProfileUpdate, SensorProfileCreate);

#[utoipa::path(
    get,
    path = "/{id}",
    responses(
        (status = 200, description = "SensorProfile found", body = SensorProfile),
        (status = 404, description = "SensorProfile not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = Uuid, description = "SensorProfile ID"),
        ("start" = Option<String>, Query, description = "Start of date range (ISO 8601)"),
        ("end" = Option<String>, Query, description = "End of date range (ISO 8601)")
    ),
    summary = format!("Get one {}", SensorProfile::RESOURCE_NAME_SINGULAR),
    description = format!("Retrieves one {} by its ID.\n\n{}", SensorProfile::RESOURCE_NAME_SINGULAR, SensorProfile::RESOURCE_DESCRIPTION)
)]
pub async fn get_one(
    State(db): State<sea_orm::DatabaseConnection>,
    Path(id): Path<uuid::Uuid>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<SensorProfile>, (axum::http::StatusCode, axum::Json<String>)> {
    match SensorProfile::get_one_with_date_range(&db, id, query.start, query.end).await {
        Ok(item) => Ok(Json(item)),
        Err(DbErr::RecordNotFound(_)) => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json("Not Found".to_string()),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal Server Error".to_string()),
        )),
    }
}

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    SensorProfile: CRUDResource,
{
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one))
        .routes(routes!(get_all_handler))
        .routes(routes!(create_one_handler))
        .routes(routes!(update_one_handler))
        .routes(routes!(delete_one_handler))
        .routes(routes!(delete_many_handler))
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
