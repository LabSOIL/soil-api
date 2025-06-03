use crate::config::Config;
use crate::routes::private::{plots::db::Gradientchoices, transects::models::Transect};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbBackend, DbErr,
    EntityTrait, FromQueryResult, Order, QueryOrder, QuerySelect, Statement, entity::prelude::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToUpdateModel, ToCreateModel, Deserialize, Clone, ToSchema)]
#[active_model = "super::db::ActiveModel"]
pub struct Plot {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub area_id: Uuid,
    pub gradient: Option<Gradientchoices>,
    pub vegetation_type: Option<String>,
    pub topography: Option<String>,
    pub aspect: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_create = chrono::Local::now().naive_local())]
    pub created_on: Option<NaiveDate>,
    pub weather: Option<String>,
    pub lithology: Option<String>,
    pub slope: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now(), on_create = chrono::Utc::now())]
    pub last_updated: DateTime<Utc>,
    pub image: Option<String>,
    pub coord_x: f64,
    pub coord_y: f64,
    pub coord_z: f64,
    #[crudcrate(update_model = false, create_model = false, on_create = Config::from_env().srid)]
    pub coord_srid: i32,
    #[crudcrate(update_model = false, create_model = false)]
    #[schema(no_recursion)]
    pub area: Option<crate::routes::private::areas::models::Area>,
    #[crudcrate(update_model = false, create_model = false)]
    #[schema(no_recursion)]
    pub samples: Vec<crate::routes::private::samples::models::PlotSample>,
    #[crudcrate(update_model = false, create_model = false)]
    pub nearest_sensor_profiles: Vec<ClosestSensorProfile>,
    #[crudcrate(update_model = false, create_model = false)]
    pub transects: Vec<crate::routes::private::transects::models::Transect>,
    // Store the replicate aggregates in a hashmap where replicate ID is the key
    #[crudcrate(non_db_attr = true, default = HashMap::new())]
    pub aggregated_samples: HashMap<i32, SampleReplicateAggregate>,
}

