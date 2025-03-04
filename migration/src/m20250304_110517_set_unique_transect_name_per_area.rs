use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Alter the transect table to set "name" as NOT NULL.
        manager
            .alter_table(
                Table::alter()
                    .table(Transect::Table)
                    .modify_column(
                        ColumnDef::new(Transect::Name).string().not_null(), // Set not null constraint
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Create a unique index on (area_id, name)
        manager
            .create_index(
                Index::create()
                    .name("idx_transect_areaid_name")
                    .table(Transect::Table)
                    .col(Transect::AreaId)
                    .col(Transect::Name)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse the changes in up()

        // 1. Drop the unique index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_transect_areaid_name")
                    .table(Transect::Table)
                    .to_owned(),
            )
            .await?;

        // 2. Alter the transect table to set "name" back to nullable.
        manager
            .alter_table(
                Table::alter()
                    .table(Transect::Table)
                    .modify_column(
                        ColumnDef::new(Transect::Name).string().null(), // Remove not null constraint
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Transect {
    Table,
    Name,
    AreaId,
}
