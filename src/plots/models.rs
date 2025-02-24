use crate::config::Config;
use crate::plots::db::Gradientchoices;
use async_trait::async_trait;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection,
    DbErr, EntityTrait, Order, QueryOrder, QuerySelect,
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
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now().naive_utc(), on_create = chrono::Utc::now().naive_utc())]
    pub last_updated: NaiveDateTime,
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
        }
    }
}
impl
    From<(
        super::db::Model,
        crate::areas::db::Model,
        Vec<crate::samples::db::Model>,
    )> for Plot
{
    fn from(
        (plot_db, area_db, samples_db): (
            super::db::Model,
            crate::areas::db::Model,
            Vec<crate::samples::db::Model>,
        ),
    ) -> Self {
        let area: crate::areas::models::Area = area_db.into();
        let samples: Vec<crate::samples::models::PlotSample> = samples_db
            .into_iter()
            .map(|sample| crate::samples::models::PlotSample::from(sample))
            .collect();
        let mut plot: Plot = plot_db.into();

        plot.area = Some(area);
        plot.samples = samples;
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

            plots.push(Plot::from((obj, area, samples)));
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

        Ok(Plot::from((plot, area, samples)))
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
