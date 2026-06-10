use std::collections::HashSet;
use rand::Rng;
use rand::thread_rng;

#[derive(Debug, Clone)]
pub struct FractalResult {
    pub dimension: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
    pub confidence_level: f64,
    pub r_squared: f64,
    pub data_quality: f64,
    pub method: String,
    pub num_valid_scales: usize,
}

#[derive(Debug, Clone)]
pub struct MultiFractalResult {
    pub box_counting: FractalResult,
    pub perimeter_area: FractalResult,
    pub divider: FractalResult,
    pub weighted_average: f64,
    pub agreement_score: f64,
    pub overall_quality: f64,
}

impl FractalResult {
    pub fn invalid(method: &str) -> Self {
        FractalResult {
            dimension: 0.0,
            confidence_lower: 0.0,
            confidence_upper: 0.0,
            confidence_level: 0.95,
            r_squared: 0.0,
            data_quality: 0.0,
            method: method.to_string(),
            num_valid_scales: 0,
        }
    }
}

pub fn boundary_fractal_dimension_robust(
    polygon_points: &[(f64, f64)],
    num_scales: usize,
) -> MultiFractalResult {
    let data_quality = assess_data_quality(polygon_points);
    
    let box_counting = box_counting_dimension(polygon_points, num_scales, data_quality);
    let perimeter_area = perimeter_area_dimension(polygon_points);
    let divider = divider_dimension(polygon_points, num_scales);

    let mut valid_dims = Vec::new();
    let mut weights = Vec::new();

    if box_counting.data_quality > 0.3 {
        valid_dims.push(box_counting.dimension);
        weights.push(box_counting.data_quality * box_counting.r_squared);
    }
    if perimeter_area.data_quality > 0.3 {
        valid_dims.push(perimeter_area.dimension);
        weights.push(perimeter_area.data_quality * perimeter_area.r_squared);
    }
    if divider.data_quality > 0.3 {
        valid_dims.push(divider.dimension);
        weights.push(divider.data_quality * divider.r_squared);
    }

    let weighted_average = if weights.is_empty() {
        0.0
    } else {
        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            valid_dims.iter().sum::<f64>() / valid_dims.len() as f64
        } else {
            valid_dims.iter().zip(weights.iter())
                .map(|(d, w)| d * w)
                .sum::<f64>() / total_weight
        }
    };

    let agreement_score = if valid_dims.len() >= 2 {
        let mean = valid_dims.iter().sum::<f64>() / valid_dims.len() as f64;
        let variance: f64 = valid_dims.iter()
            .map(|d| (d - mean).powi(2))
            .sum::<f64>() / valid_dims.len() as f64;
        let std_dev = variance.sqrt();
        (1.0 - (std_dev / mean.max(0.01)).min(1.0)).max(0.0)
    } else {
        0.5
    };

    let overall_quality = (data_quality * 0.4 + agreement_score * 0.3 + 
        (box_counting.r_squared + perimeter_area.r_squared + divider.r_squared) / 3.0 * 0.3);

    MultiFractalResult {
        box_counting,
        perimeter_area,
        divider,
        weighted_average,
        agreement_score,
        overall_quality,
    }
}

pub fn network_fractal_dimension_robust(
    line_segments: &[((f64, f64), (f64, f64))],
    num_scales: usize,
) -> MultiFractalResult {
    let data_quality = if line_segments.is_empty() {
        0.0
    } else {
        (line_segments.len() as f64 / 20.0).min(1.0)
    };

    let box_counting = network_box_counting_dimension(line_segments, num_scales, data_quality);
    let perimeter_area = FractalResult::invalid("perimeter_area");
    let divider = network_divider_dimension(line_segments, num_scales);

    let mut valid_dims = Vec::new();
    let mut weights = Vec::new();

    if box_counting.data_quality > 0.3 {
        valid_dims.push(box_counting.dimension);
        weights.push(box_counting.data_quality * box_counting.r_squared);
    }
    if divider.data_quality > 0.3 {
        valid_dims.push(divider.dimension);
        weights.push(divider.data_quality * divider.r_squared);
    }

    let weighted_average = if weights.is_empty() {
        0.0
    } else {
        let total_weight: f64 = weights.iter().sum();
        if total_weight <= 0.0 {
            valid_dims.iter().sum::<f64>() / valid_dims.len() as f64
        } else {
            valid_dims.iter().zip(weights.iter())
                .map(|(d, w)| d * w)
                .sum::<f64>() / total_weight
        }
    };

    let agreement_score = if valid_dims.len() >= 2 {
        let mean = valid_dims.iter().sum::<f64>() / valid_dims.len() as f64;
        let variance: f64 = valid_dims.iter()
            .map(|d| (d - mean).powi(2))
            .sum::<f64>() / valid_dims.len() as f64;
        let std_dev = variance.sqrt();
        (1.0 - (std_dev / mean.max(0.01)).min(1.0)).max(0.0)
    } else {
        0.5
    };

    let overall_quality = (data_quality * 0.5 + agreement_score * 0.2 + 
        (box_counting.r_squared + divider.r_squared) / 2.0 * 0.3);

    MultiFractalResult {
        box_counting,
        perimeter_area,
        divider,
        weighted_average,
        agreement_score,
        overall_quality,
    }
}

