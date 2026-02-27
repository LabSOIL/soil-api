/// Validate that a website slug contains only safe characters.
pub fn validate_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 100
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

use crate::routes::private::area_websites::db as AreaWebsiteDB;
use crate::routes::private::sensors::profile::db as ProfileDB;
use crate::routes::private::website_plot_exclusions::db as PlotExclusionDB;
use crate::routes::private::website_sensor_exclusions::db as SensorExclusionDB;
use crate::routes::private::websites::db as WebsiteDB;
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Statement};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct WebsiteAccess {
    pub website_id: Uuid,
    /// area_id → (date_from, date_to) — None means no restriction
    pub area_dates: HashMap<Uuid, (Option<DateTime<Utc>>, Option<DateTime<Utc>>)>,
    /// Set of area_ids assigned to this website
    pub area_ids: HashSet<Uuid>,
    /// Set of plot_ids excluded from this website
    pub excluded_plot_ids: HashSet<Uuid>,
    /// Set of sensorprofile_ids excluded from this website
    pub excluded_sensor_ids: HashSet<Uuid>,
}

/// Resolve a website slug into access control data.
/// Returns None if slug doesn't match any website.
pub async fn resolve_website_access(
    db: &DatabaseConnection,
    slug: &str,
) -> Result<Option<WebsiteAccess>, sea_orm::DbErr> {
    // 1. Lookup website by slug
    let website = WebsiteDB::Entity::find()
        .filter(WebsiteDB::Column::Slug.eq(slug))
        .one(db)
        .await?;

    let website = match website {
        Some(w) => w,
        None => return Ok(None),
    };

    // 2. Fetch all area_website rows for this website
    let area_websites = AreaWebsiteDB::Entity::find()
        .filter(AreaWebsiteDB::Column::WebsiteId.eq(website.id))
        .all(db)
        .await?;

    let mut area_ids = HashSet::new();
    let mut area_dates = HashMap::new();
    for aw in &area_websites {
        area_ids.insert(aw.area_id);
        area_dates.insert(aw.area_id, (aw.date_from, aw.date_to));
    }

    // 3. Fetch all plot exclusions for this website
    let plot_exclusions = PlotExclusionDB::Entity::find()
        .filter(PlotExclusionDB::Column::WebsiteId.eq(website.id))
        .all(db)
        .await?;
    let excluded_plot_ids: HashSet<Uuid> = plot_exclusions.iter().map(|e| e.plot_id).collect();

    // 4. Fetch all sensor exclusions for this website
    let sensor_exclusions = SensorExclusionDB::Entity::find()
        .filter(SensorExclusionDB::Column::WebsiteId.eq(website.id))
        .all(db)
        .await?;
    let excluded_sensor_ids: HashSet<Uuid> =
        sensor_exclusions.iter().map(|e| e.sensorprofile_id).collect();

    Ok(Some(WebsiteAccess {
        website_id: website.id,
        area_ids,
        area_dates,
        excluded_plot_ids,
        excluded_sensor_ids,
    }))
}

/// Check if a sensor is accessible on a website.
/// Returns Some((profile, date_from, date_to)) if accessible, None if blocked.
/// Combines website lookup, area_website check, and exclusion check into a single query.
pub async fn check_sensor_access(
    db: &DatabaseConnection,
    sensor_id: Uuid,
    slug: &str,
) -> Result<Option<(ProfileDB::Model, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>, sea_orm::DbErr>
{
    // Single query: website + area_website join + exclusion check
    let sql = r#"
        SELECT aw.date_from, aw.date_to
        FROM website w
        JOIN sensorprofile sp ON sp.id = $1
        JOIN area_website aw ON aw.area_id = sp.area_id AND aw.website_id = w.id
        LEFT JOIN website_sensor_exclusion wse
            ON wse.website_id = w.id AND wse.sensorprofile_id = sp.id
        WHERE w.slug = $2 AND wse.id IS NULL
    "#;
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        vec![sensor_id.into(), slug.into()],
    );

    let row = match db.query_one(stmt).await? {
        Some(r) => r,
        None => return Ok(None),
    };

    let date_from: Option<DateTime<Utc>> = row.try_get("", "date_from").ok().flatten();
    let date_to: Option<DateTime<Utc>> = row.try_get("", "date_to").ok().flatten();

    // Profile must exist since the JOIN with sensorprofile succeeded
    let profile = match ProfileDB::Entity::find_by_id(sensor_id).one(db).await? {
        Some(p) => p,
        None => return Ok(None),
    };

    Ok(Some((profile, date_from, date_to)))
}
