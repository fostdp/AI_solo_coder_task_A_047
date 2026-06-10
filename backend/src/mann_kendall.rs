use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MannKendallResult {
    pub s: f64,
    pub variance: f64,
    pub z_score: f64,
    pub p_value: f64,
    pub sen_slope: f64,
    pub trend_direction: String,
    pub trend_significance: bool,
}

pub fn mann_kendall_test(data: &[f64], alpha: f64) -> MannKendallResult {
    let n = data.len();
    if n < 3 {
        return MannKendallResult {
            s: 0.0,
            variance: 0.0,
            z_score: 0.0,
            p_value: 1.0,
            sen_slope: 0.0,
            trend_direction: "no trend".to_string(),
            trend_significance: false,
        };
    }

    let mut s = 0.0;
    for i in 0..n - 1 {
        for j in i + 1..n {
            let diff = data[j] - data[i];
            if diff > 0.0 {
                s += 1.0;
            } else if diff < 0.0 {
                s -= 1.0;
            }
        }
    }

    let (ties, _) = count_ties(data);
    let variance = compute_variance(n, &ties);

    let z_score = if s > 0.0 {
        (s - 1.0) / variance.sqrt()
    } else if s < 0.0 {
        (s + 1.0) / variance.sqrt()
    } else {
        0.0
    };

    let p_value = 2.0 * (1.0 - normal_cdf(z_score.abs()));

    let sen_slope = sen_slope_estimator(data);

    let trend_direction = if z_score > 0.0 {
        "increasing".to_string()
    } else if z_score < 0.0 {
        "decreasing".to_string()
    } else {
        "no trend".to_string()
    };

    let trend_significance = p_value < alpha;

    MannKendallResult {
        s,
        variance,
        z_score,
        p_value,
        sen_slope,
        trend_direction,
        trend_significance,
    }
}

fn count_ties(data: &[f64]) -> (Vec<usize>, Vec<f64>) {
    let mut counts = HashMap::new();
    for &value in data {
        *counts.entry(value).or_insert(0) += 1;
    }

    let mut tie_counts = Vec::new();
    let mut tie_values = Vec::new();
    for (&value, &count) in &counts {
        if count > 1 {
            tie_counts.push(count);
            tie_values.push(value);
        }
    }

    (tie_counts, tie_values)
}

fn compute_variance(n: usize, ties: &[usize]) -> f64 {
    let n_f64 = n as f64;
    let mut var = n_f64 * (n_f64 - 1.0) * (2.0 * n_f64 + 5.0) / 18.0;

    for &t in ties {
        let t_f64 = t as f64;
        var -= t_f64 * (t_f64 - 1.0) * (2.0 * t_f64 + 5.0) / 18.0;
    }

    var
}

fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

fn sen_slope_estimator(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 2 {
        return 0.0;
    }

    let mut slopes = Vec::new();
    for i in 0..n - 1 {
        for j in i + 1..n {
            let slope = (data[j] - data[i]) / (j - i) as f64;
            slopes.push(slope);
        }
    }

    median(&mut slopes)
}

fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

#[derive(Debug, Clone)]
pub struct SeasonalMannKendallResult {
    pub overall: MannKendallResult,
    pub seasonal: Vec<MannKendallResult>,
}

pub fn seasonal_mann_kendall(
    data: &[f64],
    seasons: usize,
    alpha: f64,
) -> SeasonalMannKendallResult {
    let mut seasonal_results = Vec::new();
    let mut combined_s = 0.0;
    let mut combined_var = 0.0;

    for s in 0..seasons {
        let seasonal_data: Vec<f64> = data
            .iter()
            .skip(s)
            .step_by(seasons)
            .copied()
            .collect();

        let result = mann_kendall_test(&seasonal_data, alpha);
        combined_s += result.s;
        combined_var += result.variance;
        seasonal_results.push(result);
    }

    let z_score = if combined_s > 0.0 {
        (combined_s - 1.0) / combined_var.sqrt()
    } else if combined_s < 0.0 {
        (combined_s + 1.0) / combined_var.sqrt()
    } else {
        0.0
    };

    let p_value = 2.0 * (1.0 - normal_cdf(z_score.abs()));

    let mut all_slopes = Vec::new();
    for result in &seasonal_results {
        all_slopes.push(result.sen_slope);
    }
    let overall_sen_slope = median(&mut all_slopes);

    let trend_direction = if z_score > 0.0 {
        "increasing".to_string()
    } else if z_score < 0.0 {
        "decreasing".to_string()
    } else {
        "no trend".to_string()
    };

    let trend_significance = p_value < alpha;

    SeasonalMannKendallResult {
        overall: MannKendallResult {
            s: combined_s,
            variance: combined_var,
            z_score,
            p_value,
            sen_slope: overall_sen_slope,
            trend_direction,
            trend_significance,
        },
        seasonal: seasonal_results,
    }
}

pub fn trend_magnitude_classification(sen_slope: f64, data_range: f64) -> String {
    if data_range <= 0.0 {
        return "negligible".to_string();
    }
    let relative = sen_slope.abs() / data_range;
    if relative < 0.01 {
        "negligible".to_string()
    } else if relative < 0.05 {
        "weak".to_string()
    } else if relative < 0.15 {
        "moderate".to_string()
    } else if relative < 0.3 {
        "strong".to_string()
    } else {
        "very strong".to_string()
    }
}
