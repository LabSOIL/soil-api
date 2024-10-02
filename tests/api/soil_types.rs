use crate::mock_fixtures::mock_api;
use axum::body::to_bytes;
use axum::body::Body;
use axum::routing::Router;
use hyper::{Request, StatusCode};
use rstest::*;
use serde_json::{from_slice, Value};
use tower::ServiceExt;

#[rstest]
#[tokio::test]
async fn test_get_all_soil_types(#[future(awt)] mock_api: Router) {
    let soil_type = serde_json::json!({
        "name": "Clay",
        "description": "Clay soil type",
        "image": "clay.png"
    });

    // Insert mock data into the API with the POST method.

    let response = mock_api
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/soil_types")
                .header("Content-Type", "application/json")
                .body(Body::from(soil_type.clone().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Get all from API
    let response = mock_api
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/soil_types")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = from_slice(&body_bytes).expect("Failed to parse response body as JSON");

    // Assert that the body is an array and contains expected mock data.
    assert!(body_json.is_array(), "Response body is not an array");
    let soil_type = body_json.as_array().unwrap();
    println!("{:?}", soil_type);

    // // Assert that at least two soil types are present in the response.
    assert!(
        soil_type.len() == 1,
        "Expected only 1 soil type, found {}",
        soil_type.len()
    );

    assert!(soil_type[0].is_object());

    // Check the soil type matches the expected mock data from above.
    let soil_type = soil_type[0].as_object().unwrap();
    assert_eq!(
        soil_type.get("name").unwrap(),
        &serde_json::Value::String("Clay".to_string())
    );
    assert_eq!(
        soil_type.get("description").unwrap(),
        &serde_json::Value::String("Clay soil type".to_string())
    );

    // Should not be in response on getall);
    assert_eq!(soil_type.get("image"), None);
}
