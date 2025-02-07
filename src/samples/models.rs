use crate::common::crud::traits::CRUDResource;
use crate::plots::models::PlotBasicWithAreaAndProject;
use async_trait::async_trait;
use crudcrate::{ToCreateModel, ToUpdateModel};
use sea_orm::{
    query::*, sea_query::Order, ActiveModelTrait, ActiveValue, ColumnTrait, Condition,
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, ToCreateModel, ToUpdateModel)]
#[active_model = "crate::samples::db::ActiveModel"]
pub struct PlotSample {
    #[update = false]
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
    #[update = false]
    pub plot: crate::plots::models::PlotBasicWithAreaAndProject,
}
#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct PlotSampleBasic {
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
}

impl From<crate::samples::db::Model> for PlotSample {
    fn from(sample: crate::samples::db::Model) -> Self {
        let plot = crate::plots::models::PlotBasicWithAreaAndProject {
            id: sample.plot_id,
            name: String::new(), // Placeholder, should be fetched from DB
            area: crate::areas::models::AreaBasicWithProject {
                id: Uuid::new_v4(),        // Placeholder, should be fetched from DB
                name: Some(String::new()), // Placeholder, should be fetched from DB
                project: crate::common::crud::models::GenericNameAndID {
                    id: Uuid::new_v4(),  // Placeholder, should be fetched from DB
                    name: String::new(), // Placeholder, should be fetched from DB
                },
            },
        };

        PlotSample {
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
            plot,
        }
    }
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
                    project: crate::common::crud::models::GenericNameAndID {
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
}

// // // === Create Model ===
// #[derive(Serialize, Deserialize, ToSchema)]
// pub struct PlotSampleCreate {
//     pub name: String,
//     pub upper_depth_cm: f64,
//     pub lower_depth_cm: f64,
//     pub plot_id: Uuid,
//     pub sample_weight: Option<f64>,
//     pub subsample_weight: Option<f64>,
//     pub ph: Option<f64>,
//     pub rh: Option<f64>,
//     pub loi: Option<f64>,
//     pub mfc: Option<f64>,
//     pub c: Option<f64>,
//     pub n: Option<f64>,
//     pub cn: Option<f64>,
//     pub clay_percent: Option<f64>,
//     pub silt_percent: Option<f64>,
//     pub sand_percent: Option<f64>,
//     pub fe_ug_per_g: Option<f64>,
//     pub na_ug_per_g: Option<f64>,
//     pub al_ug_per_g: Option<f64>,
//     pub k_ug_per_g: Option<f64>,
//     pub ca_ug_per_g: Option<f64>,
//     pub mg_ug_per_g: Option<f64>,
//     pub mn_ug_per_g: Option<f64>,
//     pub s_ug_per_g: Option<f64>,
//     pub cl_ug_per_g: Option<f64>,
//     pub p_ug_per_g: Option<f64>,
//     pub si_ug_per_g: Option<f64>,
//     pub subsample_replica_weight: Option<f64>,
//     pub fungi_per_g: Option<f64>,
//     pub bacteria_per_g: Option<f64>,
//     pub archea_per_g: Option<f64>,
//     pub methanogens_per_g: Option<f64>,
//     pub methanotrophs_per_g: Option<f64>,
//     pub replicate: i32,
// }

// Conversion from create model to the DB active model.
impl From<PlotSampleCreate> for crate::samples::db::ActiveModel {
    fn from(sample: PlotSampleCreate) -> Self {
        let now = chrono::Utc::now().naive_utc();
        crate::samples::db::ActiveModel {
            created_on: ActiveValue::Set(Some(now.date())),
            id: ActiveValue::Set(Uuid::new_v4()),
            name: ActiveValue::Set(sample.name),
            upper_depth_cm: ActiveValue::Set(sample.upper_depth_cm),
            lower_depth_cm: ActiveValue::Set(sample.lower_depth_cm),
            plot_id: ActiveValue::Set(sample.plot_id),
            sample_weight: ActiveValue::Set(sample.sample_weight),
            subsample_weight: ActiveValue::Set(sample.subsample_weight),
            ph: ActiveValue::Set(sample.ph),
            rh: ActiveValue::Set(sample.rh),
            loi: ActiveValue::Set(sample.loi),
            mfc: ActiveValue::Set(sample.mfc),
            c: ActiveValue::Set(sample.c),
            n: ActiveValue::Set(sample.n),
            cn: ActiveValue::Set(sample.cn),
            clay_percent: ActiveValue::Set(sample.clay_percent),
            silt_percent: ActiveValue::Set(sample.silt_percent),
            sand_percent: ActiveValue::Set(sample.sand_percent),
            fe_ug_per_g: ActiveValue::Set(sample.fe_ug_per_g),
            na_ug_per_g: ActiveValue::Set(sample.na_ug_per_g),
            al_ug_per_g: ActiveValue::Set(sample.al_ug_per_g),
            k_ug_per_g: ActiveValue::Set(sample.k_ug_per_g),
            ca_ug_per_g: ActiveValue::Set(sample.ca_ug_per_g),
            mg_ug_per_g: ActiveValue::Set(sample.mg_ug_per_g),
            mn_ug_per_g: ActiveValue::Set(sample.mn_ug_per_g),
            s_ug_per_g: ActiveValue::Set(sample.s_ug_per_g),
            cl_ug_per_g: ActiveValue::Set(sample.cl_ug_per_g),
            p_ug_per_g: ActiveValue::Set(sample.p_ug_per_g),
            si_ug_per_g: ActiveValue::Set(sample.si_ug_per_g),
            subsample_replica_weight: ActiveValue::Set(sample.subsample_replica_weight),
            fungi_per_g: ActiveValue::Set(sample.fungi_per_g),
            bacteria_per_g: ActiveValue::Set(sample.bacteria_per_g),
            archea_per_g: ActiveValue::Set(sample.archea_per_g),
            methanogens_per_g: ActiveValue::Set(sample.methanogens_per_g),
            methanotrophs_per_g: ActiveValue::Set(sample.methanotrophs_per_g),
            replicate: ActiveValue::Set(sample.replicate),
            last_updated: ActiveValue::Set(now),
        }
    }
}

// // === Update Model ===
// #[derive(Serialize, Deserialize, ToSchema)]
// pub struct PlotSampleUpdate {
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub name: Option<Option<String>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub upper_depth_cm: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub lower_depth_cm: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub plot_id: Option<Option<Uuid>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub sample_weight: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub subsample_weight: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub ph: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub rh: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub loi: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub mfc: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub c: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub n: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub cn: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub clay_percent: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub silt_percent: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub sand_percent: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub fe_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub na_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub al_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub k_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub ca_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub mg_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub mn_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub s_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub cl_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub p_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub si_ug_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub subsample_replica_weight: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub fungi_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub bacteria_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub archea_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub methanogens_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub methanotrophs_per_g: Option<Option<f64>>,
//     #[serde(
//         default,
//         skip_serializing_if = "Option::is_none",
//         with = "::serde_with::rust::double_option"
//     )]
//     pub replicate: Option<Option<i32>>,
// }

// impl PlotSampleUpdate {
//     pub fn merge_into_activemodel(
//         self,
//         mut model: crate::samples::db::ActiveModel,
//     ) -> crate::samples::db::ActiveModel {
//         if let Some(opt) = self.name {
//             model.name = ActiveValue::Set(opt.unwrap_or_default());
//         }
//         if let Some(opt) = self.upper_depth_cm {
//             model.upper_depth_cm = ActiveValue::Set(opt.unwrap_or_default());
//         }
//         if let Some(opt) = self.lower_depth_cm {
//             model.lower_depth_cm = ActiveValue::Set(opt.unwrap_or_default());
//         }
//         if let Some(opt) = self.plot_id {
//             model.plot_id = ActiveValue::Set(opt.unwrap_or_default());
//         }
//         if let Some(opt) = self.sample_weight {
//             model.sample_weight = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.subsample_weight {
//             model.subsample_weight = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.ph {
//             model.ph = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.rh {
//             model.rh = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.loi {
//             model.loi = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.mfc {
//             model.mfc = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.c {
//             model.c = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.n {
//             model.n = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.cn {
//             model.cn = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.clay_percent {
//             model.clay_percent = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.silt_percent {
//             model.silt_percent = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.sand_percent {
//             model.sand_percent = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.fe_ug_per_g {
//             model.fe_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.na_ug_per_g {
//             model.na_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.al_ug_per_g {
//             model.al_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.k_ug_per_g {
//             model.k_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.ca_ug_per_g {
//             model.ca_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.mg_ug_per_g {
//             model.mg_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.mn_ug_per_g {
//             model.mn_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.s_ug_per_g {
//             model.s_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.cl_ug_per_g {
//             model.cl_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.p_ug_per_g {
//             model.p_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.si_ug_per_g {
//             model.si_ug_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.subsample_replica_weight {
//             model.subsample_replica_weight = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.fungi_per_g {
//             model.fungi_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.bacteria_per_g {
//             model.bacteria_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.archea_per_g {
//             model.archea_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.methanogens_per_g {
//             model.methanogens_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.methanotrophs_per_g {
//             model.methanotrophs_per_g = ActiveValue::Set(opt);
//         }
//         if let Some(opt) = self.replicate {
//             model.replicate = ActiveValue::Set(opt.unwrap_or_default());
//         }
//         model.last_updated = ActiveValue::Set(chrono::Utc::now().naive_utc());
//         model
//     }
// }

// === CRUDResource Implementation for PlotSample ===
#[async_trait]
impl CRUDResource for PlotSample {
    type EntityType = crate::samples::db::Entity;
    type ColumnType = crate::samples::db::Column;
    type ModelType = crate::samples::db::Model;
    type ActiveModelType = crate::samples::db::ActiveModel;
    type ApiModel = PlotSample;
    type CreateModel = PlotSampleCreate;
    type UpdateModel = PlotSampleUpdate;

