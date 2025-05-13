use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Plotsample::Table)
                    .add_column(ColumnDef::new(Plotsample::SampledOn).date().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Plotsample::Table)
                    .drop_column(Plotsample::SampledOn)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Plotsample {
    Table,
    SampledOn,
}
