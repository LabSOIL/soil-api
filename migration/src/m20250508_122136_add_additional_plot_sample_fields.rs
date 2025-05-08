use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Plot::Table)
                    .add_column(ColumnDef::new(Plot::Slope).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Plotsample::Table)
                    .add_column(ColumnDef::new(Plotsample::SocStockGPerCm3).double().null())
                    .add_column(
                        ColumnDef::new(Plotsample::FeAbundanceGPerCm3)
                            .double()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Plot::Table)
                    .drop_column(Plot::Slope)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Plotsample::Table)
                    .drop_column(Plotsample::SocStockGPerCm3)
                    .drop_column(Plotsample::FeAbundanceGPerCm3)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Plot {
    Table,
    Slope,
}

#[derive(DeriveIden)]
enum Plotsample {
    Table,
    SocStockGPerCm3,
    FeAbundanceGPerCm3,
}
