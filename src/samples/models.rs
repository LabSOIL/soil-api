use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::{query::*, sea_query::Order, DatabaseConnection, QueryFilter, QueryOrder};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct PlotSample {
    pub id: Uuid,
    pub name: String,
    pub upper_depth_cm: f64,
    pub lower_depth_cm: f64,
    pub plot_id: Uuid,
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
    pub plot: crate::plots::models::PlotBasicWithAreaAndProject,
}

impl PlotSample {
    pub async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: <super::db::Entity as sea_orm::EntityTrait>::Column,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Vec<Self> {
        let samples = crate::samples::db::Entity::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await
            .unwrap();

        let mut plot_samples: Vec<PlotSample> = Vec::new();

        for sample in samples {
            let plot_obj = crate::plots::db::Entity::find()
                .filter(crate::plots::db::Column::Id.eq(sample.plot_id))
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let area = crate::areas::db::Entity::find()
                .filter(crate::areas::db::Column::Id.eq(plot_obj.area_id))
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let project = crate::projects::db::Entity::find()
                .filter(crate::projects::db::Column::Id.eq(area.project_id))
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let plot = crate::plots::models::PlotBasicWithAreaAndProject {
                id: plot_obj.id,
                name: plot_obj.name,
                area: crate::areas::models::AreaBasicWithProject {
                    id: area.id,
                    name: area.name,
                    project: crate::common::models::GenericNameAndID {
                        id: project.id,
                        name: project.name,
                    },
                },
            };

            plot_samples.push(PlotSample {
                id: sample.id,
                name: sample.name,
                upper_depth_cm: sample.upper_depth_cm,
                lower_depth_cm: sample.lower_depth_cm,
                plot_id: sample.plot_id,
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
                plot: plot,
            });
        }
        plot_samples
    }

    // pub async fn get_one(id: Uuid, db: &DatabaseConnection) -> Option<Self> {
    //     let sample = crate::samples::db::Entity::find()
    //         .filter(crate::samples::db::Column::Id.eq(id))
    //         .one(db)
    //         .await
    //         .unwrap()
    //         .unwrap();

    //     let plot_obj = crate::plots::db::Entity::find()
    //         .filter(crate::plots::db::Column::Id.eq(sample.plot_id))
    //         .one(db)
    //         .await
    //         .unwrap()
    //         .unwrap();

    //     let area = crate::areas::db::Entity::find()
    //         .filter(crate::areas::db::Column::Id.eq(plot_obj.area_id))
    //         .one(db)
    //         .await
    //         .unwrap()
    //         .unwrap();

    //     let project = crate::projects::db::Entity::find()
    //         .filter(crate::projects::db::Column::Id.eq(area.project_id))
    //         .one(db)
    //         .await
    //         .unwrap()
    //         .unwrap();

    //     let plot = crate::plots::models::PlotBasicWithAreaAndProject {
    //         id: plot_obj.id,
    //         name: plot_obj.name,
    //         area: crate::areas::models::AreaBasicWithProject {
    //             id: area.id,
    //             name: area.name,
    //             project: crate::common::models::GenericNameAndID {
    //                 id: project.id,
    //                 name: project.name,
    //             },
    //         },
    //     };

    //     Some(PlotSample {
    //         id: sample.id,
    //         name: sample.name,
    //         upper_depth_cm: sample.upper_depth_cm,
    //         lower_depth_cm: sample.lower_depth_cm,
    //         plot_id: sample.plot_id,
    //         sample_weight: sample.sample_weight,
    //         subsample_weight: sample.subsample_weight,
    //         ph: sample.ph,
    //         rh: sample.rh,
    //         loi: sample.loi,
    //         mfc: sample.mfc,
    //         c: sample.c,
    //         n: sample.n,
    //         cn: sample.cn,
    //         clay_percent: sample.clay_percent,
    //         silt_percent: sample.silt_percent,
    //         sand_percent: sample.sand_percent,
    //         fe_ug_per_g: sample.fe_ug_per_g,
    //         na_ug_per_g: sample.na_ug_per_g,
    //         al_ug_per_g: sample.al_ug_per_g,
    //         k_ug_per_g: sample.k_ug_per_g,
    //         ca_ug_per_g: sample.ca_ug_per_g,
    //         mg_ug_per_g: sample.mg_ug_per_g,
    //         mn_ug_per_g: sample.mn_ug_per_g,
    //         s_ug_per_g: sample.s_ug_per_g,
    //         cl_ug_per_g: sample.cl_ug_per_g,
    //         p_ug_per_g: sample.p_ug_per_g,
    //         si_ug_per_g: sample.si_ug_per_g,
    //         subsample_replica_weight: sample.subsample_replica_weight,
    //         fungi_per_g: sample.fungi_per_g,
    //         bacteria_per_g: sample.bacteria_per_g,
    //         archea_per_g: sample.archea_per_g,
    //         methanogens_per_g: sample.methanogens_per_g,
    //         methanotrophs_per_g: sample.methanotrophs_per_g,
    //         replicate: sample.replicate,
    //         plot: plot,
    //     })
    // }
}
