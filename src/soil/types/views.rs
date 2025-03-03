use crate::common::auth::Role;
use crate::soil::types::models::SoilType;
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
    SoilType: CRUDResource,
{
    let mut mutating_router = Router::new()
        .route(
            "/",
            get(crud::get_all::<SoilType>).post(crud::create_one::<SoilType>),
        )
        .route(
            "/{id}",
            get(crud::get_one::<SoilType>)
                .put(crud::update_one::<SoilType>)
                .delete(crud::delete_one::<SoilType>),
        )
        .route("/batch", delete(crud::delete_many::<SoilType>))
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
            SoilType::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema};
    use serde_json::{from_slice, json};
    use tower::ServiceExt;

    // A simplified version of your setup_database function for testing.
    async fn setup_database() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(db.get_database_backend());
        // Assuming you only need to create the soil_types table for this module.
        let stmt = schema
            .create_table_from_entity(crate::soil::types::db::Entity)
            .clone();
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();
        db
    }

    #[tokio::test]
    async fn test_get_all_soil_types() {
        let db = setup_database().await;
        // Initialize the router with the test DB
        let app = Router::new().nest("/api/soil_types", router(&db, None));

        // Create a new soil type via POST
        let soil_type = json!({
            "name": "Clay",
            "description": "Clay soil type",
            "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA"
        });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/soil_types")
                    .header("Content-Type", "application/json")
                    .body(Body::from(soil_type.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{response:?}");
        assert_eq!(response.status(), StatusCode::CREATED);

        // Retrieve all soil types via GET
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/soil_types")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: serde_json::Value = from_slice(&body_bytes).unwrap();
        assert!(body_json.is_array(), "Response body is not an array");

        let soil_types = body_json.as_array().unwrap();
        assert_eq!(
            soil_types.len(),
            1,
            "Expected only 1 soil type, found {}",
            soil_types.len()
        );

        // Verify the soil type details (ignoring the "image" field in the GET response)
        let soil_type_obj = soil_types[0].as_object().unwrap();
        println!("{soil_type_obj:?}");
        assert_eq!(soil_type_obj.get("name").unwrap(), "Clay");
        assert_eq!(soil_type_obj.get("description").unwrap(), "Clay soil type");
        assert_eq!(
            soil_type_obj.get("image"),
            Some(&serde_json::Value::Null),
            "Field 'image' should not be returned"
        );
    }
}
