use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Modify soil_moisture_count column to integer.
        manager
            .alter_table(
                Table::alter()
                    .table(Sensordata::Table)
                    .modify_column(ColumnDef::new(Sensordata::SoilMoistureCount).integer())
                    .to_owned(),
            )
            .await?;

        // 2. Drop the time_zone column.
        manager
            .alter_table(
                Table::alter()
                    .table(Sensordata::Table)
                    .drop_column(Sensordata::TimeZone)
                    .to_owned(),
            )
            .await?;

        // 3. Drop the existing primary key constraint.
        // (Assumes the primary key constraint is named "sensordata_pkey")
        let drop_pk_sql = "ALTER TABLE sensordata DROP CONSTRAINT sensordata_pkey;";
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                drop_pk_sql.to_owned(),
            ))
            .await?;

        // 4. Drop the "id" column.
        manager
            .alter_table(
                Table::alter()
                    .table(Sensordata::Table)
                    .drop_column(Sensordata::Id)
                    .to_owned(),
            )
            .await?;

        // 5. Drop the unique constraint on (time_utc, sensor_id).
        let drop_unique_sql =
            "ALTER TABLE sensordata DROP CONSTRAINT IF EXISTS sensordata_time_utc_sensor_id_key;";
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                drop_unique_sql.to_owned(),
            ))
            .await?;

        // 6. Add a new composite primary key on (time_utc, sensor_id).
        let add_pk_sql = "ALTER TABLE sensordata ADD PRIMARY KEY (time_utc, sensor_id);";
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                add_pk_sql.to_owned(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Modify soil_moisture_count column back to float.
        manager
            .alter_table(
                Table::alter()
                    .table(Sensordata::Table)
                    .modify_column(ColumnDef::new(Sensordata::SoilMoistureCount).float())
                    .to_owned(),
            )
            .await?;

        // 2. Drop the composite primary key.
        let drop_pk_sql = "ALTER TABLE sensordata DROP CONSTRAINT sensordata_pkey;";
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                drop_pk_sql.to_owned(),
            ))
            .await?;

        // 3. Add back the "id" column as uuid with a default value.
        manager
            .alter_table(
                Table::alter()
                    .table(Sensordata::Table)
                    .add_column(
                        ColumnDef::new(Sensordata::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .to_owned(),
            )
            .await?;

        // 4. Set the primary key on the "id" column.
        let add_pk_sql = "ALTER TABLE sensordata ADD PRIMARY KEY (id);";
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                add_pk_sql.to_owned(),
            ))
            .await?;

        // 5. Add back the "time_zone" column.
        manager
            .alter_table(
                Table::alter()
                    .table(Sensordata::Table)
                    .add_column(ColumnDef::new(Sensordata::TimeZone).integer().null())
                    .to_owned(),
            )
            .await?;

        // 6. Recreate the unique constraint on (time_utc, sensor_id).
        let add_unique_sql = "ALTER TABLE sensordata ADD CONSTRAINT sensordata_time_utc_sensor_id_key UNIQUE (time_utc, sensor_id);";
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                add_unique_sql.to_owned(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Sensordata {
    Table,
    Id,
    TimeZone,
    TimeUtc,
    SensorId,
    SoilMoistureCount,
}
