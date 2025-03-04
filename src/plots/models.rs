use crate::plots::db::Gradientchoices;
use crate::{config::Config, transects::models::Transect};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbBackend, DbErr,
    EntityTrait, FromQueryResult, Order, QueryOrder, QuerySelect, Statement, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, ToUpdateModel, ToCreateModel, Deserialize, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct Plot {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub area_id: Uuid,
    pub gradient: Gradientchoices,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    pub created_on: Option<NaiveDate>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now(), on_create = chrono::Utc::now())]
    pub last_updated: DateTime<Utc>,
    pub image: Option<String>,
    pub coord_x: f64,
    pub coord_y: f64,
    pub coord_z: f64,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    pub coord_srid: i32,
    #[crudcrate(update_model = false, create_model = false)]
    pub area: Option<crate::areas::models::Area>,
    #[crudcrate(update_model = false, create_model = false)]
    pub samples: Vec<crate::samples::models::PlotSample>,
    #[crudcrate(update_model = false, create_model = false)]
    pub nearest_sensor_profiles: Vec<ClosestSensorProfile>,
    #[crudcrate(update_model = false, create_model = false)]
    pub transects: Vec<crate::transects::models::Transect>,
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
            area: None,
            samples: vec![],
            nearest_sensor_profiles: vec![],
            transects: vec![],
        }
    }
}
impl
    From<(
        super::db::Model,
        crate::areas::db::Model,
        Vec<crate::samples::db::Model>,
        Vec<ClosestSensorProfile>,
        Vec<Transect>,
    )> for Plot
{
    fn from(
        (plot_db, area_db, samples_db, nearest_sensor_profiles, transects): (
            super::db::Model,
            crate::areas::db::Model,
            Vec<crate::samples::db::Model>,
            Vec<ClosestSensorProfile>,
            Vec<Transect>,
        ),
    ) -> Self {
        let area: crate::areas::models::Area = area_db.into();
        let samples: Vec<crate::samples::models::PlotSample> = samples_db
            .into_iter()
            .map(crate::samples::models::PlotSample::from)
            .collect();
        let mut plot: Plot = plot_db.into();

        plot.area = Some(area);
        plot.samples = samples;
        plot.nearest_sensor_profiles = nearest_sensor_profiles;
        plot.transects = transects;

        plot
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
        let objs = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            // .find_also_related(crate::areas::db::Entity)
            .all(db)
            .await
            .unwrap();

        let mut plots = Vec::new();
        for obj in objs {
            let area = obj
                .find_related(crate::areas::db::Entity)
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let samples = obj
                .find_related(crate::samples::db::Entity)
                .all(db)
                .await
                .unwrap();

            plots.push(Plot::from((obj, area, samples, vec![], vec![])));
        }

        Ok(plots)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let plot = Self::EntityType::find()
            .filter(super::db::Column::Id.eq(id))
            .one(db)
            .await
            .unwrap()
            .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;

        let area = plot
            .find_related(crate::areas::db::Entity)
            .one(db)
            .await
            .unwrap()
            .unwrap();

        let samples = plot
            .find_related(crate::samples::db::Entity)
            .all(db)
            .await
            .unwrap();

        // Search transect nodes table where the plot_id is the same as the id
        // then get the transect that it belongs to
        let transect_nodes = crate::transects::nodes::db::Entity::find()
            .filter(crate::transects::nodes::db::Column::PlotId.eq(id))
            // .find_also_related(crate::transects::db::Entity)
            .all(db)
            .await;

        let mut transects = vec![];
        if let Ok(transect_nodes) = transect_nodes {
            for node in transect_nodes {
                let transect = node
                    .find_related(crate::transects::db::Entity)
                    .one(db)
                    .await
                    .unwrap()
                    .unwrap();
                transects.push(crate::transects::models::Transect::from(transect));
            }
        }

        // Get "nearest_sensor_profiles" from the sensor profile table with a postgis spatial query
        // on the geom of the plot vs the geom of the sensor profile as nearest distance
        // let mut nearest_sensor_profiles = vec![];

        // Perform query
        // select b.id, st_distance(a.geom, b.geom) from plot a, sensorprofile b where a.area_id = b.area_id and a.id = '0ced76e8-1526-4f28-93ad-9378926183af' order by st_distance(a.geom, b.geom);
        println!("Plot ID: {:?}", id);
        let nearest_sensor_profiles: Vec<ClosestSensorProfile> =
            ClosestSensorProfile::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r"SELECT b.id, st_distance(a.geom, b.geom) AS distance, st_z(a.geom) - st_z(b.geom) AS elevation_difference, b.name
                FROM plot a, sensorprofile b
                WHERE a.area_id = b.area_id
                AND a.id = $1
                ORDER BY st_distance(a.geom, b.geom);
            ",
            vec![id.into()],
            ))
            .all(db)
            .await
            .unwrap_or_else(|_| vec![]);
        println!("Nearest Sensor Profiles: {:?}", nearest_sensor_profiles);
        Ok(Plot::from((
            plot,
            area,
            samples,
            nearest_sensor_profiles,
            transects,
        )))
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_data: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find()
            .filter(super::db::Column::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot not found".into()))?
            .into();
        let updated_model = update_data.merge_into_activemodel(existing);
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
            ("name", super::db::Column::Name),
            ("vegetation_type", super::db::Column::VegetationType),
            ("topography", super::db::Column::Topography),
            ("aspect", super::db::Column::Aspect),
            // ("gradient", super::db::Column::Gradient),
            ("weather", super::db::Column::Weather),
            ("lithology", super::db::Column::Lithology),
        ]
    }
}

#[derive(Debug, FromQueryResult, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClosestSensorProfile {
    pub id: Uuid,
    pub name: String,
    pub distance: f64,
    pub elevation_difference: f64,
}
