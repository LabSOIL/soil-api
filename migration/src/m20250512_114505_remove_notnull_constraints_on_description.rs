use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Soiltype::Table)
                    .modify_column(
                        ColumnDef::new(Soiltype::Description).string().null(), // Allow NULL values
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Soiltype::Table)
                    .modify_column(
                        ColumnDef::new(Soiltype::Description).string().not_null(), // Revert back to NOT NULL
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Soiltype {
    Table,
    Description,
}
