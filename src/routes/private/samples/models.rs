use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel, traits::MergeIntoActiveModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, sea_query::Order,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, ToCreateModel, ToUpdateModel, Deserialize, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct PlotSample {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_create = chrono::Local::now().naive_local())]
    pub created_on: Option<NaiveDate>,
    #[crudcrate(
        update_model = false, create_model = false,
        on_create = Utc::now(),
        on_update = Utc::now()
    )]
    pub last_updated: DateTime<Utc>,
    pub name: String,
    pub upper_depth_cm: f64,
    pub lower_depth_cm: f64,
    pub plot_id: Uuid,
    pub soil_classification_id: Option<Uuid>,
    pub sample_weight: Option<f64>,
    pub subsample_weight: Option<f64>,
    pub ph: Option<f64>,
    pub rh: Option<f64>,
    pub loi: Option<f64>,
    pub mfc: Option<f64>,
    pub c: Option<f64>,
    pub n: Option<f64>,
    pub cn: Option<f64>,
    pub clay_percent: Option<f64>,
    pub silt_percent: Option<f64>,
    pub sand_percent: Option<f64>,
    pub fe_ug_per_g: Option<f64>,
    pub na_ug_per_g: Option<f64>,
    pub al_ug_per_g: Option<f64>,
    pub k_ug_per_g: Option<f64>,
    pub ca_ug_per_g: Option<f64>,
    pub mg_ug_per_g: Option<f64>,
    pub mn_ug_per_g: Option<f64>,
    pub s_ug_per_g: Option<f64>,
    pub cl_ug_per_g: Option<f64>,
    pub p_ug_per_g: Option<f64>,
    pub si_ug_per_g: Option<f64>,
    pub subsample_replica_weight: Option<f64>,
    pub fungi_per_g: Option<f64>,
    pub bacteria_per_g: Option<f64>,
    pub archea_per_g: Option<f64>,
    pub methanogens_per_g: Option<f64>,
    pub methanotrophs_per_g: Option<f64>,
    pub replicate: i32,
    #[crudcrate(update_model = false, create_model = false)]
    pub plot: Option<crate::routes::private::plots::models::Plot>,
    pub sampled_on: Option<NaiveDate>,
    #[crudcrate(non_db_attr = true, default = None)]
    pub fe_abundance_g_per_cm3: Option<f64>,
    #[crudcrate(non_db_attr = true, default = None)]
    pub soc_stock_g_per_cm3: Option<f64>,
}

impl
    From<(
        crate::routes::private::samples::db::Model,
        Option<crate::routes::private::soil::classification::db::Model>,
    )> for PlotSample
{
    fn from(
        (sample, soil_classification): (
            crate::routes::private::samples::db::Model,
            Option<crate::routes::private::soil::classification::db::Model>,
        ),
    ) -> Self {
        let mut sample = PlotSample {
            id: sample.id,
            name: sample.name,
            upper_depth_cm: sample.upper_depth_cm,
            lower_depth_cm: sample.lower_depth_cm,
            plot_id: sample.plot_id,
            soil_classification_id: sample.soil_classification_id,
            sample_weight: sample.sample_weight,
            subsample_weight: sample.subsample_weight,
            ph: sample.ph,
            rh: sample.rh,
            loi: sample.loi,
            mfc: sample.mfc,
            c: sample.c,
            n: sample.n,
            cn: sample.cn,
            clay_percent: sample.clay_percent,
            silt_percent: sample.silt_percent,
            sand_percent: sample.sand_percent,
            fe_ug_per_g: sample.fe_ug_per_g,
            na_ug_per_g: sample.na_ug_per_g,
            al_ug_per_g: sample.al_ug_per_g,
            k_ug_per_g: sample.k_ug_per_g,
            ca_ug_per_g: sample.ca_ug_per_g,
            mg_ug_per_g: sample.mg_ug_per_g,
            mn_ug_per_g: sample.mn_ug_per_g,
            s_ug_per_g: sample.s_ug_per_g,
            cl_ug_per_g: sample.cl_ug_per_g,
            p_ug_per_g: sample.p_ug_per_g,
            si_ug_per_g: sample.si_ug_per_g,
            subsample_replica_weight: sample.subsample_replica_weight,
            fungi_per_g: sample.fungi_per_g,
            bacteria_per_g: sample.bacteria_per_g,
            archea_per_g: sample.archea_per_g,
            methanogens_per_g: sample.methanogens_per_g,
            methanotrophs_per_g: sample.methanotrophs_per_g,
            replicate: sample.replicate,
            last_updated: sample.last_updated,
            created_on: sample.created_on,
            plot: None,
            sampled_on: sample.sampled_on,
            fe_abundance_g_per_cm3: None,
            soc_stock_g_per_cm3: None,
        };
        sample.fe_abundance_g_per_cm3 =
            soil_classification.and_then(|sc| sc.fe_abundance_g_per_cm3);
        sample.soc_stock_g_per_cm3 = sample.calculate_soc_stock();

        sample
    }
}