    const RESOURCE_NAME_SINGULAR: &'static str = "plotsample";
    const RESOURCE_NAME_PLURAL: &'static str = "plotsamples";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        let samples = Self::EntityType::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        let mut plot_samples: Vec<PlotSample> = Vec::new();
        for sample in samples {
            let plot_obj = crate::plots::db::Entity::find()
                .filter(crate::plots::db::Column::Id.eq(sample.plot_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;
            let area = crate::areas::db::Entity::find()
                .filter(crate::areas::db::Column::Id.eq(plot_obj.area_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Area not found".into()))?;
            let project = crate::projects::db::Entity::find()
                .filter(crate::projects::db::Column::Id.eq(area.project_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Project not found".into()))?;
            let plot = crate::plots::models::PlotBasicWithAreaAndProject {
                id: plot_obj.id,
                name: plot_obj.name,
                area: crate::areas::models::AreaBasicWithProject {
                    id: area.id,
                    name: area.name,
                    project: crate::common::crud::models::GenericNameAndID {
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
                plot,
            });
        }
        Ok(plot_samples)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let sample = Self::EntityType::find()
            .filter(crate::samples::db::Column::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot sample not found".into()))?;
        let plot_obj = crate::plots::db::Entity::find()
            .filter(crate::plots::db::Column::Id.eq(sample.plot_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;
        let area = crate::areas::db::Entity::find()
            .filter(crate::areas::db::Column::Id.eq(plot_obj.area_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Area not found".into()))?;
        let project = crate::projects::db::Entity::find()
            .filter(crate::projects::db::Column::Id.eq(area.project_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Project not found".into()))?;
        let plot = crate::plots::models::PlotBasicWithAreaAndProject {
            id: plot_obj.id,
            name: plot_obj.name,
            area: crate::areas::models::AreaBasicWithProject {
                id: area.id,
                name: area.name,
                project: crate::common::crud::models::GenericNameAndID {
                    id: project.id,
                    name: project.name,
                },
            },
        };
        Ok(PlotSample {
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
            plot,
        })
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
            .filter(crate::samples::db::Column::Id.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Plot sample not found".into()))?
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
            .filter(crate::samples::db::Column::Id.is_in(ids.clone()))
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
        crate::samples::db::Column::Id
    }

    fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", crate::samples::db::Column::Id),
            ("name", crate::samples::db::Column::Name),
        ]
    }

    fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", crate::samples::db::Column::Id),
            ("name", crate::samples::db::Column::Name),
            ("plot_id", crate::samples::db::Column::PlotId),
        ]
    }
}
