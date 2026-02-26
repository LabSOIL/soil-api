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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

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
