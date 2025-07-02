use super::db::Model;
use crate::config::Config;
use async_trait::async_trait;
use base64;
use base64::Engine;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use gpx::read;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{self, NotSet},
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
    entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use utoipa::ToSchema;
use uuid::Uuid;

// The creation and update of a GNSS record is just the file, but we unpack
// all of the values, so include these data_base74/filename fields in the model,
// and remove the other fields that are not needed for creation or update.
#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel, Debug)]
#[active_model = "super::db::ActiveModel"]
pub struct Gnss {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(
        update_model = false,
        create_model = false,
        on_update = Utc::now(),
        on_create = Utc::now()
    )]
    pub last_updated: DateTime<Utc>,
    pub time: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub original_filename: Option<String>,
    pub elevation_gps: Option<f64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    pub coord_srid: Option<i32>,
    #[crudcrate(non_db_attr = true)]
    pub data_base64: Option<String>,
    #[crudcrate(non_db_attr = true)]
    pub filename: Option<String>,
}

impl From<Model> for Gnss {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            last_updated: model.last_updated,
            name: model.name,
            time: model.time,
            comment: model.comment,
            original_filename: model.original_filename,
            elevation_gps: model.elevation_gps,
            latitude: model.latitude,
            longitude: model.longitude,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_srid: model.coord_srid,
            data_base64: None,
            filename: None,
        }
    }
}

#[async_trait]
impl CRUDResource for Gnss {
    type EntityType = crate::routes::private::gnss::db::Entity;
    type ColumnType = crate::routes::private::gnss::db::Column;
    type ActiveModelType = crate::routes::private::gnss::db::ActiveModel;
    type CreateModel = GnssCreate;
    type UpdateModel = GnssUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "GNSS recording";
    const RESOURCE_NAME_PLURAL: &'static str = "GNSS recordings";
    const RESOURCE_DESCRIPTION: &'static str = "GNSS recordings taken from the field are stored here to help propagate to other resources (soil profiles, plots, sensor profiles, etc.)";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self>, DbErr> {
        let models = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        Ok(models.into_iter().map(Gnss::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "{} not found",
                Self::RESOURCE_NAME_SINGULAR
            )))?;
        Ok(Gnss::from(model))
    }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self, DbErr> {
        let gnss = GNSSCreateFromFile {
            data_base64: create_model.data_base64.unwrap(),
            filename: create_model.filename.unwrap(),
        };
        let creates = gnss.into_gnss_creates().unwrap();

        let mut response_objs = Vec::new();
        for create in creates {
            let mut active_model: Self::ActiveModelType = create.into();

            // Delete coord_x and coord_y from the active model
            active_model.coord_x = NotSet;
            active_model.coord_y = NotSet;
            active_model.coord_srid = NotSet;

            let response_obj = active_model.insert(db).await?;
            response_objs.push(response_obj);
        }
        let obj = Self::get_one(db, response_objs[0].id).await?;
        Ok(obj)
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self, DbErr> {
        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "{} not found",
                Self::RESOURCE_NAME_SINGULAR
            )))?
            .into();
        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let obj = Self::get_one(db, response_obj.id).await?;
        Ok(obj)
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("last_updated", Self::ColumnType::LastUpdated),
            ("coord_x", Self::ColumnType::CoordX),
            ("coord_y", Self::ColumnType::CoordY),
            ("coord_srid", Self::ColumnType::CoordSrid),
            ("elevation_gps", Self::ColumnType::ElevationGps),
            ("latitude", Self::ColumnType::Latitude),
            ("longitude", Self::ColumnType::Longitude),
            ("original_filename", Self::ColumnType::OriginalFilename),
            ("time", Self::ColumnType::Time),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", Self::ColumnType::Name),
            ("original_filename", Self::ColumnType::OriginalFilename),
            ("comment", Self::ColumnType::Comment),
        ]
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct GNSSCreateFromFile {
    pub data_base64: String,
    pub filename: String,
}

impl GNSSCreateFromFile {
    /// Converts the uploaded GPX file (in base64) into a list of `GNSSCreate` models.
    /// The `srid` parameter defines the target spatial reference (e.g. from your config).
    pub fn into_gnss_creates(self) -> Result<Vec<GnssCreate>, Box<dyn std::error::Error>> {
        // Remove known base64 prefix if present
        let base64_prefix = "data:application/gpx+xml;base64,";
        let encoded = if self.data_base64.starts_with(base64_prefix) {
            &self.data_base64[base64_prefix.len()..]
        } else {
            &self.data_base64
        };
        let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(encoded)?;
        let gpx_data = String::from_utf8(decoded_bytes)?;

        // Parse the GPX file
        let cursor = Cursor::new(gpx_data.as_bytes());
        let gpx = read(cursor)?;

        let mut creates = Vec::new();
        for wpt in gpx.waypoints {
            // Time is structured as: 2023-07-20T09:32:34.000000000Z convert to Date
            let time: DateTime<Utc> =
                DateTime::parse_from_rfc3339(&wpt.time.unwrap().format().unwrap())?.into();

            creates.push(GnssCreate {
                latitude: Some(wpt.point().y()),
                longitude: Some(wpt.point().x()),
                elevation_gps: wpt.elevation,
                time: Some(time),
                name: wpt.name,
                comment: wpt.comment,
                original_filename: Some(self.filename.clone()),
                coord_x: None,
                coord_y: None,
                data_base64: None,
                filename: None,
            });
        }
        Ok(creates)
    }
}
