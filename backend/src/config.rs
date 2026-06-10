pub mod algorithm {
    pub const SYNTAX_CHUNK_SIZE: usize = 32;
    pub const SYNTAX_NODE_THRESHOLD_CHUNKED: usize = 500;
    pub const SYNTAX_NODE_THRESHOLD_OPTIMIZED_GRAPH: usize = 100;
    pub const SYNTAX_LOCAL_RADIUS: usize = 3;

    pub const FRACTAL_NUM_SCALES: usize = 8;
    pub const FRACTAL_BOOTSTRAP_SAMPLES_MIN: usize = 100;
    pub const FRACTAL_BOOTSTRAP_SAMPLES_MAX: usize = 500;
    pub const FRACTAL_QUALITY_THRESHOLD: f64 = 0.3;
    pub const FRACTAL_CONFIDENCE_LEVEL: f64 = 0.95;

    pub const MK_ALPHA: f64 = 0.05;
}
