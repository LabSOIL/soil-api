pub use sea_orm_migration::prelude::*;

mod m20250217_105133_first_migration;
mod m20250219_075041_sensorprofile;
mod m20250219_152310_modify_sensordata_moisturecount;
mod m20250226_092944_use_utc_timestamps;
mod m20250226_104416_remove_last_updated_sensordata;
mod m20250227_110710_remove_indexes_on_large_attributes;

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
        ]
    }
}