impl From<crate::routes::private::samples::db::Model> for PlotSample {
    fn from(sample: crate::routes::private::samples::db::Model) -> Self {
        PlotSample {
            id: sample.id,
            name: sample.name,
            upper_depth_cm: sample.upper_depth_cm,
            lower_depth_cm: sample.lower_depth_cm,
            plot_id: sample.plot_id,
            soil_classification_id: sample.soil_classification_id,
            sample_weight: sample.sample_weight,
            subsample_weight: sample.subsample_weight,
            ph: sample.ph,
            rh: sample.rh,
            loi: sample.loi,
            mfc: sample.mfc,
            c: sample.c,
            n: sample.n,
            cn: sample.cn,
            clay_percent: sample.clay_percent,
            silt_percent: sample.silt_percent,
            sand_percent: sample.sand_percent,
            fe_ug_per_g: sample.fe_ug_per_g,
            na_ug_per_g: sample.na_ug_per_g,
            al_ug_per_g: sample.al_ug_per_g,
            k_ug_per_g: sample.k_ug_per_g,
            ca_ug_per_g: sample.ca_ug_per_g,
            mg_ug_per_g: sample.mg_ug_per_g,
            mn_ug_per_g: sample.mn_ug_per_g,
            s_ug_per_g: sample.s_ug_per_g,
            cl_ug_per_g: sample.cl_ug_per_g,
            p_ug_per_g: sample.p_ug_per_g,
            si_ug_per_g: sample.si_ug_per_g,
            subsample_replica_weight: sample.subsample_replica_weight,
            fungi_per_g: sample.fungi_per_g,
            bacteria_per_g: sample.bacteria_per_g,
            archea_per_g: sample.archea_per_g,
            methanogens_per_g: sample.methanogens_per_g,
            methanotrophs_per_g: sample.methanotrophs_per_g,
            replicate: sample.replicate,
            last_updated: sample.last_updated,
            created_on: sample.created_on,
            plot: None,
            sampled_on: sample.sampled_on,
            fe_abundance_g_per_cm3: None,
            soc_stock_g_per_cm3: None,
        }
    }
}

impl
    From<(
        crate::routes::private::samples::db::Model,
        crate::routes::private::plots::db::Model,
    )> for PlotSample
{
    fn from(
        (sample, plot): (
            crate::routes::private::samples::db::Model,
            crate::routes::private::plots::db::Model,
        ),
    ) -> Self {
        let sample: PlotSample = sample.into();
        let plot: crate::routes::private::plots::models::Plot = plot.into();
        PlotSample {
            plot: Some(plot),
            ..sample
        }
    }
}

impl
    From<(
        crate::routes::private::samples::db::Model,
        crate::routes::private::plots::db::Model,
        crate::routes::private::areas::db::Model,
        crate::routes::private::projects::db::Model,
        Option<crate::routes::private::soil::classification::db::Model>,
    )> for PlotSample
{
    fn from(
        (sample, plot, area, project, soil_classification): (
            crate::routes::private::samples::db::Model,
            crate::routes::private::plots::db::Model,
            crate::routes::private::areas::db::Model,
            crate::routes::private::projects::db::Model,
            Option<crate::routes::private::soil::classification::db::Model>,
        ),
    ) -> Self {
        let mut sample: PlotSample = sample.into();
        let mut plot: crate::routes::private::plots::models::Plot = plot.into();
        let mut area: crate::routes::private::areas::models::Area = area.into();
        let project: crate::routes::private::projects::models::Project = project.into();

        area.project = Some(project);
        plot.area = Some(area);
        sample.plot = Some(plot);
        sample.fe_abundance_g_per_cm3 =
            soil_classification.and_then(|sc| sc.fe_abundance_g_per_cm3);
        sample.soc_stock_g_per_cm3 = sample.calculate_soc_stock();
        sample
    }
}

