use async_trait::async_trait;
use chrono::NaiveDateTime;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::areas::models::AreaBasicWithProject;
use crate::common::crud::traits::CRUDResource;
use crate::plots::models::PlotSimple;
use crate::transects::db;
use crate::transects::nodes::models::TransectNodeAsPlotWithOrder;

// ==========================
// API Model
// ==========================
#[derive(ToSchema, Serialize)]
pub struct Transect {
    pub id: Uuid,
    pub name: Option<String>,
    pub nodes: Vec<TransectNodeAsPlotWithOrder>,
    pub area_id: Uuid,
    pub last_updated: NaiveDateTime,
    pub area: Option<AreaBasicWithProject>,
}

impl From<db::Model> for Transect {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            nodes: Vec::new(),
            area_id: model.area_id,
            last_updated: model.last_updated,
            area: None,
        }
    }
}

// ==========================
// Create & Update Models
// ==========================
#[derive(ToSchema, Serialize, Deserialize)]
pub struct TransectCreate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub area_id: Uuid,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct TransectUpdate {
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
    pub description: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub area_id: Option<Option<Uuid>>,
}

impl TransectUpdate {
    pub fn merge_into_activemodel(self, mut model: db::ActiveModel) -> db::ActiveModel {
        // Update name (note: the field is an Option<String> in the DB model)
        model.name = match self.name {
            Some(opt) => ActiveValue::Set(opt),
            None => ActiveValue::NotSet,
        };
        // Update description
        model.description = match self.description {
            Some(opt) => ActiveValue::Set(opt),
            None => ActiveValue::NotSet,
        };
        // Update area_id (remember, area_id is a Uuid so we don’t expect None)
        model.area_id = match self.area_id {
            Some(Some(val)) => ActiveValue::Set(val),
            _ => ActiveValue::NotSet,
        };
        // Update the timestamp (optional but usually desired)
        model.last_updated = ActiveValue::Set(chrono::Utc::now().naive_utc());
        model
    }
}

impl From<TransectCreate> for db::ActiveModel {
    fn from(create: TransectCreate) -> Self {
        db::ActiveModel {
            name: ActiveValue::Set(create.name),
            description: ActiveValue::Set(create.description),
            area_id: ActiveValue::Set(create.area_id),
            id: ActiveValue::Set(Uuid::new_v4()),
            date_created: ActiveValue::Set(Some(chrono::Utc::now().naive_utc())),
            last_updated: ActiveValue::Set(chrono::Utc::now().naive_utc()),
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
                let plot: PlotSimple = crate::plots::db::Entity::find()
                    .filter(crate::plots::db::Column::Id.eq(node.plot_id))
                    .column_as(Expr::cust("ST_X(plot.geom)"), "coord_x")
                    .column_as(Expr::cust("ST_Y(plot.geom)"), "coord_y")
                    .column_as(Expr::cust("ST_Z(plot.geom)"), "coord_z")
                    .column_as(
                        Expr::cust("ST_X(st_transform(plot.geom, 4326))"),
                        "longitude",
                    )
                    .column_as(
                        Expr::cust("ST_Y(st_transform(plot.geom, 4326))"),
                        "latitude",
                    )
                    .column_as(Expr::cust("st_srid(plot.geom)"), "coord_srid")
                    .into_model::<PlotSimple>()
                    .one(db)
                    .await?
                    .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;
                transect_nodes.push(TransectNodeAsPlotWithOrder {
                    id: plot.id,
                    order: node.order,
                    name: plot.name,
                    latitude: plot.latitude,
                    longitude: plot.longitude,
                    coord_srid: plot.coord_srid,
                    coord_x: plot.coord_x,
                    coord_y: plot.coord_y,
                    coord_z: plot.coord_z,
                });
            }
            // Load the associated area (with project info)
            let area = AreaBasicWithProject::from(transect_model.area_id, db.clone()).await;
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
                let plot: PlotSimple = crate::plots::db::Entity::find()
                    .filter(crate::plots::db::Column::Id.eq(node.plot_id))
                    .column_as(Expr::cust("ST_X(plot.geom)"), "coord_x")
                    .column_as(Expr::cust("ST_Y(plot.geom)"), "coord_y")
                    .column_as(Expr::cust("ST_Z(plot.geom)"), "coord_z")
                    .column_as(
                        Expr::cust("ST_X(st_transform(plot.geom, 4326))"),
                        "longitude",
                    )
                    .column_as(
                        Expr::cust("ST_Y(st_transform(plot.geom, 4326))"),
                        "latitude",
                    )
                    .column_as(Expr::cust("st_srid(plot.geom)"), "coord_srid")
                    .into_model::<PlotSimple>()
                    .one(db)
                    .await?
                    .ok_or(DbErr::RecordNotFound("Plot not found".into()))?;
                transect_nodes.push(TransectNodeAsPlotWithOrder {
                    id: plot.id,
                    order: node.order,
                    name: plot.name,
                    latitude: plot.latitude,
                    longitude: plot.longitude,
                    coord_srid: plot.coord_srid,
                    coord_x: plot.coord_x,
                    coord_y: plot.coord_y,
                    coord_z: plot.coord_z,
                });
            }
            let area = AreaBasicWithProject::from(transect_model.area_id, db.clone()).await;
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

    async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<usize, DbErr> {
        let res = Self::EntityType::delete_by_id(id).exec(db).await?;
        Ok(res.rows_affected as usize)
    }

    async fn delete_many(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Vec<Uuid>, DbErr> {
        Self::EntityType::delete_many()
            .filter(Self::ColumnType::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(ids)
    }

    async fn total_count(db: &DatabaseConnection, condition: Condition) -> u64 {
        Self::EntityType::find()
            .filter(condition)
            .select_only()
            .count(db)
            .await
            .unwrap_or(0)
    }

    fn default_index_column() -> Self::ColumnType {
        Self::ColumnType::Id
    }

    fn sortable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("last_updated", Self::ColumnType::LastUpdated),
        ]
    }

    fn filterable_columns<'a>() -> &'a [(&'a str, Self::ColumnType)] {
        &[
            ("id", Self::ColumnType::Id),
            ("name", Self::ColumnType::Name),
            ("area_id", Self::ColumnType::AreaId),
        ]
    }
}
