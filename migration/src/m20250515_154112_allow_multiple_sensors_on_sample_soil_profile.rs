use sea_orm_migration::{prelude::*, sea_orm::Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the constraint on time range
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"
                    ALTER TABLE sensorprofile_assignment
                    DROP CONSTRAINT no_overlapping_sensor_assignments
                    "#
                .to_string(),
            ))
            .await?;

        // Add new fields: depth_cm_sensor_1, depth_cm_sensor_2, depth_cm_sensor_3
        manager
            .alter_table(
                Table::alter()
                    .table(SensorprofileAssignment::Table)
                    .add_column(ColumnDef::new(SensorprofileAssignment::DepthCmSensor1).integer())
                    .add_column(ColumnDef::new(SensorprofileAssignment::DepthCmSensor2).integer())
                    .add_column(ColumnDef::new(SensorprofileAssignment::DepthCmSensor3).integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"
                    ALTER TABLE sensorprofile_assignment
                    ADD CONSTRAINT no_overlapping_sensor_assignments
                    EXCLUDE USING gist (
                        sensor_id WITH =,
                        tstzrange(date_from, date_to) WITH &&
                    )
                    "#
                .to_string(),
            ))
            .await?;

        // Remove the new fields: depth_cm_sensor_1, depth_cm_sensor_2, depth_cm_sensor_3
        manager
            .alter_table(
                Table::alter()
                    .table(SensorprofileAssignment::Table)
                    .drop_column(SensorprofileAssignment::DepthCmSensor1)
                    .drop_column(SensorprofileAssignment::DepthCmSensor2)
                    .drop_column(SensorprofileAssignment::DepthCmSensor3)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SensorprofileAssignment {
    Table,
    DepthCmSensor1,
    DepthCmSensor2,
    DepthCmSensor3,
}
