pub use sea_orm_migration::prelude::*;

mod m20250217_105133_first_migration;
mod m20250219_075041_sensorprofile;
mod m20250219_152310_modify_sensordata_moisturecount;
mod m20250226_092944_use_utc_timestamps;
mod m20250226_104416_remove_last_updated_sensordata;
mod m20250227_110710_remove_indexes_on_large_attributes;
mod m20250303_101534_remove_areaid_from_sensors;
mod m20250304_110517_set_unique_transect_name_per_area;
mod m20250307_124139_plot_remove_notnull_gradient;
mod m20250507_101706_add_ispublic_flag_to_areas;
mod m20250508_122136_add_additional_plot_sample_fields;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250217_105133_first_migration::Migration),
            Box::new(m20250219_075041_sensorprofile::Migration),
            Box::new(m20250219_152310_modify_sensordata_moisturecount::Migration),
            Box::new(m20250226_092944_use_utc_timestamps::Migration),
            Box::new(m20250226_104416_remove_last_updated_sensordata::Migration),
            Box::new(m20250227_110710_remove_indexes_on_large_attributes::Migration),
            Box::new(m20250303_101534_remove_areaid_from_sensors::Migration),
            Box::new(m20250304_110517_set_unique_transect_name_per_area::Migration),
            Box::new(m20250307_124139_plot_remove_notnull_gradient::Migration),
            Box::new(m20250507_101706_add_ispublic_flag_to_areas::Migration),
            Box::new(m20250508_122136_add_additional_plot_sample_fields::Migration),
        ]
    }
}
