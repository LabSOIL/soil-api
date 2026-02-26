use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Profile types, new columns, flux/redox/website tables
        db.execute_unprepared(
            r#"
            -- profile_type enum
            DO $$ BEGIN
                CREATE TYPE profile_type_enum AS ENUM ('tms', 'chamber', 'redox');
            EXCEPTION WHEN duplicate_object THEN null;
            END $$;

            -- New columns on sensorprofile
            DO $$ BEGIN ALTER TABLE sensorprofile ADD COLUMN profile_type profile_type_enum DEFAULT 'tms' NOT NULL;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;
            ALTER TABLE sensorprofile ALTER COLUMN soil_type_vwc DROP NOT NULL;
            DO $$ BEGIN ALTER TABLE sensorprofile ADD COLUMN volume_ml DOUBLE PRECISION;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;
            DO $$ BEGIN ALTER TABLE sensorprofile ADD COLUMN area_cm2 DOUBLE PRECISION;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;
            DO $$ BEGIN ALTER TABLE sensorprofile ADD COLUMN instrument_model VARCHAR;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;
            DO $$ BEGIN ALTER TABLE sensorprofile ADD COLUMN chamber_id_external VARCHAR;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;
            DO $$ BEGIN ALTER TABLE sensorprofile ADD COLUMN position INTEGER;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;

            -- flux_data table
            CREATE TABLE IF NOT EXISTS flux_data (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                sensorprofile_id UUID NOT NULL REFERENCES sensorprofile(id) ON DELETE CASCADE,
                measured_on TIMESTAMPTZ NOT NULL,
                replicate VARCHAR NOT NULL,
                setting VARCHAR,
                flux_co2_umol_m2_s DOUBLE PRECISION,
                flux_ch4_nmol_m2_s DOUBLE PRECISION,
                flux_h2o_umol_m2_s DOUBLE PRECISION,
                r2_co2 DOUBLE PRECISION,
                r2_ch4 DOUBLE PRECISION,
                r2_h2o DOUBLE PRECISION,
                swc DOUBLE PRECISION,
                n_measurements INTEGER,
                raw_readings JSONB,
                UNIQUE(sensorprofile_id, measured_on, replicate)
            );
            DO $$ BEGIN ALTER TABLE flux_data ADD COLUMN raw_readings JSONB;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;

            -- redox_data table
            CREATE TABLE IF NOT EXISTS redox_data (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                sensorprofile_id UUID NOT NULL REFERENCES sensorprofile(id) ON DELETE CASCADE,
                measured_on TIMESTAMPTZ NOT NULL,
                ch1_5cm_mv DOUBLE PRECISION,
                ch2_15cm_mv DOUBLE PRECISION,
                ch3_25cm_mv DOUBLE PRECISION,
                ch4_35cm_mv DOUBLE PRECISION,
                temp_c DOUBLE PRECISION,
                UNIQUE(sensorprofile_id, measured_on)
            );

            -- Website visibility tables
            CREATE TABLE IF NOT EXISTS website (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR NOT NULL,
                slug VARCHAR NOT NULL UNIQUE,
                url VARCHAR,
                description TEXT
            );
            CREATE TABLE IF NOT EXISTS area_website (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                area_id UUID NOT NULL REFERENCES area(id) ON DELETE CASCADE,
                website_id UUID NOT NULL REFERENCES website(id) ON DELETE CASCADE,
                date_from TIMESTAMPTZ,
                date_to TIMESTAMPTZ,
                UNIQUE(area_id, website_id)
            );
            CREATE TABLE IF NOT EXISTS website_plot_exclusion (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                website_id UUID NOT NULL REFERENCES website(id) ON DELETE CASCADE,
                plot_id UUID NOT NULL REFERENCES plot(id) ON DELETE CASCADE,
                UNIQUE(website_id, plot_id)
            );
            CREATE TABLE IF NOT EXISTS website_sensor_exclusion (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                website_id UUID NOT NULL REFERENCES website(id) ON DELETE CASCADE,
                sensorprofile_id UUID NOT NULL REFERENCES sensorprofile(id) ON DELETE CASCADE,
                UNIQUE(website_id, sensorprofile_id)
            );
            "#,
        )
        .await?;

        // 2. TimescaleDB: hypertable, continuous aggregates (with min/max), compression
        db.execute_unprepared(
            r#"
            CREATE EXTENSION IF NOT EXISTS timescaledb;

            -- Drop constraints/indexes incompatible with hypertables
            ALTER TABLE sensordata
              DROP CONSTRAINT IF EXISTS sensordata_instrument_seq_sensor_id_key;

            DROP INDEX IF EXISTS idx_sensordata_time_utc;
            DROP INDEX IF EXISTS idx_sensordata_instrument_seq;
            DROP INDEX IF EXISTS idx_sensordata_error_flat;
            DROP INDEX IF EXISTS idx_sensordata_shake;
            DROP INDEX IF EXISTS idx_sensordata_soil_moisture_count;
            DROP INDEX IF EXISTS idx_sensordata_temperature_1;
            DROP INDEX IF EXISTS idx_sensordata_temperature_2;
            DROP INDEX IF EXISTS idx_sensordata_temperature_3;
            DROP INDEX IF EXISTS idx_sensordata_temperature_average;

            -- Convert to hypertable (7-day chunks)
            SELECT create_hypertable('sensordata', 'time_utc',
                chunk_time_interval => INTERVAL '7 days',
                migrate_data => true);

            -- Covering index for fast sensor+time range lookups
            CREATE INDEX idx_sensordata_sensor_time_cover
            ON sensordata (sensor_id, time_utc DESC)
            INCLUDE (temperature_average, soil_moisture_count, temperature_1);

            -- Hourly continuous aggregate (with min/max)
            CREATE MATERIALIZED VIEW sensordata_hourly
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('1 hour', time_utc) AS bucket,
                sensor_id,
                AVG(temperature_1) AS avg_temp_1,
                MIN(temperature_1) AS min_temp_1,
                MAX(temperature_1) AS max_temp_1,
                AVG(temperature_2) AS avg_temp_2,
                MIN(temperature_2) AS min_temp_2,
                MAX(temperature_2) AS max_temp_2,
                AVG(temperature_3) AS avg_temp_3,
                MIN(temperature_3) AS min_temp_3,
                MAX(temperature_3) AS max_temp_3,
                AVG(temperature_average) AS avg_temp,
                MIN(temperature_average) AS min_temp,
                MAX(temperature_average) AS max_temp,
                AVG(soil_moisture_count::double precision) AS avg_moisture_count,
                MIN(soil_moisture_count) AS min_moisture_count,
                MAX(soil_moisture_count) AS max_moisture_count,
                COUNT(*) AS sample_count
            FROM sensordata
            GROUP BY time_bucket('1 hour', time_utc), sensor_id
            WITH NO DATA;

            -- Daily continuous aggregate (with min/max)
            CREATE MATERIALIZED VIEW sensordata_daily
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('1 day', time_utc) AS bucket,
                sensor_id,
                AVG(temperature_1) AS avg_temp_1,
                MIN(temperature_1) AS min_temp_1,
                MAX(temperature_1) AS max_temp_1,
                AVG(temperature_2) AS avg_temp_2,
                MIN(temperature_2) AS min_temp_2,
                MAX(temperature_2) AS max_temp_2,
                AVG(temperature_3) AS avg_temp_3,
                MIN(temperature_3) AS min_temp_3,
                MAX(temperature_3) AS max_temp_3,
                AVG(temperature_average) AS avg_temp,
                MIN(temperature_average) AS min_temp,
                MAX(temperature_average) AS max_temp,
                AVG(soil_moisture_count::double precision) AS avg_moisture_count,
                MIN(soil_moisture_count) AS min_moisture_count,
                MAX(soil_moisture_count) AS max_moisture_count,
                COUNT(*) AS sample_count
            FROM sensordata
            GROUP BY time_bucket('1 day', time_utc), sensor_id
            WITH NO DATA;

            -- Weekly continuous aggregate (with min/max)
            CREATE MATERIALIZED VIEW sensordata_weekly
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('1 week', time_utc) AS bucket,
                sensor_id,
                AVG(temperature_1) AS avg_temp_1,
                MIN(temperature_1) AS min_temp_1,
                MAX(temperature_1) AS max_temp_1,
                AVG(temperature_2) AS avg_temp_2,
                MIN(temperature_2) AS min_temp_2,
                MAX(temperature_2) AS max_temp_2,
                AVG(temperature_3) AS avg_temp_3,
                MIN(temperature_3) AS min_temp_3,
                MAX(temperature_3) AS max_temp_3,
                AVG(temperature_average) AS avg_temp,
                MIN(temperature_average) AS min_temp,
                MAX(temperature_average) AS max_temp,
                AVG(soil_moisture_count::double precision) AS avg_moisture_count,
                MIN(soil_moisture_count) AS min_moisture_count,
                MAX(soil_moisture_count) AS max_moisture_count,
                COUNT(*) AS sample_count
            FROM sensordata
            GROUP BY time_bucket('1 week', time_utc), sensor_id
            WITH NO DATA;

            -- Auto-refresh policies
            SELECT add_continuous_aggregate_policy('sensordata_hourly',
                start_offset => INTERVAL '3 hours',
                end_offset => INTERVAL '1 hour',
                schedule_interval => INTERVAL '1 hour');

            SELECT add_continuous_aggregate_policy('sensordata_daily',
                start_offset => INTERVAL '3 days',
                end_offset => INTERVAL '1 day',
                schedule_interval => INTERVAL '1 day');

            SELECT add_continuous_aggregate_policy('sensordata_weekly',
                start_offset => INTERVAL '3 weeks',
                end_offset => INTERVAL '1 week',
                schedule_interval => INTERVAL '1 week');

            -- Compression (data older than 30 days)
            ALTER TABLE sensordata SET (
                timescaledb.compress,
                timescaledb.compress_segmentby = 'sensor_id'
            );
            SELECT add_compression_policy('sensordata', INTERVAL '30 days');
            "#,
        )
        .await?;

        // 3. Area hull precomputation
        db.execute_unprepared(
            r#"
            -- hull_geom column
            ALTER TABLE area ADD COLUMN hull_geom GEOMETRY(Geometry, 4326);

            -- PL/pgSQL function to recompute hull for a given area
            CREATE OR REPLACE FUNCTION recompute_area_hull(target_area_id UUID)
            RETURNS VOID AS $$
            DECLARE
              collected GEOMETRY;
            BEGIN
              SELECT ST_Collect(geom_2056) INTO collected
              FROM (
                SELECT ST_Transform(geom, 2056) AS geom_2056
                  FROM plot WHERE area_id = target_area_id AND geom IS NOT NULL
                UNION ALL
                SELECT ST_Transform(geom, 2056) AS geom_2056
                  FROM soilprofile WHERE area_id = target_area_id AND geom IS NOT NULL
                UNION ALL
                SELECT ST_Transform(geom, 2056) AS geom_2056
                  FROM sensorprofile WHERE area_id = target_area_id AND geom IS NOT NULL
              ) AS pts;

              IF collected IS NULL THEN
                UPDATE area SET hull_geom = NULL WHERE id = target_area_id;
              ELSE
                UPDATE area SET hull_geom = ST_Transform(
                  ST_Buffer(ST_ConvexHull(collected), 10), 4326
                ) WHERE id = target_area_id;
              END IF;
            END;
            $$ LANGUAGE plpgsql;

            -- Trigger function
            CREATE OR REPLACE FUNCTION trigger_recompute_area_hull()
            RETURNS TRIGGER AS $$
            BEGIN
              IF TG_OP = 'DELETE' THEN
                PERFORM recompute_area_hull(OLD.area_id);
              ELSIF TG_OP = 'UPDATE' AND OLD.area_id IS DISTINCT FROM NEW.area_id THEN
                PERFORM recompute_area_hull(OLD.area_id);
                PERFORM recompute_area_hull(NEW.area_id);
              ELSE
                PERFORM recompute_area_hull(NEW.area_id);
              END IF;
              RETURN NULL;
            END;
            $$ LANGUAGE plpgsql;

            -- Triggers on child tables
            CREATE TRIGGER trg_plot_hull
              AFTER INSERT OR UPDATE OF coord_x, coord_y, coord_z, coord_srid, area_id
                 OR DELETE ON plot
              FOR EACH ROW EXECUTE FUNCTION trigger_recompute_area_hull();

            CREATE TRIGGER trg_soilprofile_hull
              AFTER INSERT OR UPDATE OF coord_x, coord_y, coord_z, coord_srid, area_id
                 OR DELETE ON soilprofile
              FOR EACH ROW EXECUTE FUNCTION trigger_recompute_area_hull();

            CREATE TRIGGER trg_sensorprofile_hull
              AFTER INSERT OR UPDATE OF coord_x, coord_y, coord_z, coord_srid, area_id
                 OR DELETE ON sensorprofile
              FOR EACH ROW EXECUTE FUNCTION trigger_recompute_area_hull();

            -- Backfill existing areas
            DO $$
            DECLARE r RECORD;
            BEGIN
              FOR r IN SELECT id FROM area LOOP
                PERFORM recompute_area_hull(r.id);
              END LOOP;
            END $$;
            "#,
        )
        .await?;

        // NOTE: To populate aggregates with existing data after migration, run manually:
        //   CALL refresh_continuous_aggregate('sensordata_hourly', NULL, NULL);
        //   CALL refresh_continuous_aggregate('sensordata_daily', NULL, NULL);
        //   CALL refresh_continuous_aggregate('sensordata_weekly', NULL, NULL);
        // These cannot run inside a transaction (which SeaORM migrations use).

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop hull triggers/functions
        db.execute_unprepared(
            r#"
            DROP TRIGGER IF EXISTS trg_plot_hull ON plot;
            DROP TRIGGER IF EXISTS trg_soilprofile_hull ON soilprofile;
            DROP TRIGGER IF EXISTS trg_sensorprofile_hull ON sensorprofile;
            DROP FUNCTION IF EXISTS trigger_recompute_area_hull();
            DROP FUNCTION IF EXISTS recompute_area_hull(UUID);
            ALTER TABLE area DROP COLUMN IF EXISTS hull_geom;
            "#,
        )
        .await?;

        // Drop TimescaleDB objects
        db.execute_unprepared(
            r#"
            DROP MATERIALIZED VIEW IF EXISTS sensordata_weekly CASCADE;
            DROP MATERIALIZED VIEW IF EXISTS sensordata_daily CASCADE;
            DROP MATERIALIZED VIEW IF EXISTS sensordata_hourly CASCADE;
            "#,
        )
        .await?;

        // Drop tables and columns
        db.execute_unprepared(
            r#"
            DROP TABLE IF EXISTS website_sensor_exclusion;
            DROP TABLE IF EXISTS website_plot_exclusion;
            DROP TABLE IF EXISTS area_website;
            DROP TABLE IF EXISTS website;
            DROP TABLE IF EXISTS flux_data;
            DROP TABLE IF EXISTS redox_data;

            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS position;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS chamber_id_external;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS instrument_model;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS area_cm2;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS volume_ml;
            ALTER TABLE sensorprofile ALTER COLUMN soil_type_vwc SET NOT NULL;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS profile_type;
            DROP TYPE IF EXISTS profile_type_enum;
            "#,
        )
        .await?;

        Ok(())
    }
}
