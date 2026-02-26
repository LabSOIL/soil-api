use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            -- 1. profile_type enum
            DO $$ BEGIN
                CREATE TYPE profile_type_enum AS ENUM ('tms', 'chamber', 'redox');
            EXCEPTION WHEN duplicate_object THEN null;
            END $$;

            -- 2. New columns on sensorprofile (all idempotent)
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

            -- 3. flux_data table (includes raw_readings from the start)
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
            -- If table existed without raw_readings (from old migration 20)
            DO $$ BEGIN ALTER TABLE flux_data ADD COLUMN raw_readings JSONB;
            EXCEPTION WHEN duplicate_column THEN NULL; END $$;

            -- 4. redox_data table
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

            -- 5. Website visibility tables
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

        // TimescaleDB: hypertable, continuous aggregates, compression
        db.execute_unprepared(
            r#"
            -- Enable TimescaleDB
            CREATE EXTENSION IF NOT EXISTS timescaledb;

            -- Drop unique constraint incompatible with hypertables
            -- (doesn't include the time dimension column)
            ALTER TABLE sensordata
              DROP CONSTRAINT IF EXISTS sensordata_instrument_seq_sensor_id_key;

            -- Drop redundant single-column indexes
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

            -- Hourly continuous aggregate
            CREATE MATERIALIZED VIEW sensordata_hourly
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('1 hour', time_utc) AS bucket,
                sensor_id,
                AVG(temperature_1) AS avg_temp_1,
                AVG(temperature_2) AS avg_temp_2,
                AVG(temperature_3) AS avg_temp_3,
                AVG(temperature_average) AS avg_temp,
                AVG(soil_moisture_count::double precision) AS avg_moisture_count,
                COUNT(*) AS sample_count
            FROM sensordata
            GROUP BY time_bucket('1 hour', time_utc), sensor_id
            WITH NO DATA;

            -- Daily continuous aggregate
            CREATE MATERIALIZED VIEW sensordata_daily
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('1 day', time_utc) AS bucket,
                sensor_id,
                AVG(temperature_1) AS avg_temp_1,
                AVG(temperature_2) AS avg_temp_2,
                AVG(temperature_3) AS avg_temp_3,
                AVG(temperature_average) AS avg_temp,
                AVG(soil_moisture_count::double precision) AS avg_moisture_count,
                COUNT(*) AS sample_count
            FROM sensordata
            GROUP BY time_bucket('1 day', time_utc), sensor_id
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

            -- Compression (data older than 30 days)
            ALTER TABLE sensordata SET (
                timescaledb.compress,
                timescaledb.compress_segmentby = 'sensor_id'
            );
            SELECT add_compression_policy('sensordata', INTERVAL '30 days');
            "#,
        )
        .await?;

        // NOTE: To populate aggregates with existing data after a DB restore, run manually:
        //   CALL refresh_continuous_aggregate('sensordata_hourly', NULL, NULL);
        //   CALL refresh_continuous_aggregate('sensordata_daily', NULL, NULL);
        // These cannot run inside a transaction (which SeaORM migrations use).
        // The auto-refresh policies will populate new data on schedule.

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop TimescaleDB objects first (before dropping tables they depend on)
        db.execute_unprepared(
            r#"
            DROP MATERIALIZED VIEW IF EXISTS sensordata_daily CASCADE;
            DROP MATERIALIZED VIEW IF EXISTS sensordata_hourly CASCADE;
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            -- 1. Drop website visibility tables
            DROP TABLE IF EXISTS website_sensor_exclusion;
            DROP TABLE IF EXISTS website_plot_exclusion;
            DROP TABLE IF EXISTS area_website;
            DROP TABLE IF EXISTS website;

            -- 2. Drop flux_data and redox_data
            DROP TABLE IF EXISTS flux_data;
            DROP TABLE IF EXISTS redox_data;

            -- 3. Drop new columns from sensorprofile
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS position;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS chamber_id_external;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS instrument_model;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS area_cm2;
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS volume_ml;

            -- 4. Re-add NOT NULL to soil_type_vwc
            ALTER TABLE sensorprofile ALTER COLUMN soil_type_vwc SET NOT NULL;

            -- 5. Drop profile_type column and enum
            ALTER TABLE sensorprofile DROP COLUMN IF EXISTS profile_type;
            DROP TYPE IF EXISTS profile_type_enum;
            "#,
        )
        .await?;

        Ok(())
    }
}