fn assess_data_quality(points: &[(f64, f64)]) -> f64 {
    if points.len() < 4 {
        return 0.1;
    }

    let mut quality = 1.0;

    if points.len() < 10 {
        quality *= 0.5 + 0.05 * points.len() as f64;
    }

    let mut total_len = 0.0;
    let mut seglen_count = 0;
    for i in 0..points.len() {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % points.len()];
        let len = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        total_len += len;
        seglen_count += 1;
    }

    if seglen_count > 0 {
        let avg_len = total_len / seglen_count as f64;
        let mut variance = 0.0;
        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            let len = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            variance += (len - avg_len).powi(2);
        }
        let std_dev = (variance / seglen_count as f64).sqrt();
        let cv = if avg_len > 0.0 { std_dev / avg_len } else { 1.0 };
        if cv > 2.0 {
            quality *= 0.7;
        }
    }

    let (min_x, max_x, min_y, max_y) = bounding_box(points);
    let width = max_x - min_x;
    let height = max_y - min_y;
    if width <= 0.0 || height <= 0.0 {
        return 0.1;
    }

    let is_closed = {
        let (sx, sy) = points[0];
        let (ex, ey) = points[points.len() - 1];
        let dist = ((sx - ex).powi(2) + (sy - ey).powi(2)).sqrt();
        dist < (width.max(height)) * 0.01
    };
    if !is_closed {
        quality *= 0.6;
    }

    quality.clamp(0.0, 1.0)
}

fn box_counting_dimension(
    polygon_points: &[(f64, f64)],
    num_scales: usize,
    data_quality: f64,
) -> FractalResult {
    if polygon_points.len() < 3 {
        return FractalResult::invalid("box_counting");
    }

    let (min_x, max_x, min_y, max_y) = bounding_box(polygon_points);
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;
    let max_size = total_width.max(total_height);

    if max_size <= 0.0 {
        return FractalResult::invalid("box_counting");
    }

    let mut log_scales = Vec::new();
    let mut log_counts = Vec::new();

    for i in 0..num_scales {
        let scale = max_size / (2.0_f64).powi(i as i32 + 1);
        if scale <= 0.0 {
            continue;
        }
        let count = count_boundary_boxes_ransac(polygon_points, min_x, min_y, scale);
        if count > 0 {
            log_scales.push(scale.ln());
            log_counts.push(count as f64);
        }
    }

    if log_scales.len() < 3 {
        let mut res = FractalResult::invalid("box_counting");
        res.data_quality = data_quality * 0.3;
        return res;
    }

    let (slope, r_squared, ci_lower, ci_upper) = robust_linear_regression(&log_scales, &log_counts);

    FractalResult {
        dimension: slope.abs(),
        confidence_lower: ci_lower.abs(),
        confidence_upper: ci_upper.abs(),
        confidence_level: 0.95,
        r_squared,
        data_quality,
        method: "box_counting".to_string(),
        num_valid_scales: log_scales.len(),
    }
}

fn network_box_counting_dimension(
    segments: &[((f64, f64), (f64, f64))],
    num_scales: usize,
    data_quality: f64,
) -> FractalResult {
    if segments.is_empty() {
        return FractalResult::invalid("box_counting");
    }

    let (min_x, max_x, min_y, max_y) = segments_bounding_box(segments);
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;
    let max_size = total_width.max(total_height);

    if max_size <= 0.0 {
        return FractalResult::invalid("box_counting");
    }

    let mut log_scales = Vec::new();
    let mut log_counts = Vec::new();

    for i in 0..num_scales {
        let scale = max_size / (2.0_f64).powi(i as i32 + 1);
        if scale <= 0.0 {
            continue;
        }
        let count = count_network_boxes(segments, min_x, min_y, scale);
        if count > 0 {
            log_scales.push(scale.ln());
            log_counts.push(count as f64);
        }
    }

    if log_scales.len() < 3 {
        let mut res = FractalResult::invalid("box_counting");
        res.data_quality = data_quality * 0.3;
        return res;
    }

    let (slope, r_squared, ci_lower, ci_upper) = robust_linear_regression(&log_scales, &log_counts);

    FractalResult {
        dimension: slope.abs(),
        confidence_lower: ci_lower.abs(),
        confidence_upper: ci_upper.abs(),
        confidence_level: 0.95,
        r_squared,
        data_quality,
        method: "box_counting".to_string(),
        num_valid_scales: log_scales.len(),
    }
}

