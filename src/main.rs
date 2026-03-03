mod common;
mod config;
mod routes;
mod services;

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

#[tokio::main]
async fn main() {
    // Set up tracing/logging
    tracing_subscriber::fmt::init();
    println!("Starting server...");

    // Load configuration
    let config: config::Config = config::Config::from_env();

    let db: DatabaseConnection = Database::connect(config.db_url.as_ref().unwrap())
        .await
        .unwrap();

    if db.ping().await.is_ok() {
        println!("Connected to the database");
    } else {
        println!("Could not connect to the database");
    }

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    // Left commented here in case of need to downgrade
    // Migrator::down(&db, Some(1))  // Downgrade one migration step
    //     .await
    //     .expect("Failed to run downgrade migration");

    println!("DB migrations complete");

    // Refresh continuous aggregates over the full time range on startup.
    // Hourly must run before 6h since 6h is a hierarchical aggregate built on top of hourly.
    for view in ["sensordata_hourly", "sensordata_6h"] {
        let sql = format!("CALL refresh_continuous_aggregate('{view}', NULL, NULL)");
        match db.execute(Statement::from_string(db.get_database_backend(), sql)).await {
            Ok(_) => println!("Refreshed {view} (full range)"),
            Err(e) => println!("Could not refresh {view}: {e}"),
        }
    }

    // Recompute sensor profile averages (safety net for any data ingested outside the API)
    {
        let sql = r"DO $$ DECLARE r RECORD;
        BEGIN
          FOR r IN SELECT id FROM sensorprofile LOOP
            PERFORM recompute_sensor_averages(r.id);
          END LOOP;
        END $$;";
        match db.execute(Statement::from_string(db.get_database_backend(), sql.to_string())).await {
            Ok(_) => println!("Recomputed sensor profile averages"),
            Err(e) => println!("Could not recompute sensor profile averages: {e}"),
        }
    }

    println!(
        "Starting server {} ({} deployment) ...",
        config.app_name,
        config.deployment.to_uppercase()
    );

    let addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("Listening on {addr}");

    let router = routes::build_router(&db);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
