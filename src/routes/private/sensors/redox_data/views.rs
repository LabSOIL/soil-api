use super::models::{RedoxData, RedoxDataCreate, RedoxDataUpdate};
use crate::common::auth::Role;
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use crudcrate::{CRUDResource, crud_handlers};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

crud_handlers!(RedoxData, RedoxDataUpdate, RedoxDataCreate);

/// A single error from a batch ingest operation.
#[derive(Serialize, ToSchema)]
pub struct BatchIngestError {
    pub index: usize,
    pub message: String,
}

/// Result of a batch ingest operation.
#[derive(Serialize, ToSchema)]
pub struct BatchIngestResult {
    pub inserted: usize,
    pub errors: Vec<BatchIngestError>,
}

#[utoipa::path(
    post,
    path = "/batch",
    request_body = Vec<RedoxDataCreate>,
    responses(
        (status = 200, description = "Batch create results.", body = BatchIngestResult),
    ),
    summary = "Batch create redox data records",
    description = "Accepts an array of redox data records. Each entry is inserted independently; errors are recorded per-entry without aborting the batch.",
    operation_id = "create_redox_data_batch",
)]
pub async fn create_redox_data_batch(
    axum::extract::State(db): axum::extract::State<DatabaseConnection>,
    axum::Json(requests): axum::Json<Vec<RedoxDataCreate>>,
) -> axum::Json<BatchIngestResult> {
    let mut inserted = 0usize;
    let mut errors = Vec::new();

    for (index, create_data) in requests.into_iter().enumerate() {
        let active_model = super::db::ActiveModel {
            id: ActiveValue::Set(uuid::Uuid::new_v4()),
            sensorprofile_id: ActiveValue::Set(create_data.sensorprofile_id),
            measured_on: ActiveValue::Set(create_data.measured_on),
            ch1_5cm_mv: ActiveValue::Set(create_data.ch1_5cm_mv),
            ch2_15cm_mv: ActiveValue::Set(create_data.ch2_15cm_mv),
            ch3_25cm_mv: ActiveValue::Set(create_data.ch3_25cm_mv),
            ch4_35cm_mv: ActiveValue::Set(create_data.ch4_35cm_mv),
            temp_c: ActiveValue::Set(create_data.temp_c),
        };
        match active_model.insert(&db).await {
            Ok(_) => inserted += 1,
            Err(e) => errors.push(BatchIngestError {
                index,
                message: format!("Failed to insert: {e}"),
            }),
        }
    }

    axum::Json(BatchIngestResult { inserted, errors })
}

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    RedoxData: CRUDResource,
{
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one_handler))
        .routes(routes!(get_all_handler))
        .routes(routes!(create_one_handler))
        .routes(routes!(update_one_handler))
        .routes(routes!(delete_one_handler))
        .routes(routes!(delete_many_handler))
        .routes(routes!(create_redox_data_batch))
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
            RedoxData::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}
