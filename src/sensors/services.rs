use base64::{engine::general_purpose, Engine as _};
use chrono::NaiveDateTime;
use lttb::lttb;
use lttb::DataPoint;
use std::str;
use uuid::Uuid;

pub fn simplify_sensor_data_lttb(
    data: Vec<crate::sensors::data::db::Model>,
    target_points: usize,
) -> Vec<crate::sensors::data::db::Model> {
    let len = data.len();

    if len <= target_points {
        return data;
    }

    // Convert the sensor data timestamps and temperature_1 values to DataPoint structs
    let timestamps: Vec<DataPoint> = data
        .iter()
        .enumerate()
        .map(|(i, d)| DataPoint {
            x: i as f64, // or use d.time_utc.and_utc().timestamp() as f64
            y: d.temperature_1,
        })
        .collect();

    // Downsample the data using the LTTB algorithm
    let downsampled_points = lttb(timestamps, target_points);

    // Now map the downsampled points back to your original data
    let mut downsampled_data = vec![];

    for point in downsampled_points {
        let original = &data[point.x as usize]; // Map x value back to original index
        let simplified = crate::sensors::data::db::Model {
            temperature_1: point.y,
            ..original.clone()
        };
        downsampled_data.push(simplified);
    }

    downsampled_data
}

// Helper function: decode base64 and return (raw_bytes, file_type)
fn decode_base64(value: &str) -> Result<(Vec<u8>, String), String> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() < 2 {
        return Err("Invalid base64 format".into());
    }
    let meta = parts[0];
    let data_part = parts[1];
    let file_type = if meta.contains("text/csv") {
        "csv".to_string()
    } else if meta.contains("gpx") {
        "gpx".to_string()
    } else if meta.contains("text/plain") {
        "txt".to_string()
    } else {
        return Err("Only CSV, TXT, and GPX files are supported".into());
    };
    let decoded = general_purpose::STANDARD
        .decode(data_part)
        .map_err(|e| e.to_string())?;
    Ok((decoded, file_type))
}

// Helper function: ingest CSV sensor data and create SensorData objects
fn ingest_csv_data(
    sensor_data: &[u8],
    sensor_id: Uuid,
) -> Result<Vec<crate::sensors::data::models::SensorData>, String> {
    let data_str = str::from_utf8(sensor_data).map_err(|_| "Invalid UTF-8 sequence")?;
    let mut objs = Vec::new();
    for line in data_str.lines() {
        if !line.trim().is_empty() {
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() < 9 {
                continue; // Skip malformed lines
            }
            let instrument_seq = parts[0].parse::<i32>().unwrap_or(0);
            // println!("Date: {}", parts[1]);
            let time_utc = NaiveDateTime::parse_from_str(parts[1], "%Y.%m.%d %H:%M")
                .map_err(|_| "Invalid date format")?;
            let temperature_1 = parts[3].parse::<f64>().unwrap_or(0.0);
            let temperature_2 = parts[4].parse::<f64>().unwrap_or(0.0);
            let temperature_3 = parts[5].parse::<f64>().unwrap_or(0.0);
            let temperature_average = (temperature_1 + temperature_2 + temperature_3) / 3.0;
            let soil_moisture_count = parts[6].parse::<i32>().unwrap_or(0);
            let shake = parts[7].parse::<i32>().unwrap_or(0);
            let error_flat = parts[8].parse::<i32>().unwrap_or(0);

            let sensor_data_obj = crate::sensors::data::models::SensorData {
                instrument_seq,
                time_utc,
                temperature_1,
                temperature_2,
                temperature_3,
                temperature_average,
                soil_moisture_count,
                shake,
                error_flat,
                sensor_id,
                last_updated: chrono::Utc::now().naive_utc(),
            };
            objs.push(sensor_data_obj);
        }
    }
    Ok(objs)
}

// New helper function: process the base64 CSV sensor data and return SensorData models.
pub fn process_sensor_data_base64(
    data_base64: &str,
    sensor_id: Uuid,
) -> Result<Vec<crate::sensors::data::models::SensorData>, String> {
    let (raw_data, file_type) = decode_base64(data_base64)?;
    if file_type != "csv" {
        return Err("Only CSV files are supported".into());
    }
    let data_objs = ingest_csv_data(&raw_data, sensor_id)?;
    Ok(data_objs)
}
