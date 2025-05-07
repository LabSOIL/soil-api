use super::models::{SoilProfile, SoilProfileCreate, SoilProfileUpdate};
use crate::common::auth::Role;
use axum_keycloak_auth::{
    PassthroughMode, instance::KeycloakAuthInstance, layer::KeycloakAuthLayer,
};
use crudcrate::{CRUDResource, crud_handlers};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use utoipa_axum::{router::OpenApiRouter, routes};

crud_handlers!(SoilProfile, SoilProfileUpdate, SoilProfileCreate);

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    SoilProfile: CRUDResource,
{
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one_handler))
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
            SoilProfile::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema};
    use serde_json::{Value, from_slice, json};
    use tower::ServiceExt;

    // Setup a fresh in-memory SQLite database for each test.
    async fn setup_database() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(db.get_database_backend());
        // Create the soilprofile table.
        let soilprofile_stmt = schema
            .create_table_from_entity(crate::routes::private::soil::profiles::db::Entity)
            .clone();
        db.execute(db.get_database_backend().build(&soilprofile_stmt))
            .await
            .unwrap();

        let project_stmt = schema
            .create_table_from_entity(crate::routes::private::projects::db::Entity)
            .clone();
        db.execute(db.get_database_backend().build(&project_stmt))
            .await
            .unwrap();

        let area_stmt = schema
            .create_table_from_entity(crate::routes::private::areas::db::Entity)
            .clone();
        db.execute(db.get_database_backend().build(&area_stmt))
            .await
            .unwrap();
        // Also create the soiltype table for the FK constraint.
        let soiltype_stmt = schema
            .create_table_from_entity(crate::routes::private::soil::types::db::Entity)
            .clone();
        db.execute(db.get_database_backend().build(&soiltype_stmt))
            .await
            .unwrap();
        db
    }

    async fn create_dummy_entities(
        db: &DatabaseConnection,
    ) -> (uuid::Uuid, uuid::Uuid, uuid::Uuid) {
        use sea_orm::Set;
        // Insert dummy soil type.
        let dummy_soil_type_id = uuid::Uuid::new_v4();
        let dummy_soiltype = crate::routes::private::soil::types::db::ActiveModel {
            id: Set(dummy_soil_type_id),
            name: Set("Dummy".to_owned()),
            description: Set("Dummy soil type".to_owned()),
            last_updated: Set(Utc::now()),
            image: Set(None),
        };
        crate::routes::private::soil::types::db::Entity::insert(dummy_soiltype)
            .exec(db)
            .await
            .unwrap();

        // Insert dummy project.
        let dummy_project_id = uuid::Uuid::new_v4();
        let dummy_project = crate::routes::private::projects::db::ActiveModel {
            id: Set(dummy_project_id),
            name: Set("Dummy Project".to_owned()),
            description: Set(Some("Dummy project description".to_owned())),
            color: Set("#0000FF".to_owned()),
            last_updated: Set(Utc::now()),
        };
        crate::routes::private::projects::db::Entity::insert(dummy_project)
            .exec(db)
            .await
            .unwrap();

        // Insert dummy area.
        let dummy_area_id = uuid::Uuid::new_v4();
        let dummy_area = crate::routes::private::areas::db::ActiveModel {
            id: Set(dummy_area_id),
            project_id: Set(dummy_project_id),
            name: Set("Dummy Area".to_owned()),
            description: Set(Some("Dummy area description".to_owned())),
            last_updated: Set(Utc::now()),
            is_public: Set(false),
        };
        crate::routes::private::areas::db::Entity::insert(dummy_area)
            .exec(db)
            .await
            .unwrap();

        (dummy_soil_type_id, dummy_project_id, dummy_area_id)
    }

    // Convenience function to build the router.
    fn build_router(db: &DatabaseConnection) -> axum::Router {
        // Here, we assume that your profiles router is defined in this module.
        // Passing None for the Keycloak auth instance.
        let (router, _api) = router(db, None).split_for_parts();
        router
    }

    #[tokio::test]
    async fn test_create_and_get_soil_profile() {
        let db = setup_database().await;
        let app = axum::Router::new().nest("/api/soil_profiles", build_router(&db));

        // Insert a dummy soil type (required for FK constraint).
        let (dummy_soil_type_id, _dummy_project_id, dummy_area_id) =
            create_dummy_entities(&db).await;

        // Create a new soil profile via POST.
        let create_payload = json!({
            "name": "BF01",
            "gradient": "flat",
            "soil_type_id": dummy_soil_type_id.to_string(),
            "area_id": dummy_area_id.to_string(),
            // Optional fields are set to null.
            "description_horizon": null,
            "weather": null,
            "topography": null,
            "vegetation_type": null,
            "aspect": null,
            "lythology_surficial_deposit": null,
            "created_on": null,
            "soil_diagram": null,
            "photo": null,
            "parent_material": null,
            "coord_srid": 4326,
            "coord_x": 10.0,
            "coord_y": 20.0,
            "coord_z": 0.0
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/soil_profiles")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{:?}", response.body());
        assert_eq!(response.status(), StatusCode::CREATED);

        // Retrieve all soil profiles via GET.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/soil_profiles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = from_slice(&body_bytes).unwrap();
        assert!(body_json.is_array(), "Response body is not an array");
        let profiles = body_json.as_array().unwrap();
        assert_eq!(
            profiles.len(),
            1,
            "Expected one profile, got {}",
            profiles.len()
        );

        let profile_obj = profiles[0].as_object().unwrap();
        assert_eq!(profile_obj.get("name").unwrap(), "BF01");
        assert_eq!(profile_obj.get("gradient").unwrap(), "flat");
    }

    #[tokio::test]
    async fn test_update_soil_profile() {
        let db = setup_database().await;
        let app = axum::Router::new().nest("/api/soil_profiles", build_router(&db));

        // Insert dummy data
        let (dummy_soil_type_id, _dummy_project_id, dummy_area_id) =
            create_dummy_entities(&db).await;

        // Create a soil profile via POST.
        let create_payload = json!({
            "name": "BF01",
            "gradient": "flat",
            "soil_type_id": dummy_soil_type_id.to_string(),
            "area_id": dummy_area_id.to_string(),
            "description_horizon": null,
            "weather": "sunny",
            "topography": "hilly",
            "vegetation_type": "forest",
            "aspect": "north",
            "lythology_surficial_deposit": null,
            "created_on": null,
            "soil_diagram": null,
            "photo": null,
            "parent_material": null,
            "coord_srid": 4326,
            "coord_x": 10.0,
            "coord_y": 20.0,
            "coord_z": 0.0
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/soil_profiles")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{:?}", response.body());
        assert_eq!(response.status(), StatusCode::CREATED);

        // Get the created profile to extract its ID.
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/soil_profiles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = from_slice(&body_bytes).unwrap();
        let profiles = body_json.as_array().unwrap();
        assert_eq!(profiles.len(), 1);
        let profile = profiles[0].as_object().unwrap();
        let profile_id: String = profile.get("id").unwrap().as_str().unwrap().into();

        // Update the soil profile.
        let update_payload = json!({
            "name": "BF01 Updated",
            "gradient": "sloped",
            "weather": "rainy",
            "topography": "flat",
            "vegetation_type": "grassland",
            "aspect": "south"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/soil_profiles/{profile_id}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(update_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Retrieve soil profiles and verify updates.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/soil_profiles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = from_slice(&body_bytes).unwrap();
        let profiles = body_json.as_array().unwrap();
        assert_eq!(profiles.len(), 1);
        let profile_obj = profiles[0].as_object().unwrap();
        assert_eq!(profile_obj.get("name").unwrap(), "BF01 Updated");
        assert_eq!(profile_obj.get("gradient").unwrap(), "sloped");
        assert_eq!(profile_obj.get("weather").unwrap(), "rainy");
        assert_eq!(profile_obj.get("topography").unwrap(), "flat");
        assert_eq!(profile_obj.get("vegetation_type").unwrap(), "grassland");
        assert_eq!(profile_obj.get("aspect").unwrap(), "south");
    }

    #[tokio::test]
    async fn test_delete_soil_profile() {
        let db = setup_database().await;
        let app = axum::Router::new().nest("/api/soil_profiles", build_router(&db));

        // Insert a dummy soil type.
        let (dummy_soil_type_id, _dummy_project_id, dummy_area_id) =
            create_dummy_entities(&db).await;

        // Create a soil profile via POST.
        let create_payload = json!({
            "name": "BF01",
            "gradient": "flat",
            "soil_type_id": dummy_soil_type_id.to_string(),
            "area_id": dummy_area_id.to_string(),
            "description_horizon": null,
            "weather": null,
            "topography": null,
            "vegetation_type": null,
            "aspect": null,
            "lythology_surficial_deposit": null,
            "created_on": null,
            "soil_diagram": null,
            "photo": null,
            "parent_material": null,
            "coord_srid": 4326,
            "coord_x": 10.0,
            "coord_y": 20.0,
            "coord_z": 0.0
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/soil_profiles")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{:?}", response.body());
        assert_eq!(response.status(), StatusCode::CREATED,);

        // Retrieve the created profile to get its ID.
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/soil_profiles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = from_slice(&body_bytes).unwrap();
        let profiles = body_json.as_array().unwrap();
        assert_eq!(profiles.len(), 1);
        let profile = profiles[0].as_object().unwrap();
        let profile_id: String = profile.get("id").unwrap().as_str().unwrap().into();

        // Delete the soil profile.
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/soil_profiles/{profile_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify that the profile has been deleted.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/soil_profiles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = from_slice(&body_bytes).unwrap();
        let profiles = body_json.as_array().unwrap();
        assert_eq!(profiles.len(), 0, "Expected no profiles after deletion");
    }
}
