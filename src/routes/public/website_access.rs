use crate::routes::private::area_websites::db as AreaWebsiteDB;
use crate::routes::private::sensors::profile::db as ProfileDB;
use crate::routes::private::website_plot_exclusions::db as PlotExclusionDB;
use crate::routes::private::website_sensor_exclusions::db as SensorExclusionDB;
use crate::routes::private::websites::db as WebsiteDB;
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
/// Returns Some((date_from, date_to)) if accessible, None if blocked.
pub async fn check_sensor_access(
    db: &DatabaseConnection,
    sensor_id: Uuid,
    slug: &str,
) -> Result<Option<(Option<DateTime<Utc>>, Option<DateTime<Utc>>)>, sea_orm::DbErr> {
    // 1. Lookup website by slug
    let website = WebsiteDB::Entity::find()
        .filter(WebsiteDB::Column::Slug.eq(slug))
        .one(db)
        .await?;

    let website = match website {
        Some(w) => w,
        None => return Ok(None),
    };

    // 2. Lookup sensor profile to get area_id
    let profile = ProfileDB::Entity::find_by_id(sensor_id).one(db).await?;

    let profile = match profile {
        Some(p) => p,
        None => return Ok(None),
    };

    // 3. Check if area_id has an area_website entry for this website
    let area_website = AreaWebsiteDB::Entity::find()
        .filter(AreaWebsiteDB::Column::AreaId.eq(profile.area_id))
        .filter(AreaWebsiteDB::Column::WebsiteId.eq(website.id))
        .one(db)
        .await?;

    let area_website = match area_website {
        Some(aw) => aw,
        None => return Ok(None),
    };

    // 4. Check if sensor is in the exclusion list
    let exclusion = SensorExclusionDB::Entity::find()
        .filter(SensorExclusionDB::Column::WebsiteId.eq(website.id))
        .filter(SensorExclusionDB::Column::SensorprofileId.eq(sensor_id))
        .one(db)
        .await?;

    if exclusion.is_some() {
        return Ok(None); // Sensor is excluded
    }

    // 5. Return the date restrictions
    Ok(Some((area_website.date_from, area_website.date_to)))
}
