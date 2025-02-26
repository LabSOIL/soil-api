use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // In the up migration, drop the last_updated column from sensordata.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE public.sensordata DROP COLUMN IF EXISTS last_updated;")
            .await?;
        Ok(())
    }

    // In the down migration, add the last_updated column back as timestamptz NOT NULL with a default value.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE public.sensordata ADD COLUMN last_updated timestamptz NOT NULL DEFAULT now();")
            .await?;
        Ok(())
    }
}
