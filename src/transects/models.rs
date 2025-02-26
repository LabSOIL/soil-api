use crate::plots::models::Plot;
use crate::transects::db;
use crate::transects::nodes::models::TransectNodeAsPlotWithOrder;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crudcrate::{CRUDResource, ToCreateModel, ToUpdateModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    Order, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, ToCreateModel, ToUpdateModel, Clone)]
#[active_model = "super::db::ActiveModel"]
pub struct Transect {
    #[crudcrate(update_model = false, create_model = false, on_create = Uuid::new_v4())]
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    #[crudcrate(update_model = false, create_model = false, on_create = chrono::Utc::now())]
    pub date_created: Option<DateTime<Utc>>,
    pub area_id: Uuid,
    #[crudcrate(update_model = false, create_model = false, on_update = chrono::Utc::now(), on_create = chrono::Utc::now())]
    pub last_updated: DateTime<Utc>,
    #[crudcrate(update_model = false, create_model = false)]
    pub area: Option<crate::areas::models::Area>,
    #[crudcrate(update_model = false, create_model = false)]
    pub nodes: Vec<TransectNodeAsPlotWithOrder>,
}

impl From<db::Model> for Transect {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            date_created: model.date_created,
            nodes: Vec::new(),
            area_id: model.area_id,
            last_updated: model.last_updated,
            area: None,
        }
    }
}

#[async_trait]
impl CRUDResource for Transect {
    type EntityType = db::Entity;
    type ColumnType = db::Column;
    type ModelType = db::Model;
    type ActiveModelType = db::ActiveModel;
    type ApiModel = Transect;
    type CreateModel = TransectCreate;
    type UpdateModel = TransectUpdate;

    const ID_COLUMN: Self::ColumnType = super::db::Column::Id;
    const RESOURCE_NAME_PLURAL: &'static str = "transects";
    const RESOURCE_NAME_SINGULAR: &'static str = "transect";

    async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: Self::ColumnType,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Self::ApiModel>, DbErr> {
        // Fetch transects along with their related transect nodes.
        let results: Vec<(db::Model, Vec<crate::transects::nodes::db::Model>)> =
            Self::EntityType::find()
                .filter(condition)
                .order_by(order_column, order_direction)
                .offset(offset)
                .limit(limit)
                .find_with_related(crate::transects::nodes::db::Entity)
                .all(db)
                .await?;
        let mut transects = Vec::new();
        for (transect_model, nodes) in results {
            let mut transect_nodes = Vec::new();
            for node in nodes {
                // Fetch the plot details for each node
                let plot: Plot = crate::plots::db::Entity::find()
                    .filter(crate::plots::db::Column::Id.eq(node.plot_id))
                    .one(db)
                    .await?
                    .ok_or(DbErr::RecordNotFound("Plot not found".into()))?
                    .into();
                transect_nodes.push(TransectNodeAsPlotWithOrder {
                    id: plot.id,
                    order: node.order,
                    name: plot.name,
                    coord_srid: plot.coord_srid,
                    coord_x: plot.coord_x,
                    coord_y: plot.coord_y,
                    coord_z: plot.coord_z,
                });
            }
            // Load the associated area (with project info)
            let area: crate::areas::models::Area = crate::areas::db::Entity::find()
                .filter(crate::areas::db::Column::Id.eq(transect_model.area_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Area not found".into()))?
                .into();

            let mut transect_api: Transect = transect_model.into();
            transect_api.nodes = transect_nodes;
            transect_api.area = Some(area);
            transects.push(transect_api);
        }
        Ok(transects)
    }

    async fn get_one(db: &DatabaseConnection, id: Uuid) -> Result<Self::ApiModel, DbErr> {
        let results: Vec<(db::Model, Vec<crate::transects::nodes::db::Model>)> =
            Self::EntityType::find()
                .filter(db::Column::Id.eq(id))
                .find_with_related(crate::transects::nodes::db::Entity)
                .all(db)
                .await?;
        if let Some((transect_model, nodes)) = results.into_iter().next() {
            let mut transect_nodes = Vec::new();
            for node in nodes {
                let plot: Plot = crate::plots::db::Entity::find()
                    .filter(crate::plots::db::Column::Id.eq(node.plot_id))
                    .one(db)
                    .await?
                    .ok_or(DbErr::RecordNotFound("Plot not found".into()))?
                    .into();
                transect_nodes.push(TransectNodeAsPlotWithOrder {
                    id: plot.id,
                    order: node.order,
                    name: plot.name,
                    coord_srid: plot.coord_srid,
                    coord_x: plot.coord_x,
                    coord_y: plot.coord_y,
                    coord_z: plot.coord_z,
                });
            }
            let area: crate::areas::models::Area = crate::areas::db::Entity::find()
                .filter(crate::areas::db::Column::Id.eq(transect_model.area_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Area not found".into()))?
                .into();

            let mut transect_api: Transect = transect_model.into();
            transect_api.nodes = transect_nodes;
            transect_api.area = Some(area);
            Ok(transect_api)
        } else {
            Err(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_SINGULAR).into(),
            ))
        }
    }

    async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        update_model: Self::UpdateModel,
    ) -> Result<Self::ApiModel, DbErr> {
        let existing: Self::ActiveModelType = Self::EntityType::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                format!("{} not found", Self::RESOURCE_NAME_PLURAL).into(),
            ))?
            .into();
        let updated_model = update_model.merge_into_activemodel(existing);
        let updated = updated_model.update(db).await?;
        Self::get_one(db, updated.id).await
    }

    fn sortable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("last_updated", Self::ColumnType::LastUpdated),
        ]
    }

    fn filterable_columns() -> Vec<(&'static str, Self::ColumnType)> {
        vec![
            ("name", Self::ColumnType::Name),
            ("description", Self::ColumnType::Description),
        ]
    }
}