impl Plot {
    pub fn aggregate_samples(&self) -> HashMap<i32, SampleReplicateAggregate> {
        let mut sample_replicates: HashMap<i32, SampleReplicateAggregate> = HashMap::new();

        // Aggregate the samples by replicate
        for sample in &self.samples {
            let replicate = sample.replicate;
            let depth: f64 = sample.lower_depth_cm - sample.upper_depth_cm;

            let entry = sample_replicates
                .entry(replicate)
                .or_insert(SampleReplicateAggregate {
                    sample_count: 0,
                    mean_c: 0.0,
                    ph: None,
                    total_depth: 0.0,
                    soc_stock_to_total_depth_g_per_cm3: 0.0,
                    soc_stock_megag_per_hectare: 0.0,
                });

            if sample.upper_depth_cm == 0.0 && sample.ph.is_some() {
                entry.ph = sample.ph;
            }
            entry.sample_count += 1;
            entry.mean_c += sample.c.unwrap_or(0.0);
            entry.total_depth += depth;
            entry.soc_stock_to_total_depth_g_per_cm3 += sample.soc_stock_g_per_cm3.unwrap_or(0.0);
        }

        // Finalize the calculations for each replicate
        for aggregate in sample_replicates.values_mut() {
            if aggregate.sample_count > 0 {
                aggregate.mean_c /= f64::from(aggregate.sample_count);
                aggregate.soc_stock_megag_per_hectare =
                    aggregate.soc_stock_to_total_depth_g_per_cm3 * 100.0;
            }
        }

        sample_replicates
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct SampleReplicateAggregate {
    pub sample_count: i32,
    pub mean_c: f64,
    pub ph: Option<f64>,
    pub total_depth: f64,
    pub soc_stock_to_total_depth_g_per_cm3: f64,
    pub soc_stock_megag_per_hectare: f64,
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
            slope: model.slope,
            coord_x: model.coord_x,
            coord_y: model.coord_y,
            coord_z: model.coord_z,
            coord_srid: model.coord_srid,
            area: None,
            samples: vec![],
            nearest_sensor_profiles: vec![],
            transects: vec![],
            aggregated_samples: HashMap::new(),
        }
    }
}
impl
    From<(
        super::db::Model,
        crate::routes::private::areas::db::Model,
        Vec<crate::routes::private::samples::models::PlotSample>,
        Vec<ClosestSensorProfile>,
        Vec<Transect>,
    )> for Plot
{
    fn from(
        (plot_db, area_db, samples, nearest_sensor_profiles, transects): (
            super::db::Model,
            crate::routes::private::areas::db::Model,
            Vec<crate::routes::private::samples::models::PlotSample>,
            Vec<ClosestSensorProfile>,
            Vec<Transect>,
        ),
    ) -> Self {
        let area: crate::routes::private::areas::models::Area = area_db.into();

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
    type ActiveModelType = super::db::ActiveModel;
    type CreateModel = PlotCreate;
    type UpdateModel = PlotUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "plot";
    const RESOURCE_NAME_PLURAL: &'static str = "plots";
    const RESOURCE_DESCRIPTION: &'static str = "This is a record of a plot, which is a specific area of land that is being studied. It is used to collect samples and data for analysis.";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self>, DbErr> {
        let objs = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            // .find_also_related(crate::routes::private::areas::db::Entity)
            .all(db)
            .await
            .unwrap();

        let mut plots = Vec::new();
        for obj in objs {
            let area = obj
                .find_related(crate::routes::private::areas::db::Entity)
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let samples = obj
                .find_related(crate::routes::private::samples::db::Entity)
                .all(db)
                .await
                .unwrap();
            let mut sample_objs = Vec::new();
            // Get the soil classification for each sample
            for sample in samples {
                let soil_classification =
                    crate::routes::private::soil::classification::db::Entity::find()
                        .filter(
                            crate::routes::private::soil::classification::db::Column::Id
                                .eq(sample.soil_classification_id),
                        )
                        .one(db)
                        .await
                        .unwrap();

                let updated_sample = crate::routes::private::samples::models::PlotSample::from((
                    sample.clone(),
                    soil_classification,
                ));
                sample_objs.push(updated_sample);
            }

            let mut plot: Plot = (obj, area, sample_objs, vec![], vec![]).into();

            // We need to aggregate the samples to get some plot specific values
            plot.aggregated_samples = plot.aggregate_samples();

            plots.push(plot);
        }

        Ok(plots)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let plot = Self::EntityType::find()
            .filter(super::db::Column::Id.eq(id))
            .one(db)
            .await
            .unwrap()
            .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;

        let area = plot
            .find_related(crate::routes::private::areas::db::Entity)
            .one(db)
            .await
            .unwrap()
            .unwrap();

        let samples = plot
            .find_related(crate::routes::private::samples::db::Entity)
            .all(db)
            .await
            .unwrap();
        let mut sample_objs = Vec::new();
        // Get the soil classification for each sample
        for sample in samples {
            let soil_classification =
                crate::routes::private::soil::classification::db::Entity::find()
                    .filter(
                        crate::routes::private::soil::classification::db::Column::Id
                            .eq(sample.soil_classification_id),
                    )
                    .one(db)
                    .await
                    .unwrap();

            let updated_sample = crate::routes::private::samples::models::PlotSample::from((
                sample.clone(),
                soil_classification,
            ));
            sample_objs.push(updated_sample);
        }

        // Search transect nodes table where the plot_id is the same as the id
        // then get the transect that it belongs to
        let transect_nodes = crate::routes::private::transects::nodes::db::Entity::find()
            .filter(crate::routes::private::transects::nodes::db::Column::PlotId.eq(id))
            // .find_also_related(crate::routes::private::transects::db::Entity)
            .all(db)
            .await;

        let mut transects = vec![];
        if let Ok(transect_nodes) = transect_nodes {
            for node in transect_nodes {
                let transect = node
                    .find_related(crate::routes::private::transects::db::Entity)
                    .one(db)
                    .await
                    .unwrap()
                    .unwrap();
                transects.push(crate::routes::private::transects::models::Transect::from(
                    transect,
                ));
            }
        }

        // Get "nearest_sensor_profiles" from the sensor profile table with a postgis spatial query
        // on the geom of the plot vs the geom of the sensor profile as nearest distance
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

        let mut plot = Plot::from((plot, area, sample_objs, nearest_sensor_profiles, transects));
        plot.aggregated_samples = plot.aggregate_samples();

        Ok(plot)
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_data: Self::UpdateModel,
    ) -> Result<Self, DbErr> {
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
