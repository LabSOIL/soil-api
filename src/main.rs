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

    // Refresh continuous aggregates with bounded windows (can't run inside migration transactions).
    // Historical backfill is handled by dbctl.sh restore and the initial full refresh after migration.
    // Hourly must refresh before 6h since 6h is a hierarchical aggregate on top of hourly.
    for (view, interval) in [
        ("sensordata_hourly", "3 hours"),
        ("sensordata_6h", "18 hours"),
    ] {
        let sql = format!(
            "CALL refresh_continuous_aggregate('{view}', now() - interval '{interval}', now())"
        );
        match db.execute(Statement::from_string(db.get_database_backend(), sql)).await {
            Ok(_) => println!("Refreshed {view} (last {interval})"),
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
