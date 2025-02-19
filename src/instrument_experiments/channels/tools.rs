use serde_json::json;
use std::cmp::Ordering;

/// Calculate a spline for the given x and y data based on selected baseline points.
/// This implementation uses linear interpolation between the baseline points.
///
/// # Arguments
/// - `x`: Slice of x values.
/// - `y`: Slice of y values.
/// - `baseline_selected_points`: Slice of x-values chosen as baseline points.
/// - `interpolation_method`: Interpolation method (currently only "linear" is supported).
///
/// # Returns
/// A `Vec<f64>` containing the interpolated spline values for each x.
pub fn calculate_spline(
    x: &[f64],
    y: &[f64],
    baseline_selected_points: &[f64],
    interpolation_method: &str,
) -> Vec<f64> {
    // Build pairs (baseline point, corresponding y value)
    let pairs: Vec<(f64, f64)> = baseline_selected_points
        .iter()
        .filter_map(|&bp| {
            x.iter()
                .position(|&xi| (xi - bp).abs() < 1e-6)
                .map(|i| (bp, y[i]))
        })
        .collect();

    if pairs.is_empty() {
        return vec![0.0; x.len()];
    }

    // For now we only support "linear" interpolation.
    if interpolation_method != "linear" {
        panic!(
            "Interpolation method {} not supported, only 'linear' is available",
            interpolation_method
        );
    }

    let mut spline = Vec::with_capacity(x.len());
    for &xi in x {
        let yi = if xi <= pairs.first().unwrap().0 {
            pairs.first().unwrap().1
        } else if xi >= pairs.last().unwrap().0 {
            pairs.last().unwrap().1
        } else {
            // Find two consecutive pairs where xi fits.
            let mut interp = pairs.first().unwrap().1;
            for window in pairs.windows(2) {
                if xi >= window[0].0 && xi <= window[1].0 {
                    let (x0, y0) = window[0];
                    let (x1, y1) = window[1];
                    let t = (xi - x0) / (x1 - x0);
                    interp = y0 + t * (y1 - y0);
                    break;
                }
            }
            interp
        };
        spline.push(yi);
    }
    spline
}

/// Compute the filtered baseline by subtracting the spline from the original y values.
///
/// # Arguments
/// - `y`: Slice of original y values.
/// - `spline`: Slice of spline values (must be the same length as `y`).
///
/// # Returns
/// A `Vec<f64>` containing the baseline-filtered values.
pub fn filter_baseline(y: &[f64], spline: &[f64]) -> Vec<f64> {
    y.iter().zip(spline.iter()).map(|(a, b)| a - b).collect()
}

/// Downsample data using the Largest Triangle Three Buckets (LTTB) algorithm.
///
/// # Arguments
/// - `x`: Slice of x values.
/// - `y`: Slice of y values.
/// - `threshold`: The desired number of points after downsampling.
///
/// # Returns
/// A tuple `(Vec<f64>, Vec<f64>)` containing the downsampled x and y values.
pub fn largest_triangle_three_buckets(
    x: &[f64],
    y: &[f64],
    threshold: usize,
) -> (Vec<f64>, Vec<f64>) {
    let n = x.len();
    if n <= threshold || threshold == 0 {
        return (x.to_vec(), y.to_vec());
    }
    let bucket_size = (n - 2) as f64 / (threshold - 2) as f64;
    let mut sampled_x = Vec::with_capacity(threshold);
    let mut sampled_y = Vec::with_capacity(threshold);
    sampled_x.push(x[0]);
    sampled_y.push(y[0]);

    let mut a = 0; // Index of previously selected point.
    for i in 1..(threshold - 1) {
        let bucket_start = ((i - 1) as f64 * bucket_size + 1.0).floor() as usize;
        let bucket_end = ((i as f64 * bucket_size + 1.0).floor() as usize).min(n);
        let next_bucket_start = (i as f64 * bucket_size + 1.0).floor() as usize;
        let next_bucket_end = (((i + 1) as f64 * bucket_size + 1.0).floor() as usize).min(n);

        let (avg_x, avg_y) = if next_bucket_end > next_bucket_start {
            let count = (next_bucket_end - next_bucket_start) as f64;
            let sum_x: f64 = x[next_bucket_start..next_bucket_end].iter().sum();
            let sum_y: f64 = y[next_bucket_start..next_bucket_end].iter().sum();
            (sum_x / count, sum_y / count)
        } else {
            (x[next_bucket_start], y[next_bucket_start])
        };

        let mut max_area = -1.0;
        let mut max_index = bucket_start;
        let point_a_x = x[a];
        let point_a_y = y[a];
        for j in bucket_start..bucket_end {
            let area = ((point_a_x - avg_x) * (y[j] - point_a_y)
                - (point_a_x - x[j]) * (avg_y - point_a_y))
                .abs();
            if area > max_area {
                max_area = area;
                max_index = j;
            }
        }
        sampled_x.push(x[max_index]);
        sampled_y.push(y[max_index]);
        a = max_index;
    }
    sampled_x.push(x[n - 1]);
    sampled_y.push(y[n - 1]);
    (sampled_x, sampled_y)
}