impl PlotSample {
    fn calculate_soc_stock(&self) -> Option<f64> {
        if self.fe_abundance_g_per_cm3.is_none() || self.c.is_none() {
            return None;
        }
        let depth_m = self.lower_depth_cm - self.upper_depth_cm;
        let soc_stock = (depth_m * self.fe_abundance_g_per_cm3.unwrap() * self.c.unwrap()) / 100.0;

        Some(soc_stock)
    }
}
#[async_trait]
impl CRUDResource for PlotSample {
    type EntityType = crate::routes::private::samples::db::Entity;
    type ColumnType = crate::routes::private::samples::db::Column;
    type ActiveModelType = crate::routes::private::samples::db::ActiveModel;
    type CreateModel = PlotSampleCreate;
    type UpdateModel = PlotSampleUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_SINGULAR: &'static str = "sample (plot)";
    const RESOURCE_NAME_PLURAL: &'static str = "samples (plot)";
    const RESOURCE_DESCRIPTION: &'static str = "This resource represents a sample taken from a plot. It contains the data collected from the sample, such as pH, organic carbon, nitrogen, and other soil properties.";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self>, DbErr> {
        let samples = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        let mut plot_samples: Vec<PlotSample> = Vec::new();
        for sample in samples {
            let mut plot = crate::routes::private::plots::db::Entity::find()
                .filter(crate::routes::private::plots::db::Column::Id.eq(sample.plot_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;
            plot.image = None;

            let area = crate::routes::private::areas::db::Entity::find()
                .filter(crate::routes::private::areas::db::Column::Id.eq(plot.area_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Area not found".into()))?;

            let project = crate::routes::private::projects::db::Entity::find()
                .filter(crate::routes::private::projects::db::Column::Id.eq(area.project_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Project not found".into()))?;

            let soil_classification =
                crate::routes::private::soil::classification::db::Entity::find()
                    .filter(
                        crate::routes::private::soil::classification::db::Column::Id
                            .eq(sample.soil_classification_id),
                    )
                    .one(db)
                    .await?;

            plot_samples.push((sample, plot, area, project, soil_classification).into());
        }
        Ok(plot_samples)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self, DbErr> {
        let sample = Self::EntityType::find()
            .filter(crate::routes::private::samples::db::Column::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot sample not found".into()))?;
        let mut plot = crate::routes::private::plots::db::Entity::find()
            .filter(crate::routes::private::plots::db::Column::Id.eq(sample.plot_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;
        plot.image = None;

        let area: crate::routes::private::areas::db::Model =
            crate::routes::private::areas::db::Entity::find()
                .filter(crate::routes::private::areas::db::Column::Id.eq(plot.area_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Area not found".into()))?;

        let project: crate::routes::private::projects::db::Model =
            crate::routes::private::projects::db::Entity::find()
                .filter(crate::routes::private::projects::db::Column::Id.eq(area.project_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Project not found".into()))?;

        let soil_classification = crate::routes::private::soil::classification::db::Entity::find()
            .filter(
                crate::routes::private::soil::classification::db::Column::Id
                    .eq(sample.soil_classification_id),
            )
            .one(db)
            .await?;

        Ok((sample, plot, area, project, soil_classification).into())
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_data: Self::UpdateModel,
    ) -> Result<Self, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find()
            .filter(crate::routes::private::samples::db::Column::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot sample not found".into()))?
            .into();
        let updated_model = update_data.merge_into_activemodel(existing);
        let updated = updated_model.update(db).await?;
        Self::get_one(db, updated.id).await
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", crate::routes::private::samples::db::Column::Id),
            ("name", crate::routes::private::samples::db::Column::Name),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![("name", crate::routes::private::samples::db::Column::Name)]
    }
}