fn perimeter_area_dimension(points: &[(f64, f64)]) -> FractalResult {
    if points.len() < 4 {
        return FractalResult::invalid("perimeter_area");
    }

    let quality = assess_data_quality(points);
    if quality < 0.2 {
        let mut res = FractalResult::invalid("perimeter_area");
        res.data_quality = quality;
        return res;
    }

    let area = polygon_area(points);
    let perimeter = polygon_perimeter(points);

    if area <= 0.0 || perimeter <= 0.0 {
        let mut res = FractalResult::invalid("perimeter_area");
        res.data_quality = quality * 0.5;
        return res;
    }

    let dimension = 2.0 * perimeter.ln() / (4.0 * std::f64::consts::PI * area).ln();

    let r_squared = if dimension >= 1.0 && dimension <= 2.0 {
        0.7 + 0.3 * quality
    } else {
        0.3 * quality
    };

    let ci_range = 0.15 / quality.max(0.3);

    FractalResult {
        dimension: dimension.clamp(1.0, 2.0),
        confidence_lower: (dimension - ci_range).max(1.0),
        confidence_upper: (dimension + ci_range).min(2.0),
        confidence_level: 0.95,
        r_squared,
        data_quality: quality,
        method: "perimeter_area".to_string(),
        num_valid_scales: 1,
    }
}

fn divider_dimension(points: &[(f64, f64)], num_scales: usize) -> FractalResult {
    if points.len() < 4 {
        return FractalResult::invalid("divider");
    }

    let quality = assess_data_quality(points);
    if quality < 0.2 {
        let mut res = FractalResult::invalid("divider");
        res.data_quality = quality;
        return res;
    }

    let (min_x, max_x, min_y, max_y) = bounding_box(points);
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;
    let max_size = total_width.max(total_height);

    if max_size <= 0.0 {
        return FractalResult::invalid("divider");
    }

    let mut log_steps = Vec::new();
    let mut log_lengths = Vec::new();

    for i in 0..num_scales.min(6) {
        let step_size = max_size / (2.0_f64).powi(i as i32 + 2);
        if step_size <= 0.0 {
            continue;
        }
        let (length, _count) = divider_walk(points, step_size);
        if length > 0.0 {
            log_steps.push(step_size.ln());
            log_lengths.push(length.ln());
        }
    }

    if log_steps.len() < 3 {
        let mut res = FractalResult::invalid("divider");
        res.data_quality = quality * 0.4;
        return res;
    }

    let (slope, r_squared, ci_lower, ci_upper) = robust_linear_regression(&log_steps, &log_lengths);
    let dimension = 1.0 - slope;

    FractalResult {
        dimension: dimension.clamp(1.0, 2.0),
        confidence_lower: (1.0 - ci_upper).clamp(1.0, 2.0),
        confidence_upper: (1.0 - ci_lower).clamp(1.0, 2.0),
        confidence_level: 0.95,
        r_squared,
        data_quality: quality,
        method: "divider".to_string(),
        num_valid_scales: log_steps.len(),
    }
}

