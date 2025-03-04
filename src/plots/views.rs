use super::models::Plot;
use crate::common::auth::Role;
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
// use utoipa;
// use uuid::Uuid;

// pub fn router(
//     db: &DatabaseConnection,
//     keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
// ) -> Router
// where
//     Plot: CRUDResource,
// {
//     let mut mutating_router = Router::new()
//         .route(
//             "/",
//             get(crud::get_all::<Plot>).post(crud::create_one::<Plot>),
//         )
//         .route(
//             "/{id}",
//             get(get_one_handler),
//             // get(crud::get_one::<Plot>), // .put(crud::update_one::<Plot>)
//             // .delete(crud::delete_one::<Plot>),
//         )
//         .route("/batch", delete(crud::delete_many::<Plot>))
//         .with_state(db.clone());

//     if let Some(instance) = keycloak_auth_instance {
//         mutating_router = mutating_router.layer(
//             KeycloakAuthLayer::<Role>::builder()
//                 .instance(instance)
//                 .passthrough_mode(PassthroughMode::Block)
//                 .persist_raw_claims(false)
//                 .expected_audiences(vec![String::from("account")])
//                 .required_roles(vec![Role::Administrator])
//                 .build(),
//         );
//     } else {
//         println!(
//             "Warning: Mutating routes of {} router are not protected",
//             Plot::RESOURCE_NAME_PLURAL
//         );
//     }

//     mutating_router
// }
use crudcrate::get_one;
get_one!(Plot, "plots");
use utoipa_axum::{PathItemExt, router::OpenApiRouter, routes};

pub fn router(
    db: &DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> OpenApiRouter
where
    Plot: CRUDResource,
{
    // let (mut mutating_router, api)
    let mut mutating_router = OpenApiRouter::new()
        .routes(routes!(get_one_handler))
        .with_state(db.clone());
    // .split_for_parts();

    // mutating_router;
    // let mut mutating_router = Router::new()
    //     .route(
    //         "/",
    //         get(crud::get_all::<Plot>).post(crud::create_one::<Plot>),
    //     )
    //     .route(
    //         "/{id}",
    //         get(get_one_handler),
    //         // get(crud::get_one::<Plot>), // .put(crud::update_one::<Plot>)
    //         // .delete(crud::delete_one::<Plot>),
    //     )
    //     .route("/batch", delete(crud::delete_many::<Plot>))
    //     .with_state(db.clone());

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
            Plot::RESOURCE_NAME_PLURAL
        );
    }

    mutating_router
}

// #[utoipa::path(
//     get,
//     path = "/plots",
//     params(ListQueryParams),
//     responses(
//         (status = 200, description = "List plots successfully", body = [Plot])
//     )
// )]
// pub async fn get_plots(
//     Query(params): Query<FilterOptions>,
//     State(db): State<DatabaseConnection>,
// ) -> Result<(HeaderMap, Json<Vec<Plot>>), (StatusCode, String)> {
//     // Calls the generic implementation specialized for Plot.
//     get_all::<Plot>(Query(params), State(db)).await
// }

// #[utoipa::path(
//     get,
//     path = "/plots/{id}",
//     responses(
//         (status = 200, description = "Get plot successfully", body = Plot),
//         (status = 404, description = "Plot not found")
//     ),
//     params(
//         ("id" = Uuid, Path, description = "Plot identifier")
//     )
// )]
// pub async fn get_plot(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
// ) -> Result<Json<Plot>, (StatusCode, Json<String>)> {
//     get_one::<Plot>(State(db), Path(id)).await
// }

// // Similarly, define wrappers for create, update, and delete endpoints.
// #[utoipa::path(
//     post,
//     path = "/plots",
//     request_body = PlotCreate,
//     responses(
//         (status = 201, description = "Plot created successfully", body = Plot),
//         (status = 409, description = "Duplicate entry")
//     )
// )]
// pub async fn create_plot(
//     State(db): State<DatabaseConnection>,
//     Json(payload): Json<PlotCreate>,
// ) -> Result<(StatusCode, Json<Plot>), (StatusCode, Json<String>)> {
//     create_one::<Plot>(State(db), Json(payload)).await
// }

// #[utoipa::path(
//     put,
//     path = "/plots/{id}",
//     request_body = PlotUpdate,
//     responses(
//         (status = 200, description = "Plot updated successfully", body = Plot),
//         (status = 404, description = "Plot not found")
//     ),
//     params(
//         ("id" = Uuid, Path, description = "Plot identifier")
//     )
// )]
// pub async fn update_plot(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
//     Json(payload): Json<PlotUpdate>,
// ) -> Result<Json<Plot>, (StatusCode, Json<String>)> {
//     update_one::<Plot>(State(db), Path(id), Json(payload)).await
// }

// #[utoipa::path(
//     delete,
//     path = "/plots/{id}",
//     responses(
//         (status = 204, description = "Plot deleted successfully", body = Uuid),
//         (status = 500, description = "Error deleting plot")
//     ),
//     params(
//         ("id" = Uuid, Path, description = "Plot identifier")
//     )
// )]
// pub async fn delete_plot(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
// ) -> Result<(StatusCode, Json<Uuid>), (StatusCode, Json<String>)> {
//     delete_one::<Plot>(State(db), Path(id)).await
// }
