use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub db_url: Option<String>,
    pub app_name: String,
    pub keycloak_ui_id: String,
    pub keycloak_url: String,
    pub keycloak_browser_url: String,
    pub keycloak_realm: String,
    pub deployment: String,
    pub srid: i32,
    pub public_cache_timeout_seconds: u64,
    pub disable_rate_limiting: bool,
    pub rate_limit_public_per_second: u64,
    pub rate_limit_public_burst: u32,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load from .env file if available
        let srid: i32 = 2056;
        let public_cache_timeout_seconds: u64 = 900; // 15 minutes

        let db_url = env::var("DB_URL").ok().or_else(|| {
            Some(format!(
                "{}://{}:{}@{}:{}/{}",
                env::var("DB_PREFIX").unwrap_or_else(|_| "postgresql".to_string()),
                env::var("DB_USER").expect("DB_USER must be set"),
                env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
                env::var("DB_HOST").expect("DB_HOST must be set"),
                env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()),
                env::var("DB_NAME").expect("DB_NAME must be set"),
            ))
        });

        Config {
            app_name: env::var("APP_NAME").expect("APP_NAME must be set"),
            keycloak_ui_id: env::var("KEYCLOAK_UI_ID").expect("KEYCLOAK_UI_ID must be set"),
            keycloak_url: env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set"),
            keycloak_browser_url: env::var("KEYCLOAK_BROWSER_URL")
                .unwrap_or_else(|_| env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set")),
            keycloak_realm: env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM must be set"),
            deployment: env::var("DEPLOYMENT")
                .expect("DEPLOYMENT must be set, this can be local, dev, stage, or prod"),
            db_url,
            srid,
            public_cache_timeout_seconds,
            disable_rate_limiting: env::var("DISABLE_RATE_LIMITING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            rate_limit_public_per_second: env::var("RATE_LIMIT_PUBLIC_PER_SECOND")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            rate_limit_public_burst: env::var("RATE_LIMIT_PUBLIC_BURST")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
        }
    }
}
