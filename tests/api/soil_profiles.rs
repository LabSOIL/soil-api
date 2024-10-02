use crate::mock_fixtures::mock_api;
use axum::body::{to_bytes, Body};
use axum::routing::Router;
use hyper::{Request, StatusCode};
use rstest::*;
use serde_json::{from_slice, Value};
use std::collections::HashMap;
use tower::ServiceExt;

#[rstest]
#[tokio::test]
async fn test_soil_profiles_parity(#[future(awt)] mock_api: Router) {
    // Expected soil profile data (similar to what you get from the Python API)
    let expected_soil_profiles = serde_json::json!([
        {
            "name": "BF01",
            "profile_iterator": 1,
            "gradient": "flat",
        },
        {
            "name": "BF06",
            "profile_iterator": 6,
            "gradient": "flat",
        }
    ]);

    // Insert the mock data into the API via a POST request
    for soil_profile in expected_soil_profiles.as_array().unwrap() {
        let response = mock_api
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/soil_profiles")
                    .header("Content-Type", "application/json")
                    .body(Body::from(soil_profile.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "Failed to create soil profile"
        );
    }

    // Retrieve all soil profiles from the API via a GET request
    let response = mock_api
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/soil_profiles")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Failed to fetch soil profiles"
    );

    // Parse the response body as JSON
    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = from_slice(&body_bytes).expect("Failed to parse response body as JSON");

    // Assert that the response body is an array
    assert!(body_json.is_array(), "Response body is not an array");
    let soil_profiles_array = body_json.as_array().unwrap();

    // Create HashMaps for better comparison
    let expected_profiles_map: HashMap<_, _> = expected_soil_profiles
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["id"].as_str().unwrap(), p))
        .collect();

    let actual_profiles_map: HashMap<_, _> = soil_profiles_array
        .iter()
        .map(|p| (p["id"].as_str().unwrap(), p))
        .collect();

    // Assert that the number of profiles is the same
    assert_eq!(
        actual_profiles_map.len(),
        expected_profiles_map.len(),
        "Mismatch in the number of soil profiles"
    );

    // Compare each profile in detail
    for (id, expected_profile) in expected_profiles_map {
        let actual_profile = actual_profiles_map
            .get(id)
            .expect("Profile ID not found in response");

        // Perform a deep comparison of the profiles
        assert_eq!(
            actual_profile, &expected_profile,
            "Mismatch in profile data for ID: {}",
            id
        );
    }
}