fn network_divider_dimension(
    segments: &[((f64, f64), (f64, f64))],
    num_scales: usize,
) -> FractalResult {
    if segments.len() < 3 {
        return FractalResult::invalid("divider");
    }

    let quality = (segments.len() as f64 / 10.0).min(1.0);
    if quality < 0.2 {
        let mut res = FractalResult::invalid("divider");
        res.data_quality = quality;
        return res;
    }

    let (min_x, max_x, min_y, max_y) = segments_bounding_box(segments);
    let max_size = (max_x - min_x).max(max_y - min_y);

    if max_size <= 0.0 {
        return FractalResult::invalid("divider");
    }

    let mut total_len = 0.0;
    for &((x1, y1), (x2, y2)) in segments {
        total_len += ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    }

    let mut log_steps = Vec::new();
    let mut log_counts = Vec::new();

    for i in 0..num_scales.min(6) {
        let step_size = max_size / (2.0_f64).powi(i as i32 + 2);
        if step_size <= 0.0 {
            continue;
        }
        let count = (total_len / step_size).ceil();
        if count > 1.0 {
            log_steps.push(step_size.ln());
            log_counts.push(count.ln());
        }
    }

    if log_steps.len() < 3 {
        let mut res = FractalResult::invalid("divider");
        res.data_quality = quality * 0.4;
        return res;
    }

    let (slope, r_squared, ci_lower, ci_upper) = robust_linear_regression(&log_steps, &log_counts);

    FractalResult {
        dimension: (1.0 - slope).clamp(1.0, 2.0),
        confidence_lower: (1.0 - ci_upper).clamp(1.0, 2.0),
        confidence_upper: (1.0 - ci_lower).clamp(1.0, 2.0),
        confidence_level: 0.95,
        r_squared,
        data_quality: quality,
        method: "divider".to_string(),
        num_valid_scales: log_steps.len(),
    }
}

fn divider_walk(points: &[(f64, f64)], step_size: f64) -> (f64, usize) {
    let n = points.len();
    if n < 2 || step_size <= 0.0 {
        return (0.0, 0);
    }

    let mut total_length = 0.0;
    let mut steps = 0;
    let mut remaining = step_size;
    let mut seg_idx = 0;
    let mut seg_progress = 0.0;

    let max_iter = n * 1000;
    let mut iter = 0;

    while iter < max_iter {
        iter += 1;
        let (x1, y1) = points[seg_idx];
        let (x2, y2) = points[(seg_idx + 1) % n];
        let seg_len = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

        if seg_len <= 0.0 {
            seg_idx = (seg_idx + 1) % n;
            seg_progress = 0.0;
            if seg_idx == 0 {
                break;
            }
            continue;
        }

        let remaining_on_seg = seg_len - seg_progress;

        if remaining < remaining_on_seg {
            total_length += remaining;
            seg_progress += remaining;
            steps += 1;
            remaining = step_size;
        } else {
            total_length += remaining_on_seg;
            remaining -= remaining_on_seg;
            seg_idx = (seg_idx + 1) % n;
            seg_progress = 0.0;
            if seg_idx == 0 {
                break;
            }
        }
    }

    (total_length, steps)
}

