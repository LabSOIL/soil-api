use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1a. Soil VWC coefficients lookup table
        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS soil_vwc_coefficients (
                soil_type soil_type_enum PRIMARY KEY,
                a DOUBLE PRECISION NOT NULL,
                b DOUBLE PRECISION NOT NULL,
                c DOUBLE PRECISION NOT NULL
            );
            "#,
        )
        .await?;

        // 1b. Insert coefficients (idempotent)
        db.execute_unprepared(
            r#"
            INSERT INTO soil_vwc_coefficients (soil_type, a, b, c) VALUES
                ('sand',           -3.00e-09,   0.000161192, -0.1099565),
                ('loamysanda',     -1.90e-08,   0.000265610, -0.1540893),
                ('loamysandb',     -2.30e-08,   0.000282473, -0.1672112),
                ('sandyloama',     -3.80e-08,   0.000339449, -0.2149218),
                ('sandyloamb',     -9.00e-10,   0.000261847, -0.1586183),
                ('loam',           -5.10e-08,   0.000397984, -0.2910464),
                ('siltloam',        1.70e-08,   0.000118119, -0.1011685),
                ('peat',            1.23e-07,  -0.000144644,  0.2029279),
                ('water',           0.00e+00,   0.000306700, -0.1349279),
                ('universal',      -1.34e-08,   0.000249622, -0.1578888),
                ('sandtms1',        0.00e+00,   0.000260000, -0.1330400),
                ('loamysandtms1',   0.00e+00,   0.000330000, -0.1938900),
                ('siltloamtms1',    0.00e+00,   0.000380000, -0.2942700)
            ON CONFLICT (soil_type) DO UPDATE SET a=EXCLUDED.a, b=EXCLUDED.b, c=EXCLUDED.c;
            "#,
        )
        .await?;

        // 2. Precomputed averages table
        db.execute_unprepared(
            r#"
            CREATE TABLE sensorprofile_averages (
                sensorprofile_id UUID NOT NULL REFERENCES sensorprofile(id) ON DELETE CASCADE,
                depth_cm INTEGER NOT NULL,
                avg_temp DOUBLE PRECISION,
                avg_vwc DOUBLE PRECISION,
                PRIMARY KEY (sensorprofile_id, depth_cm)
            );
            "#,
        )
        .await?;

        // 3. Recompute function (reads from raw sensordata, not caggs)
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION recompute_sensor_averages(target_profile_id UUID)
            RETURNS VOID AS $$
            DECLARE
              coeff_a DOUBLE PRECISION;
              coeff_b DOUBLE PRECISION;
              coeff_c DOUBLE PRECISION;
            BEGIN
              -- Ensure coefficients table is always populated (self-healing after data restore)
              INSERT INTO soil_vwc_coefficients (soil_type, a, b, c) VALUES
                  ('sand',           -3.00e-09,   0.000161192, -0.1099565),
                  ('loamysanda',     -1.90e-08,   0.000265610, -0.1540893),
                  ('loamysandb',     -2.30e-08,   0.000282473, -0.1672112),
                  ('sandyloama',     -3.80e-08,   0.000339449, -0.2149218),
                  ('sandyloamb',     -9.00e-10,   0.000261847, -0.1586183),
                  ('loam',           -5.10e-08,   0.000397984, -0.2910464),
                  ('siltloam',        1.70e-08,   0.000118119, -0.1011685),
                  ('peat',            1.23e-07,  -0.000144644,  0.2029279),
                  ('water',           0.00e+00,   0.000306700, -0.1349279),
                  ('universal',      -1.34e-08,   0.000249622, -0.1578888),
                  ('sandtms1',        0.00e+00,   0.000260000, -0.1330400),
                  ('loamysandtms1',   0.00e+00,   0.000330000, -0.1938900),
                  ('siltloamtms1',    0.00e+00,   0.000380000, -0.2942700)
              ON CONFLICT (soil_type) DO UPDATE SET a=EXCLUDED.a, b=EXCLUDED.b, c=EXCLUDED.c;

              -- Get soil coefficients for this profile
              SELECT sc.a, sc.b, sc.c INTO coeff_a, coeff_b, coeff_c
              FROM soil_vwc_coefficients sc
              JOIN sensorprofile sp ON sp.soil_type_vwc = sc.soil_type
              WHERE sp.id = target_profile_id;

              -- Default to 'universal' if no match (e.g. NULL soil_type_vwc)
              IF coeff_a IS NULL THEN
                SELECT a, b, c INTO coeff_a, coeff_b, coeff_c
                FROM soil_vwc_coefficients WHERE soil_type = 'universal';
              END IF;

              -- Clear existing averages for this profile
              DELETE FROM sensorprofile_averages WHERE sensorprofile_id = target_profile_id;

              -- Insert temperature averages using per-depth temperatures from raw sensordata
              INSERT INTO sensorprofile_averages (sensorprofile_id, depth_cm, avg_temp)
              SELECT
                target_profile_id,
                d.depth_cm,
                AVG(CASE d.ord
                  WHEN 1 THEN sd.temperature_1
                  WHEN 2 THEN sd.temperature_2
                  WHEN 3 THEN sd.temperature_3
                END)
              FROM sensorprofile_assignment sa
              CROSS JOIN LATERAL unnest(
                ARRAY[sa.depth_cm_sensor1, sa.depth_cm_sensor2, sa.depth_cm_sensor3]
              ) WITH ORDINALITY AS d(depth_cm, ord)
              JOIN sensordata sd
                ON sd.sensor_id = sa.sensor_id
               AND sd.time_utc >= sa.date_from
               AND sd.time_utc <= sa.date_to
              WHERE sa.sensorprofile_id = target_profile_id
              GROUP BY d.depth_cm;

              -- Insert/update moisture averages (VWC formula from raw data)
              INSERT INTO sensorprofile_averages (sensorprofile_id, depth_cm, avg_vwc)
              SELECT
                target_profile_id,
                sa.depth_cm_moisture,
                AVG(
                  GREATEST(0.0::double precision, LEAST(1.0::double precision,
                    coeff_a * vwc.tcor * vwc.tcor + coeff_b * vwc.tcor + coeff_c
                  ))
                )
              FROM sensorprofile_assignment sa
              JOIN sensordata sd
                ON sd.sensor_id = sa.sensor_id
               AND sd.time_utc >= sa.date_from
               AND sd.time_utc <= sa.date_to
              CROSS JOIN LATERAL (
                SELECT sd.soil_moisture_count::double precision + (24.0 - sd.temperature_1)
                  * (1.911327 - 1.270247
                     * (coeff_a * sd.soil_moisture_count::double precision
                              * sd.soil_moisture_count::double precision
                        + coeff_b * sd.soil_moisture_count::double precision + coeff_c))
                  AS tcor
              ) vwc
              WHERE sa.sensorprofile_id = target_profile_id
                AND sa.depth_cm_moisture IS NOT NULL
              GROUP BY sa.depth_cm_moisture
              ON CONFLICT (sensorprofile_id, depth_cm) DO UPDATE
                SET avg_vwc = EXCLUDED.avg_vwc;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

        // 4. Triggers (assignment + soil-type only; no per-row sensordata trigger)

        // Trigger on sensorprofile_assignment changes
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION trigger_assignment_averages()
            RETURNS TRIGGER AS $$
            BEGIN
              IF TG_OP = 'DELETE' THEN
                PERFORM recompute_sensor_averages(OLD.sensorprofile_id);
              ELSIF TG_OP = 'UPDATE' AND OLD.sensorprofile_id IS DISTINCT FROM NEW.sensorprofile_id THEN
                PERFORM recompute_sensor_averages(OLD.sensorprofile_id);
                PERFORM recompute_sensor_averages(NEW.sensorprofile_id);
              ELSE
                PERFORM recompute_sensor_averages(NEW.sensorprofile_id);
              END IF;
              RETURN NULL;
            END;
            $$ LANGUAGE plpgsql;

            CREATE TRIGGER trg_assignment_averages
              AFTER INSERT OR UPDATE OR DELETE ON sensorprofile_assignment
              FOR EACH ROW EXECUTE FUNCTION trigger_assignment_averages();
            "#,
        )
        .await?;

        // Trigger on sensorprofile soil type changes
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION trigger_soiltype_averages()
            RETURNS TRIGGER AS $$
            BEGIN
              PERFORM recompute_sensor_averages(NEW.id);
              RETURN NULL;
            END;
            $$ LANGUAGE plpgsql;

            CREATE TRIGGER trg_soiltype_averages
              AFTER UPDATE OF soil_type_vwc ON sensorprofile
              FOR EACH ROW EXECUTE FUNCTION trigger_soiltype_averages();
            "#,
        )
        .await?;

        // 5. Backfill existing data
        db.execute_unprepared(
            r#"
            DO $$ DECLARE r RECORD;
            BEGIN
              FOR r IN SELECT id FROM sensorprofile LOOP
                PERFORM recompute_sensor_averages(r.id);
              END LOOP;
            END $$;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DROP TRIGGER IF EXISTS trg_soiltype_averages ON sensorprofile;
            DROP FUNCTION IF EXISTS trigger_soiltype_averages();

            DROP TRIGGER IF EXISTS trg_assignment_averages ON sensorprofile_assignment;
            DROP FUNCTION IF EXISTS trigger_assignment_averages();

            DROP TRIGGER IF EXISTS trg_sensordata_averages ON sensordata;
            DROP FUNCTION IF EXISTS trigger_sensordata_averages();

            DROP FUNCTION IF EXISTS recompute_sensor_averages(UUID);

            DROP TABLE IF EXISTS sensorprofile_averages;
            DROP TABLE IF EXISTS soil_vwc_coefficients;
            "#,
        )
        .await?;

        Ok(())
    }
}
