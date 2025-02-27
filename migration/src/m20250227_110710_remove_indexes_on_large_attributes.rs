use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP INDEX IF EXISTS idx_soiltype_image;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_soilprofile_photo;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_soilprofile_soil_diagram;")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("CREATE INDEX idx_soiltype_image ON public.soiltype(image);")
            .await?;
        db.execute_unprepared("CREATE INDEX idx_soilprofile_photo ON public.soilprofile(photo);")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX idx_soilprofile_soil_diagram ON public.soilprofile(soil_diagram);",
        )
        .await?;

        Ok(())
    }
}
