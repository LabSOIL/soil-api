use sea_orm_migration::{prelude::*, sea_orm::Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1) Add the new column as nullable
        manager
            .alter_table(
                Table::alter()
                    .table(SensorprofileAssignment::Table)
                    .add_column(ColumnDef::new(SensorprofileAssignment::DepthCmMoisture).integer())
                    .to_owned(),
            )
            .await?;

        // 2) Backfill existing rows from depth_cm_sensor1
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "UPDATE sensorprofile_assignment \
                SET depth_cm_moisture = depth_cm_sensor1"
                    .to_string(),
            ))
            .await?;

        // 3) Make the new column non-nullable and sensor columns non-nullable
        manager
            .alter_table(
                Table::alter()
                    .table(SensorprofileAssignment::Table)
                    .modify_column(
                        ColumnDef::new(SensorprofileAssignment::DepthCmMoisture)
                            .integer()
                            .not_null(),
                    )
                    .modify_column(
                        ColumnDef::new(SensorprofileAssignment::DepthCmSensor1)
                            .integer()
                            .not_null(),
                    )
                    .modify_column(
                        ColumnDef::new(SensorprofileAssignment::DepthCmSensor2)
                            .integer()
                            .not_null(),
                    )
                    .modify_column(
                        ColumnDef::new(SensorprofileAssignment::DepthCmSensor3)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert: drop the moisture column and allow sensor depths to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(SensorprofileAssignment::Table)
                    .drop_column(SensorprofileAssignment::DepthCmMoisture)
                    .modify_column(
                        ColumnDef::new(SensorprofileAssignment::DepthCmSensor1)
                            .integer()
                            .null(),
                    )
                    .modify_column(
                        ColumnDef::new(SensorprofileAssignment::DepthCmSensor2)
                            .integer()
                            .null(),
                    )
                    .modify_column(
                        ColumnDef::new(SensorprofileAssignment::DepthCmSensor3)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SensorprofileAssignment {
    Table,
    DepthCmMoisture,
    DepthCmSensor1,
    DepthCmSensor2,
    DepthCmSensor3,
}
