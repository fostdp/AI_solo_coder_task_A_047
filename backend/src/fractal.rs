use std::collections::HashSet;

pub fn boundary_fractal_dimension(
    polygon_points: &[(f64, f64)],
    num_scales: usize,
) -> f64 {
    if polygon_points.len() < 3 {
        return 0.0;
    }

    let (min_x, max_x, min_y, max_y) = bounding_box(polygon_points);
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;
    let max_size = total_width.max(total_height);

    if max_size <= 0.0 {
        return 0.0;
    }

    let mut log_scales = Vec::new();
    let mut log_counts = Vec::new();

    for i in 0..num_scales {
        let scale = max_size / (2.0_f64).powi(i as i32 + 1);
        if scale <= 0.0 {
            continue;
        }
        let count = count_boundary_boxes(polygon_points, min_x, min_y, scale);
        if count > 0 {
            log_scales.push(scale.ln());
            log_counts.push(count as f64);
        }
    }

    if log_scales.len() < 2 {
        return 0.0;
    }

    linear_regression_slope(&log_scales, &log_counts).abs()
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

fn count_boundary_boxes(
    points: &[(f64, f64)],
    min_x: f64,
    min_y: f64,
    grid_size: f64,
) -> usize {
    let mut boxes = HashSet::new();

    for i in 0..points.len() {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % points.len()];

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

pub fn network_fractal_dimension(
    line_segments: &[((f64, f64), (f64, f64))],
    num_scales: usize,
) -> f64 {
    if line_segments.is_empty() {
        return 0.0;
    }

    let (min_x, max_x, min_y, max_y) = segments_bounding_box(line_segments);
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;
    let max_size = total_width.max(total_height);

    if max_size <= 0.0 {
        return 0.0;
    }

    let mut log_scales = Vec::new();
    let mut log_counts = Vec::new();

    for i in 0..num_scales {
        let scale = max_size / (2.0_f64).powi(i as i32 + 1);
        if scale <= 0.0 {
            continue;
        }
        let count = count_network_boxes(line_segments, min_x, min_y, scale);
        if count > 0 {
            log_scales.push(scale.ln());
            log_counts.push(count as f64);
        }
    }

    if log_scales.len() < 2 {
        return 0.0;
    }

    linear_regression_slope(&log_scales, &log_counts).abs()
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

fn linear_regression_slope(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = n * sum_x2 - sum_x * sum_x;

    if denominator.abs() < 1e-10 {
        return 0.0;
    }

    numerator / denominator
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
