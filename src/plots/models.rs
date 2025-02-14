use super::db::Model;
use crate::areas;
use crate::common::crud::traits::CRUDResource;
use crate::config::Config;
use crate::plots::db::Gradientchoices;
use async_trait::async_trait;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use crudcrate::ToCreateModel;
use crudcrate::ToUpdateModel;
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, FromQueryResult, Order, QueryOrder, QuerySelect,
};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, FromQueryResult)]
pub struct PlotSimple {
    pub id: Uuid,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub coord_srid: i32,
    pub coord_x: f64,
    pub coord_y: f64,
    pub coord_z: f64,
}

impl From<Model> for PlotSimple {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            latitude: None,
            longitude: None,
            coord_srid: model.coord_srid,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct PlotBasicWithAreaAndProject {
    pub id: Uuid,
    pub name: String,
    pub area: crate::areas::models::AreaBasicWithProject,
}

#[derive(ToSchema, Serialize, ToUpdateModel, ToCreateModel)]
#[active_model = "super::db::ActiveModel"]

pub struct Plot {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    id: Uuid,
    name: String,
    area_id: Uuid,
    gradient: Gradientchoices,
    vegetation_type: Option<String>,
    topography: Option<String>,
    aspect: Option<String>,
    created_on: Option<NaiveDate>,
    weather: Option<String>,
    lithology: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now().naive_utc(), on_create = chrono::Utc::now().naive_utc())]
    last_updated: NaiveDateTime,
    image: Option<String>,
    coord_x: f64,
    coord_y: f64,
    coord_z: f64,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    coord_srid: i32,
    #[crudcrate(update_model = false, create_model = false)]
    area: Area,
    #[crudcrate(update_model = false, create_model = false)]
    samples: Vec<crate::samples::models::PlotSampleBasic>,
}

#[derive(ToSchema, FromQueryResult, Serialize)]
pub struct PlotWithCoords {
    // Represents the model of the query for get all plots with the extra
    // coordinate fields
    id: Uuid,
    name: String,
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
    coord_x: f64,
    coord_y: f64,
    coord_z: f64,
    coord_srid: i32,
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
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
            coord_srid: model.coord_srid,
            area: Area {
                id: Uuid::nil(),
                name: None,
                description: None,
            },
            samples: vec![],
        }
    }
}
impl From<(PlotWithCoords, Option<Area>)> for Plot {
    fn from((plot_db, area_db_vec): (PlotWithCoords, Option<Area>)) -> Self {
        let area = area_db_vec.unwrap_or(Area {
            id: Uuid::nil(),
            name: None,
            description: None,
        });

        Plot {
            id: plot_db.id,
            name: plot_db.name,
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
            coord_srid: plot_db.coord_srid,
            area,
            samples: vec![],
        }
    }
}

#[async_trait]
impl CRUDResource for Plot {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Plot;
    type CreateModel = PlotCreate;
    type UpdateModel = PlotUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
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
        let mut objs: Vec<Plot> = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .find_also_related(crate::areas::db::Entity)
            .into_model::<PlotWithCoords, Area>() // Two type parameters
            .all(db)
            .await
            .unwrap()
            .into_iter()
            .map(|(plot_with_coords, area_vec)| (plot_with_coords, area_vec.into_iter().next()))
            .map(|(plot_with_coords, area_opt)| Plot::from((plot_with_coords, area_opt)))
            .collect();

        // For each plot obj, query for the samples, build the model with the samples and area and return
        // the vector of plots. We have to do this because in order to get the x/y/z coords we need
        // to cast into a non-db model, and we cannot do two joins in the same query in sea-orm.
        for plot in objs.iter_mut() {
            let samples = crate::samples::db::Entity::find()
                .filter(crate::samples::db::Column::PlotId.eq(plot.id))
                .into_model::<crate::samples::models::PlotSampleBasic>()
                .all(db)
                .await
                .unwrap();
            plot.samples = samples;
        }

        Ok(objs)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let tuple_opt = Self::EntityType::find()
            .filter(super::db::Column::Id.eq(id))
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

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            ("last_updated", super::db::Column::LastUpdated),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            ("area_id", super::db::Column::AreaId),
        ]
    }
}
