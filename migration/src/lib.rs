pub use sea_orm_migration::prelude::*;

mod m20250217_105133_first_migration;
mod m20250219_075041_sensorprofile;
mod m20250219_152310_modify_sensordata_moisturecount;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250217_105133_first_migration::Migration),
            Box::new(m20250219_075041_sensorprofile::Migration),
            Box::new(m20250219_152310_modify_sensordata_moisturecount::Migration),
        ]
    }
}
