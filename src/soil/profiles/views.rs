use crate::generate_router;

generate_router!(
    resource_name: "soil_profiles",
    db_entity: crate::soil::profiles::db::Entity,
    db_model: crate::soil::profiles::db::Model,
    active_model: crate::soil::profiles::db::ActiveModel,
    db_columns: crate::soil::profiles::db::Column,
    get_one_response_model: crate::soil::profiles::models::SoilProfile,
    get_all_response_model: crate::soil::profiles::models::SoilProfileBasic,
    create_one_request_model: crate::soil::profiles::models::SoilProfileCreate,
    update_one_request_model: crate::soil::profiles::models::SoilProfileUpdate,
    order_column_logic: [
        ("id", crate::soil::profiles::db::Column::Id),
        ("name", crate::soil::profiles::db::Column::Name),
        ("gradient", crate::soil::profiles::db::Column::Gradient),
        ("description_horizon", crate::soil::profiles::db::Column::DescriptionHorizon),
        ("weather", crate::soil::profiles::db::Column::Weather),
        ("topography", crate::soil::profiles::db::Column::Topography),
        ("vegetation_type", crate::soil::profiles::db::Column::VegetationType),
        ("aspect", crate::soil::profiles::db::Column::Aspect),
        ("lythology_surficial_deposit", crate::soil::profiles::db::Column::LythologySurficialDeposit),
        ("created_on", crate::soil::profiles::db::Column::CreatedOn),
        ("parent_material", crate::soil::profiles::db::Column::ParentMaterial),
        ("last_updated", crate::soil::profiles::db::Column::LastUpdated),
    ],
    searchable_columns: [
        ("name", crate::soil::types::db::Column::Name)
    ]
);
