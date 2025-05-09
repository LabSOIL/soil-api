use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add nullable soil_classification_id column to plotsample
        manager
            .alter_table(
                Table::alter()
                    .table(Plotsample::Table)
                    .add_column(
                        ColumnDef::new(Plotsample::SoilClassificationId)
                            .uuid()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint to soil_classification_id
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE public.plotsample
                ADD CONSTRAINT fk_plotsample_soilclassification
                FOREIGN KEY (soil_classification_id)
                REFERENCES public.soilclassification(id);
                "#,
            )
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove foreign key constraint
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE public.plotsample
                DROP CONSTRAINT IF EXISTS fk_plotsample_soilclassification;
                "#,
            )
            .await?;

        // Drop soil_classification_id column from plotsample
        manager
            .alter_table(
                Table::alter()
                    .table(Plotsample::Table)
                    .drop_column(Plotsample::SoilClassificationId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Plotsample {
    Table,
    SoilClassificationId,
}
