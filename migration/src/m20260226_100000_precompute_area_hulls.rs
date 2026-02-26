use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            -- 1. Add hull_geom column (nullable GEOMETRY stored in SRID 4326)
            ALTER TABLE area ADD COLUMN hull_geom GEOMETRY(Geometry, 4326);

            -- 2. PL/pgSQL function to recompute hull for a given area
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

            -- 3. Trigger function (routes to recompute_area_hull for affected area_id)
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

            -- 4. Triggers on the three child tables
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

            -- 5. Backfill existing areas
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

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

        Ok(())
    }
}
