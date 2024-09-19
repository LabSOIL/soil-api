use crate::plots::db::Entity as PlotDB;
use crate::plots::models::PlotSimple;
use crate::transects::db::Entity as TransectDB;
use crate::transects::nodes::db::Entity as TransectNodeDB;
use crate::transects::nodes::models::TransectNodeAsPlotWithOrder;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_query::Expr;
use sea_query::{Condition, Order};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize)]
pub struct Transect {
    pub id: Uuid,
    pub name: Option<String>,
    pub nodes: Vec<TransectNodeAsPlotWithOrder>,
    pub area_id: Uuid,
    pub last_updated: chrono::NaiveDateTime,
    pub area: Option<crate::areas::models::AreaBasicWithProject>,
}

impl Transect {
    pub async fn get_all(
        db: &DatabaseConnection,
        condition: Condition,
        order_column: crate::transects::db::Column,
        order_direction: Order,
        offset: u64,
        limit: u64,
    ) -> Vec<Self> {
        let transects: Vec<(
            crate::transects::db::Model,
            Vec<crate::transects::nodes::db::Model>,
        )> = crate::transects::db::Entity::find()
            .filter(condition)
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .find_with_related(crate::transects::nodes::db::Entity)
            .all(db)
            .await
            .unwrap();

        let mut transects_with_nodes: Vec<Transect> = Vec::new();
        for (transect, nodes) in transects {
            let mut transect_nodes: Vec<TransectNodeAsPlotWithOrder> = Vec::new();

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
                    .await
                    .unwrap()
                    .unwrap();

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

            let area =
                crate::areas::models::AreaBasicWithProject::from(transect.area_id, db.clone())
                    .await;

            transects_with_nodes.push(Transect {
                id: transect.id,
                name: transect.name.clone(),
                nodes: transect_nodes,
                area_id: transect.area_id,
                last_updated: transect.last_updated,
                area: Some(area),
            });
        }

        transects_with_nodes
    }

    pub async fn get_one(transect_id: Uuid, db: &DatabaseConnection) -> Option<Self> {
        // Query for the single transect with the provided id and its related transect nodes
        let transect_tuple: Option<(
            crate::transects::db::Model,
            Vec<crate::transects::nodes::db::Model>,
        )> = TransectDB::find()
            .filter(crate::transects::db::Column::Id.eq(transect_id))
            .find_with_related(TransectNodeDB)
            .all(db)
            .await
            .unwrap()
            .into_iter()
            .next(); // Retrieve the first result, if any

        // If a transect is found, process it, otherwise return None
        if let Some((transect, nodes)) = transect_tuple {
            let mut transect_nodes: Vec<TransectNodeAsPlotWithOrder> = Vec::new();

            // Iterate over nodes and fetch the associated plot for each node
            for node in nodes {
                let plot: PlotSimple = PlotDB::find()
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
                    .await
                    .unwrap()
                    .unwrap(); // Assuming plot always exists

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

            let area =
                crate::areas::models::AreaBasicWithProject::from(transect.area_id, db.clone())
                    .await;

            // Return the constructed Transect
            Some(Transect {
                id: transect.id,
                name: transect.name,
                nodes: transect_nodes,
                area_id: transect.area_id,
                last_updated: transect.last_updated,
                area: Some(area),
            })
        } else {
            // Return None if no transect is found
            None
        }
    }
}
