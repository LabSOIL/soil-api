use super::db::Model;
use async_trait::async_trait;
use base64;
use base64::Engine;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use gpx::read;
use sea_orm::{
    entity::prelude::*,
    ActiveModelTrait,
    ActiveValue::{self, NotSet},
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
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
pub struct GNSS {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(
        update_model = false,
        create_model = false,
        on_update = Utc::now(),
        on_create = Utc::now()
    )]
    pub last_updated: DateTime<Utc>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub time: Option<DateTime<Utc>>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub name: Option<String>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub comment: Option<String>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub original_filename: Option<String>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub elevation_gps: Option<f64>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub latitude: Option<f64>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub longitude: Option<f64>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    // #[crudcrate(update_model = false, create_model = false)]
    pub coord_srid: Option<i32>,
    #[crudcrate(non_db_attr = true)]
    pub data_base64: Option<String>,
    #[crudcrate(non_db_attr = true)]
    pub filename: Option<String>,
}

impl From<Model> for GNSS {
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
impl CRUDResource for GNSS {
    type EntityType = crate::gnss::db::Entity;
    type ColumnType = crate::gnss::db::Column;
    type ModelType = crate::gnss::db::Model;
    type ActiveModelType = crate::gnss::db::ActiveModel;
    type ApiModel = GNSS;
    type CreateModel = GNSSCreate;
    type UpdateModel = GNSSUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "gnss";
    const RESOURCE_NAME_PLURAL: &'static str = "gnss";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        let models = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        Ok(models.into_iter().map(GNSS::from).collect())
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find()
            .filter(Self::ColumnType::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))?;
        Ok(GNSS::from(model))
    }
    // async fn create(
    //     db: &DatabaseConnection,
    //     create_model: Self::CreateModel,
    // ) -> Result<Self::ApiModel, DbErr> {
    //     let active_model: Self::ActiveModelType = create_model.into();
    //     let result = Self::EntityType::insert(active_model).exec(db).await?;
    //     match Self::get_one(db, result.last_insert_id.into()).await {
    //         Ok(obj) => Ok(obj),
    //         Err(_) => Err(DbErr::RecordNotFound(
    //             format!("{} not created", Self::RESOURCE_NAME_SINGULAR).into(),
    //         )),
    //     }
    // }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let gnss = GNSSCreateFromFile {
            data_base64: create_model.data_base64.unwrap(),
            filename: create_model.filename.unwrap(),
        };
        let creates = gnss.into_gnss_creates(4326).await.unwrap();

        let mut response_objs = Vec::new();
        for create in creates {
            let mut active_model: Self::ActiveModelType = create.into();

            // Delete coord_x and coord_y from the active model
            active_model.coord_x = NotSet;
            active_model.coord_y = NotSet;
            active_model.coord_srid = NotSet;

            println!("Creates: {:?}", active_model);

            let response_obj = active_model.insert(db).await?;
            response_objs.push(response_obj);
        }
        let obj = Self::get_one(&db, response_objs[0].id).await?;
        Ok(obj)
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))?
            .into();
        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let obj = Self::get_one(&db, response_obj.id).await?;
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
    /// Converts the uploaded GPX file (in base64) into a list of GNSSCreate models.
    /// The `srid` parameter defines the target spatial reference (e.g. from your config).
    pub async fn into_gnss_creates(
        self,
        srid: i32,
    ) -> Result<Vec<GNSSCreate>, Box<dyn std::error::Error>> {
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

        // Prepare transformation from EPSG:4326 (lat/lon) to the target SRID
        // let to_proj = Proj::new(&format!("EPSG:{}", srid))?;
        // println!("GPX: {:?}", gpx.metadata);
        let mut creates = Vec::new();
        for wpt in gpx.waypoints {
            // println!("WPT: {:?}", wpt);
            // println!("Time: {:?}", wpt.time);
            // let latitude = wpt.point().y();
            // let longitude = wpt.point().x();
            // let elevation = wpt.elevation;
            // Time is structured as: 2023-07-20T09:32:34.000000000Z convert to Date
            let time: DateTime<Utc> =
                DateTime::parse_from_rfc3339(&wpt.time.unwrap().format().unwrap())?.into();
            // let name = wpt.name;
            // let comment = wpt.comment;
            // (Any additional GPX fields you care about can be extracted here)
            // println!("Time: {}, Name: {:?}, Comment: {:?}", time, name, comment);
            // Transform longitude, latitude to x, y using the provided SRID
            // let (x, y) = to_proj.convert((longitude, latitude))?;

            creates.push(GNSSCreate {
                latitude: Some(wpt.point().y()),
                longitude: Some(wpt.point().x()),
                elevation_gps: wpt.elevation,
                time: Some(time),
                name: wpt.name,
                comment: wpt.comment,
                original_filename: Some(self.filename.clone()),
                coord_x: None,
                coord_y: None,
                coord_srid: Some(srid),
                data_base64: None,
                filename: None,
            });
        }
        Ok(creates)
    }
}
