use crate::generate_router;

generate_router!(
    resource_name: "soil_types",
    db_entity: crate::soil::types::db::Entity,
    db_model: crate::soil::types::db::Model,
    db_columns: crate::soil::types::db::Column,
    get_one_response_model: crate::soil::types::models::SoilType,
    get_all_response_model: crate::soil::types::models::SoilTypeBasic,
    order_column_logic: [
        ("id", crate::soil::types::db::Column::Id),
        ("name", crate::soil::types::db::Column::Name),
        ("description", crate::soil::types::db::Column::Description),
        ("last_updated", crate::soil::types::db::Column::LastUpdated),
    ],
    searchable_columns: [
        ("name", crate::soil::types::db::Column::Name),
        ("description", crate::soil::types::db::Column::Description),
    ]
);
