use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // In the up() we add our new tables/constraints and remove the geom column from sensor.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ensure extension needed for the exclusion constraint exists.
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS btree_gist;")
            .await?;

        // 1. Create sensorprofile table
        let sensorprofile_table = r#"
            CREATE TABLE public.sensorprofile (
                id uuid NOT NULL,
                name character varying NOT NULL,
                description character varying,
                area_id uuid NOT NULL,
                coord_x double precision,
                coord_y double precision,
                coord_z double precision,
                coord_srid integer,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                geom public.geometry(PointZ,2056) GENERATED ALWAYS AS (
                    public.st_setsrid(public.st_makepoint(coord_x, coord_y, coord_z), coord_srid)
                ) STORED,
                CONSTRAINT sensorprofile_pkey PRIMARY KEY (id),
                CONSTRAINT sensorprofile_area_id_fkey FOREIGN KEY (area_id) REFERENCES public.area(id)
            );
            CREATE INDEX IF NOT EXISTS idx_sensorprofile_geom ON public.sensorprofile USING gist (geom);
        "#;
        db.execute_unprepared(sensorprofile_table).await?;

        // 2. Create sensorprofile_assignment join table
        let sensorprofile_assignment_table = r#"
            CREATE TABLE public.sensorprofile_assignment (
                id uuid NOT NULL,
                sensor_id uuid NOT NULL,
                sensorprofile_id uuid NOT NULL,
                date_from timestamp without time zone NOT NULL,
                date_to timestamp without time zone NOT NULL,
                last_updated timestamp without time zone DEFAULT now() NOT NULL,
                CONSTRAINT sensorprofile_assignment_pkey PRIMARY KEY (id),
                CONSTRAINT sensorprofile_assignment_sensor_id_fkey FOREIGN KEY (sensor_id) REFERENCES public.sensor(id),
                CONSTRAINT sensorprofile_assignment_sensorprofile_id_fkey FOREIGN KEY (sensorprofile_id) REFERENCES public.sensorprofile(id)
            );
            -- Prevent overlapping assignments for the same sensor:
            ALTER TABLE public.sensorprofile_assignment
              ADD CONSTRAINT no_overlapping_sensor_assignments
              EXCLUDE USING gist (
                  sensor_id WITH =,
                  tsrange(date_from, date_to) WITH &&
              );
            CREATE INDEX IF NOT EXISTS idx_sensorprofile_assignment_date_from ON public.sensorprofile_assignment(date_from);
            CREATE INDEX IF NOT EXISTS idx_sensorprofile_assignment_date_to ON public.sensorprofile_assignment(date_to);
        "#;
        db.execute_unprepared(sensorprofile_assignment_table)
            .await?;

        // 3. Remove the geom column from sensor as the location is now in sensorprofile.
        let drop_sensor_geom = r#"
            ALTER TABLE public.sensor
              DROP COLUMN IF EXISTS geom;
        "#;
        db.execute_unprepared(drop_sensor_geom).await?;

        Ok(())
    }

    // In the down() we reverse the changes.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Add back the geom column to sensor.
        // Note: This recreates the column without the original computed expression.
        let add_sensor_geom = r#"
            ALTER TABLE public.sensor
              ADD COLUMN geom public.geometry(PointZ,2056);
        "#;
        db.execute_unprepared(add_sensor_geom).await?;

        // 2. Drop the sensorprofile_assignment table.
        let drop_sensorprofile_assignment = r#"
            DROP TABLE IF EXISTS public.sensorprofile_assignment;
        "#;
        db.execute_unprepared(drop_sensorprofile_assignment).await?;

        // 3. Drop the sensorprofile table.
        let drop_sensorprofile = r#"
            DROP TABLE IF EXISTS public.sensorprofile;
        "#;
        db.execute_unprepared(drop_sensorprofile).await?;

        // (Optional) You might also drop the btree_gist extension, but it is generally left installed.
        // db.execute_unprepared("DROP EXTENSION IF EXISTS btree_gist;").await?;

        Ok(())
    }
}
