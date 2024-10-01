use axum::body::to_bytes;
use axum::body::Body;
use axum::routing::Router;
use hyper::{Request, StatusCode};
use rstest::*;
use tower::ServiceExt;
mod mock_fixtures;
use mock_fixtures::mock_api;
use serde_json::{from_slice, Value};
use uuid::Uuid;

#[rstest]
#[tokio::test]
async fn test_get_all_soil_types(#[future(awt)] mock_api: Router) {
    let now = chrono::Utc::now().naive_utc();

    let soil_type_1 = serde_json::json!({
        "id": Uuid::new_v4(),
        "iterator": 1,
        "last_updated": now,
        "name": "Clay",
        "description": "Clay soil type",
        "image": "clay.png"
    });

    let soil_type_2 = serde_json::json!({
        "id": Uuid::new_v4(),
        "iterator": 2,
        "last_updated": now,
        "name": "Sand",
        "description": "Sandy soil type",
        "image": null
    });

    // Add objects to the POST endpoint
    let response = mock_api
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = from_slice(&body_bytes).expect("Failed to parse response body as JSON");

    // Assert that the body is an array and contains expected mock data.
    assert!(body_json.is_array(), "Response body is not an array");
    let soil_types = body_json.as_array().unwrap();

    // Assert that at least two soil types are present in the response.
    assert!(soil_types.len() >= 2, "Expected at least 2 soil types");

    // Check each soil type in the array, by looping through them and checking their fields.
    for soil_type in soil_types {
        assert!(soil_type.is_object());
        let soil_type = soil_type.as_object().unwrap();

        // Check that all necessary fields exist in the soil type.
        assert!(soil_type.get("id").is_some());
        // assert!(soil_type.get("iterator").is_some());
        assert!(soil_type.get("last_updated").is_some());
        assert!(soil_type.get("name").is_some());
        assert!(soil_type.get("description").is_some());
        // assert!(soil_type.get("image").is_some());
    }

    let clay_soil = &soil_types[0];
    let sand_soil = &soil_types[1];

    assert_eq!(clay_soil["name"], "Clay");
    assert_eq!(clay_soil["description"], "Clay soil type");
    assert_eq!(sand_soil["name"], "Sand");
    assert_eq!(sand_soil["description"], "Sandy soil type");

    // Check that all necessary fields exist in the first soil type.
    assert!(clay_soil.get("id").is_some());
    // assert!(clay_soil.get("iterator").is_some());
    assert!(clay_soil.get("last_updated").is_some());
    assert!(clay_soil.get("name").is_some());
    assert!(clay_soil.get("description").is_some());
    // assert!(clay_soil.get("image").is_some());
}
