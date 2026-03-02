use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Hierarchical 6-hour continuous aggregate built on top of sensordata_hourly.
        // Since the hourly cagg already has pre-computed AVG/MIN/MAX per sensor,
        // we re-aggregate those into 6-hour buckets using the weighted average
        // (sample_count weights) so the result is correct even when hourly buckets
        // have different sample counts.
        db.execute_unprepared(
            r#"
            CREATE MATERIALIZED VIEW sensordata_6h
            WITH (timescaledb.continuous) AS
            SELECT
                time_bucket('6 hours', bucket) AS bucket,
                sensor_id,
                SUM(avg_temp_1 * sample_count) / NULLIF(SUM(sample_count), 0) AS avg_temp_1,
                MIN(min_temp_1) AS min_temp_1,
                MAX(max_temp_1) AS max_temp_1,
                SUM(avg_temp_2 * sample_count) / NULLIF(SUM(sample_count), 0) AS avg_temp_2,
                MIN(min_temp_2) AS min_temp_2,
                MAX(max_temp_2) AS max_temp_2,
                SUM(avg_temp_3 * sample_count) / NULLIF(SUM(sample_count), 0) AS avg_temp_3,
                MIN(min_temp_3) AS min_temp_3,
                MAX(max_temp_3) AS max_temp_3,
                SUM(avg_temp * sample_count) / NULLIF(SUM(sample_count), 0) AS avg_temp,
                MIN(min_temp) AS min_temp,
                MAX(max_temp) AS max_temp,
                SUM(avg_moisture_count * sample_count) / NULLIF(SUM(sample_count), 0) AS avg_moisture_count,
                MIN(min_moisture_count) AS min_moisture_count,
                MAX(max_moisture_count) AS max_moisture_count,
                SUM(sample_count) AS sample_count
            FROM sensordata_hourly
            GROUP BY time_bucket('6 hours', bucket), sensor_id
            WITH NO DATA;

            -- Auto-refresh policy
            SELECT add_continuous_aggregate_policy('sensordata_6h',
                start_offset => INTERVAL '18 hours',
                end_offset   => INTERVAL '6 hours',
                schedule_interval => INTERVAL '6 hours');

            -- Index for fast lookups
            CREATE INDEX ON sensordata_6h (sensor_id, bucket);
            "#,
        )
        .await?;

        // NOTE: To populate with existing data after migration, run manually:
        //   CALL refresh_continuous_aggregate('sensordata_6h', NULL, NULL);
        // This cannot run inside a transaction (which SeaORM migrations use).

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP MATERIALIZED VIEW IF EXISTS sensordata_6h CASCADE;")
            .await?;
        Ok(())
    }
}
