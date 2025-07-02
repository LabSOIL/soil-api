use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create the soil type enum if it doesn't exist
        // Using a different name to avoid conflict with the soiltype table
        let create_enum_sql = r#"
            DO $$ BEGIN
                CREATE TYPE soil_type_enum AS ENUM (
                    'sand',
                    'loamysanda',
                    'loamysandb',
                    'sandyloama',
                    'sandyloamb',
                    'loam',
                    'siltloam',
                    'peat',
                    'water',
                    'universal',
                    'sandtms1',
                    'loamysandtms1',
                    'siltloamtms1'
                );
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
        "#;
        db.execute_unprepared(create_enum_sql).await?;

        // Add the soil_type_vwc column to the sensorprofile table
        manager
            .alter_table(
                Table::alter()
                    .table(Sensorprofile::Table)
                    .add_column(
                        ColumnDef::new(Sensorprofile::SoilTypeVwc)
                            .custom(Alias::new("soil_type_enum"))
                            .default("universal")
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the soil_type_vwc column from the sensorprofile table
        manager
            .alter_table(
                Table::alter()
                    .table(Sensorprofile::Table)
                    .drop_column(Sensorprofile::SoilTypeVwc)
                    .to_owned(),
            )
            .await?;

        // Check if the enum is still being used by other tables/columns
        let db = manager.get_connection();
        let check_enum_usage = r#"
            SELECT COUNT(*) as usage_count
            FROM information_schema.columns 
            WHERE data_type = 'USER-DEFINED' 
            AND udt_name = 'soil_type_enum';
        "#;

        let result = db
            .query_one(sea_orm::Statement::from_string(
                manager.get_database_backend(),
                check_enum_usage.to_string(),
            ))
            .await?;

        if let Some(row) = result {
            let usage_count: i64 = row.try_get("", "usage_count")?;

            // Only drop the enum if it's not being used elsewhere
            if usage_count == 0 {
                let drop_enum_sql = "DROP TYPE IF EXISTS soil_type_enum;";
                db.execute_unprepared(drop_enum_sql).await?;
            }
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Sensorprofile {
    Table,
    SoilTypeVwc,
}
