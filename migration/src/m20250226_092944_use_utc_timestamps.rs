use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // The up migration converts all timestamp columns from timestamp without time zone to TIMESTAMPTZ.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // public.project: last_updated
        db.execute_unprepared(
            "ALTER TABLE public.project ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.soiltype: last_updated
        db.execute_unprepared(
            "ALTER TABLE public.soiltype ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.area: last_updated
        db.execute_unprepared(
            "ALTER TABLE public.area ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.gnss: time and last_updated
        db.execute_unprepared(
            "ALTER TABLE public.gnss ALTER COLUMN time TYPE TIMESTAMPTZ USING time AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.gnss ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.instrumentexperiment: date and last_updated
        db.execute_unprepared(
            "ALTER TABLE public.instrumentexperiment ALTER COLUMN date TYPE TIMESTAMPTZ USING date AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.instrumentexperiment ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.plot: last_updated
        db.execute_unprepared(
            "ALTER TABLE public.plot ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.plotsample: last_updated
        db.execute_unprepared(
            "ALTER TABLE public.plotsample ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.transect: date_created and last_updated
        db.execute_unprepared(
            "ALTER TABLE public.transect ALTER COLUMN date_created TYPE TIMESTAMPTZ USING date_created AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.transect ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.sensor: last_updated
        db.execute_unprepared(
            "ALTER TABLE public.sensor ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.plotsensorassignments: date_from and date_to
        db.execute_unprepared(
            "ALTER TABLE public.plotsensorassignments ALTER COLUMN date_from TYPE TIMESTAMPTZ USING date_from AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.plotsensorassignments ALTER COLUMN date_to TYPE TIMESTAMPTZ USING date_to AT TIME ZONE 'UTC';"
        ).await?;
        // public.sensordata: time_utc and last_updated
        db.execute_unprepared(
            "ALTER TABLE public.sensordata ALTER COLUMN time_utc TYPE TIMESTAMPTZ USING time_utc AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensordata ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.soilprofile: created_on and last_updated
        db.execute_unprepared(
            "ALTER TABLE public.soilprofile ALTER COLUMN created_on TYPE TIMESTAMPTZ USING created_on AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.soilprofile ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.sensorprofile: last_updated
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        // public.sensorprofile_assignment: special handling for exclusion constraint
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment DROP CONSTRAINT IF EXISTS no_overlapping_sensor_assignments;"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ALTER COLUMN date_from TYPE TIMESTAMPTZ USING date_from AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ALTER COLUMN date_to TYPE TIMESTAMPTZ USING date_to AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ALTER COLUMN last_updated TYPE TIMESTAMPTZ USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ADD CONSTRAINT no_overlapping_sensor_assignments EXCLUDE USING gist (sensor_id WITH =, tstzrange(date_from, date_to) WITH &&);"
        ).await?;

        Ok(())
    }

    // The down migration reverts all changes back to timestamp without time zone.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Revert public.sensorprofile_assignment: drop constraint, alter columns, and re-add constraint using tsrange.
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment DROP CONSTRAINT IF EXISTS no_overlapping_sensor_assignments;"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ALTER COLUMN date_from TYPE timestamp without time zone USING date_from AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ALTER COLUMN date_to TYPE timestamp without time zone USING date_to AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile_assignment ADD CONSTRAINT no_overlapping_sensor_assignments EXCLUDE USING gist (sensor_id WITH =, tsrange(date_from, date_to) WITH &&);"
        ).await?;

        // Revert all other tables.
        db.execute_unprepared(
            "ALTER TABLE public.project ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.soiltype ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.area ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.gnss ALTER COLUMN time TYPE timestamp without time zone USING time AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.gnss ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.instrumentexperiment ALTER COLUMN date TYPE timestamp without time zone USING date AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.instrumentexperiment ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.plot ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.plotsample ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.transect ALTER COLUMN date_created TYPE timestamp without time zone USING date_created AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.transect ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensor ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.plotsensorassignments ALTER COLUMN date_from TYPE timestamp without time zone USING date_from AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.plotsensorassignments ALTER COLUMN date_to TYPE timestamp without time zone USING date_to AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensordata ALTER COLUMN time_utc TYPE timestamp without time zone USING time_utc AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensordata ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.soilprofile ALTER COLUMN created_on TYPE timestamp without time zone USING created_on AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.soilprofile ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE public.sensorprofile ALTER COLUMN last_updated TYPE timestamp without time zone USING last_updated AT TIME ZONE 'UTC';"
        ).await?;

        Ok(())
    }
}
