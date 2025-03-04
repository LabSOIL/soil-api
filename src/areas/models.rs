use super::db::Model;
use crate::projects::db::Entity as ProjectDB;
use crate::projects::models::Project;
use crate::soil::profiles::models::SoilProfile;
use crate::transects::models::Transect;
use crate::{plots::models::Plot, sensors::profile::models::SensorProfile};
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    ActiveValue, Condition, DatabaseConnection, EntityTrait, Order, QueryOrder, entity::prelude::*,
    query::QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, ToCreateModel, ToUpdateModel, Deserialize, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct Area {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now(), on_create = chrono::Utc::now())]
    pub last_updated: DateTime<Utc>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_id: Uuid,
    #[crudcrate(update_model = false, create_model = false)]
    pub project: Option<Project>,
    #[crudcrate(update_model = false, create_model = false)]
    pub soil_profiles: Vec<SoilProfile>,
    #[crudcrate(update_model = false, create_model = false)]
    pub plots: Vec<Plot>,
    #[crudcrate(update_model = false, create_model = false)]
    pub sensor_profiles: Vec<SensorProfile>,
    #[crudcrate(update_model = false, create_model = false)]
    pub transects: Vec<Transect>,
    #[crudcrate(update_model = false, create_model = false)]
    pub geom: Option<Value>,
}

impl From<Model> for Area {
    fn from(model: Model) -> Self {
        Area {
            id: model.id,
            last_updated: model.last_updated,
            name: model.name,
            description: model.description,
            project_id: model.project_id,
            project: None,
            soil_profiles: vec![],
            plots: vec![],
            sensor_profiles: vec![],
            transects: vec![],
            geom: None,
        }
    }
}

#[async_trait::async_trait]
impl CRUDResource for Area {
    type EntityType = super::db::Entity;
    type ColumnType = super::db::Column;
    type ModelType = super::db::Model;
    type ActiveModelType = super::db::ActiveModel;
    type ApiModel = Area;
    type CreateModel = AreaCreate;
    type UpdateModel = AreaUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_PLURAL: &'static str = "areas";
    const RESOURCE_NAME_SINGULAR: &'static str = "area";

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
        let mut areas = Vec::new();
        for model in models {
            let project = model.find_related(ProjectDB).one(db).await?.unwrap();

            let plots = model.find_related(crate::plots::db::Entity).all(db).await?;

            let sensor_profiles = model
                .find_related(crate::sensors::profile::db::Entity)
                .all(db)
                .await?;

            let soil_profiles = model
                .find_related(crate::soil::profiles::db::Entity)
                .all(db)
                .await?;

            let transects = model
                .find_related(crate::transects::db::Entity)
                .all(db)
                .await?;
            let mut transect_objs: Vec<crate::transects::models::Transect> = vec![];

            for transect in &transects {
                let node_objs = transect
                    .find_related(crate::transects::nodes::db::Entity)
                    .all(db)
                    .await?;
                let mut nodes = vec![];

                for node in node_objs {
                    let plot: Plot = crate::plots::db::Entity::find()
                        .filter(crate::plots::db::Column::Id.eq(node.plot_id))
                        .one(db)
                        .await?
                        .ok_or(DbErr::RecordNotFound("Plot not found".into()))?
                        .into();
                    let transect_node = crate::transects::nodes::models::TransectNode {
                        // id: node.id,
                        plot: Some(plot),
                        order: node.order,
                        // transect_id: node.transect_id,
                        plot_id: node.plot_id,
                    };
                    nodes.push(transect_node);
                }

                transect_objs.push(crate::transects::models::Transect {
                    id: transect.id,
                    name: transect.name.clone(),
                    description: transect.description.clone(),
                    date_created: transect.date_created,
                    last_updated: transect.last_updated,
                    area: None,
                    area_id: transect.area_id,
                    nodes,
                });
            }

            let convex_hull = super::services::get_convex_hull(db, model.id).await;

            let area = Area {
                geom: convex_hull,
                last_updated: model.last_updated,
                project_id: model.project_id,
                id: model.id,
                name: model.name,
                description: model.description,
                project: Some(project.into()),
                plots: plots.into_iter().map(Into::into).collect(),
                sensor_profiles: sensor_profiles.into_iter().map(Into::into).collect(),
                soil_profiles: soil_profiles.into_iter().map(Into::into).collect(),
                transects: transect_objs,
            };
            areas.push(area);
        }
        Ok(areas)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let model = Self::EntityType::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Area not found".into()))?;

        let project = model.find_related(ProjectDB).one(db).await?.unwrap();

        let plots = model.find_related(crate::plots::db::Entity).all(db).await?;

        let sensor_profiles = model
            .find_related(crate::sensors::profile::db::Entity)
            .all(db)
            .await?;

        let soil_profiles = model
            .find_related(crate::soil::profiles::db::Entity)
            .all(db)
            .await?;

        let transects = model
            .find_related(crate::transects::db::Entity)
            .all(db)
            .await?;
        let mut transect_objs: Vec<crate::transects::models::Transect> = vec![];

        for transect in &transects {
            let node_objs = transect
                .find_related(crate::transects::nodes::db::Entity)
                .all(db)
                .await?;
            let mut nodes = vec![];

            for node in node_objs {
                let plot: Plot = crate::plots::db::Entity::find()
                    .filter(crate::plots::db::Column::Id.eq(node.plot_id))
                    .one(db)
                    .await?
                    .ok_or(DbErr::RecordNotFound("Plot not found".into()))?
                    .into();
                let transect_node = crate::transects::nodes::models::TransectNode {
                    // id: node.id,
                    plot: Some(plot),
                    order: node.order,
                    // transect_id: node.transect_id,
                    plot_id: node.plot_id,
                };
                nodes.push(transect_node);
            }

            transect_objs.push(crate::transects::models::Transect {
                id: transect.id,
                name: transect.name.clone(),
                description: transect.description.clone(),
                date_created: transect.date_created,
                last_updated: transect.last_updated,
                area: None,
                area_id: transect.area_id,
                nodes,
            });
        }

        let convex_hull = super::services::get_convex_hull(db, model.id).await;

        let area = Area {
            geom: convex_hull,
            last_updated: model.last_updated,
            project_id: model.project_id,
            id: model.id,
            name: model.name,
            description: model.description,
            project: Some(project.into()),
            plots: plots.into_iter().map(Into::into).collect(),
            sensor_profiles: sensor_profiles.into_iter().map(Into::into).collect(),
            soil_profiles: soil_profiles.into_iter().map(Into::into).collect(),
            transects: transect_objs,
        };
        Ok(area)
    }

    async fn create(
        db: &DatabaseConnection,
        create_model: Self::CreateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let active_model: Self::ActiveModelType = create_model.into();
        let inserted = active_model.insert(db).await?;
        let area = Self::get_one(db, inserted.id).await.unwrap();
        Ok(area)
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Area not found".into()))?
            .into();

        let updated_obj: super::db::ActiveModel = update_model.merge_into_activemodel(db_obj);
        let response_obj = updated_obj.update(db).await?;
        let area = Self::get_one(db, response_obj.id).await?;
        Ok(area)
    }

    fn default_index_column() -> Self::ColumnType {
        super::db::Column::Id
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", super::db::Column::Name),
            ("description", super::db::Column::Description),
        ]
    }
}