fn robust_linear_regression(
    x: &[f64],
    y: &[f64],
) -> (f64, f64, f64, f64) {
    let n = x.len();
    if n < 3 {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let (base_slope, _intercept, base_r2) = ordinary_least_squares(x, y);

    let mut rng = thread_rng();
    let num_bootstrap = if n < 10 { 100 } else { 500 };
    let mut slopes = Vec::with_capacity(num_bootstrap);

    for _ in 0..num_bootstrap {
        let mut sample_x = Vec::with_capacity(n);
        let mut sample_y = Vec::with_capacity(n);
        for _ in 0..n {
            let idx: usize = rng.gen_range(0..n);
            sample_x.push(x[idx]);
            sample_y.push(y[idx]);
        }
        let (s, _, _) = ordinary_least_squares(&sample_x, &sample_y);
        if s.is_finite() {
            slopes.push(s);
        }
    }

    if slopes.is_empty() {
        return (base_slope, base_r2, base_slope - 0.1, base_slope + 0.1);
    }

    slopes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let ci_lower_idx = ((slopes.len() as f64) * 0.025) as usize;
    let ci_upper_idx = ((slopes.len() as f64) * 0.975) as usize;
    let ci_lower = slopes[ci_lower_idx.min(slopes.len() - 1)];
    let ci_upper = slopes[ci_upper_idx.min(slopes.len() - 1)];

    (base_slope, base_r2, ci_lower, ci_upper)
}

fn ordinary_least_squares(x: &[f64], y: &[f64]) -> (f64, f64, f64) {
    let n = x.len() as f64;
    if n < 2.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();
    let sum_y2: f64 = y.iter().map(|yi| yi * yi).sum();

    let denominator = n * sum_x2 - sum_x * sum_x;
    if denominator.abs() < 1e-10 {
        return (0.0, sum_y / n, 0.0);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept = (sum_y - slope * sum_x) / n;

    let ss_tot = sum_y2 - sum_y * sum_y / n;
    let ss_res = sum_y2 - slope * sum_xy - intercept * sum_y;
    
    let r_squared = if ss_tot.abs() < 1e-10 {
        1.0
    } else {
        (1.0 - ss_res / ss_tot).max(0.0)
    };

    (slope, intercept, r_squared)
}

fn bounding_box(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for &(x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    (min_x, max_x, min_y, max_y)
}

fn count_boundary_boxes_ransac(
    points: &[(f64, f64)],
    min_x: f64,
    min_y: f64,
    grid_size: f64,
) -> usize {
    let mut boxes = HashSet::new();

    for i in 0..points.len() {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % points.len()];

        let dx = x2 - x1;
        let dy = y2 - y1;
        let seg_len = (dx * dx + dy * dy).sqrt();
        
        if seg_len <= 0.0 {
            continue;
        }

        let cells = line_grid_cells(x1, y1, x2, y2, min_x, min_y, grid_size);
        for cell in cells {
            boxes.insert(cell);
        }
    }

    boxes.len()
}

fn line_grid_cells(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    min_x: f64,
    min_y: f64,
    grid_size: f64,
) -> Vec<(i64, i64)> {
    let mut cells = Vec::new();

    let gx1 = ((x1 - min_x) / grid_size).floor() as i64;
    let gy1 = ((y1 - min_y) / grid_size).floor() as i64;
    let gx2 = ((x2 - min_x) / grid_size).floor() as i64;
    let gy2 = ((y2 - min_y) / grid_size).floor() as i64;

    cells.push((gx1, gy1));

    if gx1 == gx2 && gy1 == gy2 {
        return cells;
    }

    let dx = x2 - x1;
    let dy = y2 - y1;
    let steps = ((gx2 - gx1).abs()).max((gy2 - gy1).abs());

    if steps == 0 {
        return cells;
    }

    let step_x = dx / steps as f64;
    let step_y = dy / steps as f64;

    for i in 1..=steps {
        let x = x1 + step_x * i as f64;
        let y = y1 + step_y * i as f64;
        let gx = ((x - min_x) / grid_size).floor() as i64;
        let gy = ((y - min_y) / grid_size).floor() as i64;
        if !cells.contains(&(gx, gy)) {
            cells.push((gx, gy));
        }
    }

    cells
}

fn count_network_boxes(
    segments: &[((f64, f64), (f64, f64))],
    min_x: f64,
    min_y: f64,
    grid_size: f64,
) -> usize {
    let mut boxes = HashSet::new();

    for &((x1, y1), (x2, y2)) in segments {
        let cells = line_grid_cells(x1, y1, x2, y2, min_x, min_y, grid_size);
        for cell in cells {
            boxes.insert(cell);
        }
    }

    boxes.len()
}

fn segments_bounding_box(segments: &[((f64, f64), (f64, f64))]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for &((x1, y1), (x2, y2)) in segments {
        min_x = min_x.min(x1).min(x2);
        max_x = max_x.max(x1).max(x2);
        min_y = min_y.min(y1).min(y2);
        max_y = max_y.max(y1).max(y2);
    }

    (min_x, max_x, min_y, max_y)
}

pub fn polygon_area(points: &[(f64, f64)]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].0 * points[j].1;
        area -= points[j].0 * points[i].1;
    }

    area.abs() / 2.0
}

pub fn polygon_perimeter(points: &[(f64, f64)]) -> f64 {
    let n = points.len();
    if n < 2 {
        return 0.0;
    }

    let mut perimeter = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = points[j].0 - points[i].0;
        let dy = points[j].1 - points[i].1;
        perimeter += (dx * dx + dy * dy).sqrt();
    }

    perimeter
}

pub fn compactness_index(area: f64, perimeter: f64) -> f64 {
    if perimeter <= 0.0 {
        return 0.0;
    }
    (4.0 * std::f64::consts::PI * area) / (perimeter * perimeter)
}

pub fn elongation_ratio(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> f64 {
    let width = max_x - min_x;
    let height = max_y - min_y;
    if width <= 0.0 || height <= 0.0 {
        return 0.0;
    }
    width.min(height) / width.max(height)
}

pub fn shannon_diversity(counts: &[usize]) -> f64 {
    let total: usize = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }

    let mut diversity = 0.0;
    for &count in counts {
        if count > 0 {
            let p = count as f64 / total as f64;
            diversity -= p * p.ln();
        }
    }

    diversity
}

pub fn functional_mixing_index(zone_areas: &[f64]) -> f64 {
    let total: f64 = zone_areas.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }

    let n = zone_areas.len() as f64;
    let mut entropy = 0.0;
    for &area in zone_areas {
        if area > 0.0 {
            let p = area / total;
            entropy -= p * p.ln();
        }
    }

    if n <= 1.0 {
        return 0.0;
    }
    entropy / n.ln()
}