/// Integrate the given data using the trapezoidal rule.
///
/// # Arguments
/// - `x`: Slice of x values.
/// - `y`: Slice of y values (must be the same length as `x`).
///
/// # Returns
/// The computed integral as an `f64`.
pub fn integrate_trapz(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..(n - 1) {
        let dx = x[i + 1] - x[i];
        let avg_y = (y[i] + y[i + 1]) / 2.0;
        area += dx * avg_y;
    }
    area
}

/// Calculate the integral for a given range using the specified integration method.
/// Currently, only "trapz" (trapezoidal) and "simpson" (which falls back to trapezoidal) are supported.
///
/// # Arguments
/// - `x`: Slice of x values.
/// - `y`: Slice of y values.
/// - `integration_method`: A string specifying the method ("trapz" or "simpson").
///
/// # Returns
/// The computed integral as an `f64`.
pub fn calculate_integral_for_range(x: &[f64], y: &[f64], integration_method: &str) -> f64 {
    match integration_method {
        "trapz" => integrate_trapz(x, y),
        "simpson" => integrate_trapz(x, y), // Placeholder for Simpson's rule
        _ => panic!("Integration method {} not supported", integration_method),
    }
}

/// Calculate the integral for each pair in the provided list.
/// Each pair is expected to be a JSON object with the structure:
/// { "start": {"x": value}, "end": {"x": value}, "sample_name": "..." }
///
/// # Arguments
/// - `pairs`: A slice of JSON values representing the pairs.
/// - `baseline_values`: Slice of baseline y values.
/// - `time_values`: Slice of time x values.
/// - `integration_method`: Integration method to use ("trapz" or "simpson").
///
/// # Returns
/// A vector of JSON objects, each containing "start", "end", "area", and "sample_name".
pub fn calculate_integrals_for_pairs(
    pairs: &[serde_json::Value],
    baseline_values: &[f64],
    time_values: &[f64],
    integration_method: &str,
) -> Vec<serde_json::Value> {
    let mut integration_results = Vec::new();

    for pair in pairs {
        let start = pair
            .get("start")
            .and_then(|v| v.get("x"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let end = pair
            .get("end")
            .and_then(|v| v.get("x"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let start_index = time_values.iter().position(|&v| (v - start).abs() < 1e-6);
        let end_index = time_values.iter().position(|&v| (v - end).abs() < 1e-6);

        if let (Some(si), Some(ei)) = (start_index, end_index) {
            let x_slice = &time_values[si..=ei];
            let y_slice = &baseline_values[si..=ei];
            let area = calculate_integral_for_range(x_slice, y_slice, integration_method);
            let sample_name = pair
                .get("sample_name")
                .and_then(|v| v.as_str())
                .unwrap_or("undefined")
                .to_string();
            let result = json!({
                "start": start,
                "end": end,
                "area": area,
                "sample_name": sample_name,
            });
            integration_results.push(result);
        }
    }

    integration_results.sort_by(|a, b| {
        let a_start = a.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b_start = b.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
        a_start.partial_cmp(&b_start).unwrap_or(Ordering::Equal)
    });

    integration_results
}
