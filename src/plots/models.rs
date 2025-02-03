use super::db::Model;
use crate::areas;
use crate::common::crud::traits::CRUDResource;
use crate::plots::db::Gradientchoices;
use async_trait::async_trait;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use sea_orm::sea_query::Expr;
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, FromQueryResult, Order, PaginatorTrait, QueryOrder, QuerySelect,
};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct PlotSimple {
    pub id: Uuid,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_srid: Option<i32>,
    pub coord_x: Option<f64>,
    pub coord_y: Option<f64>,
    pub coord_z: Option<f64>,
}

impl From<Model> for PlotSimple {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            latitude: None,
            longitude: None,
            coord_srid: None,
            coord_x: None,
            coord_y: None,
            coord_z: None,
        }
    }
}

impl PlotSimple {
    pub async fn from_db(plot: super::db::Model, db: &DatabaseConnection) -> Self {
        let plot = super::db::Entity::find()
            .filter(super::db::Column::Id.eq(plot.id))
            .column_as(Expr::cust("ST_X(geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
            .column_as(Expr::cust("ST_SRID(geom)"), "coord_srid")
            .column_as(Expr::cust("ST_X(st_transform(geom, 4326))"), "longitude")
            .column_as(Expr::cust("ST_Y(st_transform(geom, 4326))"), "latitude")
            .into_model::<PlotSimple>()
            .one(db)
            .await
            .unwrap()
            .unwrap();

        PlotSimple {
            id: plot.id,
            name: plot.name,
            latitude: plot.latitude,
            longitude: plot.longitude,
            coord_srid: plot.coord_srid,
            coord_x: plot.coord_x,
            coord_y: plot.coord_y,
            coord_z: plot.coord_z,
        }
    }

    pub async fn from_area(area: &crate::areas::db::Model, db: &DatabaseConnection) -> Vec<Self> {
        super::db::Entity::find()
            .filter(super::db::Column::AreaId.eq(area.id))
            .column_as(Expr::cust("ST_X(geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
            .column_as(Expr::cust("ST_SRID(geom)"), "coord_srid")
            .column_as(Expr::cust("ST_X(st_transform(geom, 4326))"), "longitude")
            .column_as(Expr::cust("ST_Y(st_transform(geom, 4326))"), "latitude")
            .into_model::<PlotSimple>()
            .all(db)
            .await
            .unwrap()
    }
}
#[derive(ToSchema, Serialize)]
pub struct PlotBasicWithAreaAndProject {
    pub id: Uuid,
    pub name: String,
    pub area: crate::areas::models::AreaBasicWithProject,
}

#[derive(ToSchema, Serialize)]
pub struct Plot {
    id: Uuid,
    name: String,
    plot_iterator: i32,
    area_id: Uuid,
    gradient: Gradientchoices,
    vegetation_type: Option<String>,
    topography: Option<String>,
    aspect: Option<String>,
    created_on: Option<NaiveDate>,
    weather: Option<String>,
    lithology: Option<String>,
    last_updated: NaiveDateTime,
    image: Option<String>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
    area: Area,
}

#[derive(ToSchema, FromQueryResult, Serialize)]
pub struct PlotWithCoords {
    // Represents the model of the query for get all plots with the extra
    // coordinate fields
    id: Uuid,
    name: String,
    plot_iterator: i32,
    area_id: Uuid,
    gradient: Gradientchoices,
    vegetation_type: Option<String>,
    topography: Option<String>,
    aspect: Option<String>,
    created_on: Option<NaiveDate>,
    weather: Option<String>,
    lithology: Option<String>,
    last_updated: NaiveDateTime,
    image: Option<String>,
    coord_x: Option<f64>,
    coord_y: Option<f64>,
    coord_z: Option<f64>,
}
#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Area {
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
}

impl From<areas::db::Model> for Area {
    fn from(area_db: areas::db::Model) -> Self {
        Area {
            id: area_db.id,
            name: area_db.name,
            description: area_db.description,
        }
    }
}

impl From<super::db::Model> for Plot {
    fn from(model: super::db::Model) -> Self {
        Plot {
            id: model.id,
            name: model.name,
            plot_iterator: model.plot_iterator,
            area_id: model.area_id,
            gradient: model.gradient,
            vegetation_type: model.vegetation_type,
            topography: model.topography,
            aspect: model.aspect,
            created_on: model.created_on,
            weather: model.weather,
            lithology: model.lithology,
            last_updated: model.last_updated,
            image: model.image,
            coord_x: None,
            coord_y: None,
            coord_z: None,
            area: Area {
                id: Uuid::nil(),
                name: None,
                description: None,
            },
        }
    }
}

impl From<(PlotWithCoords, Option<Area>)> for Plot {
    fn from((plot_db, area_db_vec): (PlotWithCoords, Option<Area>)) -> Self {
        let area = area_db_vec.into_iter().next().map_or(
            Area {
                id: Uuid::nil(),
                name: None,
                description: None,
            },
            Area::from,
        );

        Plot {
            id: plot_db.id,
            name: plot_db.name,
            plot_iterator: plot_db.plot_iterator,
            area_id: plot_db.area_id,
            gradient: plot_db.gradient,
            vegetation_type: plot_db.vegetation_type,
            topography: plot_db.topography,
            aspect: plot_db.aspect,
            created_on: plot_db.created_on,
            weather: plot_db.weather,
            lithology: plot_db.lithology,
            last_updated: plot_db.last_updated,
            image: plot_db.image,
            coord_x: plot_db.coord_x,
            coord_y: plot_db.coord_y,
            coord_z: plot_db.coord_z,
            area,
        }
    }
}
#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]

pub struct PlotCreate {
    pub name: String,
    pub area_id: Uuid,
    pub gradient: Gradientchoices,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    pub image: Option<String>,
}

impl From<PlotCreate> for super::db::ActiveModel {
    fn from(plot: PlotCreate) -> Self {
        let now = chrono::Utc::now().naive_utc();
        super::db::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            name: ActiveValue::Set(plot.name),
            area_id: ActiveValue::Set(plot.area_id),
            gradient: ActiveValue::Set(plot.gradient),
            vegetation_type: ActiveValue::Set(plot.vegetation_type),
            topography: ActiveValue::Set(plot.topography),
            aspect: ActiveValue::Set(plot.aspect),
            weather: ActiveValue::Set(plot.weather),
            lithology: ActiveValue::Set(plot.lithology),
            image: ActiveValue::Set(plot.image),
            created_on: ActiveValue::Set(Some(chrono::Utc::now().date_naive())),
            last_updated: ActiveValue::Set(now),
            plot_iterator: ActiveValue::NotSet,
            // Assuming geom is managed separately
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PlotUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub area_id: Option<Option<Uuid>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub gradient: Option<Option<Gradientchoices>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub vegetation_type: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub topography: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub aspect: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub weather: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub lithology: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub image: Option<Option<String>>,
}

impl PlotUpdate {
    pub fn merge_into_activemodel(
        self,
        mut model: super::db::ActiveModel,
    ) -> super::db::ActiveModel {
        model.name = match self.name {
            Some(Some(name)) => ActiveValue::Set(name),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.area_id = match self.area_id {
            Some(Some(area_id)) => ActiveValue::Set(area_id),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.gradient = match self.gradient {
            Some(Some(gradient)) => ActiveValue::Set(gradient),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.vegetation_type = match self.vegetation_type {
            Some(Some(vegetation_type)) => ActiveValue::Set(Some(vegetation_type)),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.topography = match self.topography {
            Some(Some(topography)) => ActiveValue::Set(Some(topography)),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.aspect = match self.aspect {
            Some(Some(aspect)) => ActiveValue::Set(Some(aspect)),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.weather = match self.weather {
            Some(Some(weather)) => ActiveValue::Set(Some(weather)),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.lithology = match self.lithology {
            Some(Some(lithology)) => ActiveValue::Set(Some(lithology)),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.image = match self.image {
            Some(Some(image)) => ActiveValue::Set(Some(image)),
            None => ActiveValue::NotSet,
            _ => ActiveValue::NotSet,
        };
        model.last_updated = ActiveValue::Set(chrono::Utc::now().naive_utc());

        model
    }
}

// ---------------------------------------------------------------------------
// CRUDResource Implementation for Plot
// ---------------------------------------------------------------------------
#[async_trait]
impl CRUDResource for Plot {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Plot;
    type CreateModel = PlotCreate;
    type UpdateModel = PlotUpdate;

    const RESOURCE_NAME_SINGULAR: &'static str = "plot";
    const RESOURCE_NAME_PLURAL: &'static str = "plots";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        // Call find_also_related BEFORE converting into our custom model.
        let tuples = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .column_as(Expr::cust("ST_X(geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
            .find_also_related(crate::areas::db::Entity)
            .into_model::<PlotWithCoords, Area>() // Two type parameters
            .all(db)
            .await?;
        // Map (PlotWithCoords, Vec<areas::db::Model>) into Plot.
        let plots = tuples
            .into_iter()
            .map(|(plot_with_coords, area_vec)| {
                let area_opt = area_vec
                    .into_iter()
                    .next()
                    .map(|area_db| Area::from(area_db));
                Plot::from((plot_with_coords, area_opt))
            })
            .collect();
        Ok(plots)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let tuple_opt = Self::EntityType::find()
            .filter(super::db::Column::Id.eq(id))
            .column_as(Expr::cust("ST_X(geom)"), "coord_x")
            .column_as(Expr::cust("ST_Y(geom)"), "coord_y")
            .column_as(Expr::cust("ST_Z(geom)"), "coord_z")
            .find_also_related(crate::areas::db::Entity)
            .into_model::<PlotWithCoords, Area>() // Two type parameters
            .one(db)
            .await?;
        if let Some((plot_with_coords, area_vec)) = tuple_opt {
            let area_opt = area_vec
                .into_iter()
                .next()
                .map(|area_db| Area::from(area_db));
            Ok(Plot::from((plot_with_coords, area_opt)))
        } else {
            Err(DbErr::RecordNotFound("Plot not found".into()))
        }
    }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let active_model: Self::ActiveModelType = create_model.into();
        let inserted = active_model.insert(db).await?;
        Self::get_one(db, inserted.id).await
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find()
            .filter(super::db::Column::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot not found".into()))?
            .into();
        let updated_model = update_model.merge_into_activemodel(existing);
        let updated = updated_model.update(db).await?;
        Self::get_one(db, updated.id).await
    }

    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
        let res = Self::EntityType::delete_by_id(id).exec(db).await?;
        Ok(res.rows_affected as usize)
    }

    async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr> {
        Self::EntityType::delete_many()
            .filter(super::db::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(ids)
    }

    async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64 {
        Self::EntityType::find()
            .filter(condition)
            .count(db)
            .await
            .unwrap_or(0)
    }

    fn default_index_column() -> Self::ColumnType {
        super::db::Column::Id
    }

    fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            ("last_updated", super::db::Column::LastUpdated),
        ]
    }

    fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            ("area_id", super::db::Column::AreaId),
        ]
    }
}
