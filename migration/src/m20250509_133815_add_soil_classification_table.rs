use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Use raw SQL to define PostGIS geom column
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE IF NOT EXISTS public.soilclassification (
                    id uuid NOT NULL PRIMARY KEY,
                    area_id uuid NOT NULL,
                    soil_type_id uuid NOT NULL,
                    name character varying,
                    description character varying,
                    depth_upper_cm integer NOT NULL,
                    depth_lower_cm integer NOT NULL,
                    created_on TIMESTAMPTZ without time zone NOT NULL,
                    sample_date date,
                    last_updated TIMESTAMPTZ without time zone NOT NULL,
                    fe_abundance_g_per_cm3 double precision,
                    coord_x double precision,
                    coord_y double precision,
                    coord_z double precision,
                    coord_srid integer,
                    geom public.geometry(PointZ) GENERATED ALWAYS AS (
                        public.st_setsrid(public.st_makepoint(coord_x, coord_y, coord_z), coord_srid)
                    ) STORED,
                    CONSTRAINT fk_soilclassification_area FOREIGN KEY (area_id) REFERENCES public.area(id),
                    CONSTRAINT fk_soilclassification_soiltype FOREIGN KEY (soil_type_id) REFERENCES public.soiltype(id)
                );

                CREATE INDEX IF NOT EXISTS idx_soilclassification_geom ON public.soilclassification USING gist (geom);
                CREATE INDEX IF NOT EXISTS idx_soilclassification_last_updated ON public.soilclassification(last_updated);
                "#,
            )
            .await?;

        // Drop fields from plotsample
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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop soilclassification table
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS public.soilclassification;")
            .await?;

        // Re-add removed fields to plotsample
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
}

#[derive(DeriveIden)]
enum Plotsample {
    Table,
    SocStockGPerCm3,
    FeAbundanceGPerCm3,
}
