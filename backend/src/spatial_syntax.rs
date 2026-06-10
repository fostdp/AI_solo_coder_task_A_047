use std::collections::{VecDeque, HashMap};

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub connections: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct SpatialGraph {
    nodes: Vec<GraphNode>,
    compact_adj: Option<Vec<Vec<u32>>>,
}

#[derive(Debug, Clone)]
pub struct SyntaxMetrics {
    pub integration: f64,
    pub choice: f64,
    pub depth: f64,
    pub connectivity: i32,
    pub control: f64,
}

impl SpatialGraph {
    pub fn new() -> Self {
        SpatialGraph {
            nodes: Vec::new(),
            compact_adj: None,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        SpatialGraph {
            nodes: Vec::with_capacity(cap),
            compact_adj: None,
        }
    }

    pub fn add_node(&mut self, x: f64, y: f64) -> usize {
        let id = self.nodes.len();
        self.nodes.push(GraphNode {
            id,
            x,
            y,
            connections: Vec::new(),
        });
        id
    }

    pub fn add_edge(&mut self, a: usize, b: usize) {
        if a < self.nodes.len() && b < self.nodes.len() && a != b {
            if !self.nodes[a].connections.contains(&b) {
                self.nodes[a].connections.push(b);
            }
            if !self.nodes[b].connections.contains(&a) {
                self.nodes[b].connections.push(a);
            }
        }
    }

    pub fn optimize_memory(&mut self) {
        let mut compact: Vec<Vec<u32>> = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let mut conn: Vec<u32> = node.connections.iter().map(|&c| c as u32).collect();
            conn.sort_unstable();
            compact.push(conn);
        }
        self.compact_adj = Some(compact);
    }

    #[inline]
    fn neighbors(&self, node_id: usize) -> &[usize] {
        if let Some(compact) = &self.compact_adj {
            unsafe {
                &*(compact[node_id].as_slice() as *const [u32] as *const [usize])
            }
        } else {
            &self.nodes[node_id].connections
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn connectivity(&self, node_id: usize) -> f64 {
        if node_id < self.nodes.len() {
            self.neighbors(node_id).len() as f64
        } else {
            0.0
        }
    }

    pub fn depth(&self, start: usize) -> Vec<f64> {
        let n = self.nodes.len();
        let mut depths = vec![-1.0; n];
        let mut queue = VecDeque::with_capacity(n / 4);
        
        depths[start] = 0.0;
        queue.push_back(start);
        
        while let Some(node) = queue.pop_front() {
            let d = depths[node];
            for &neighbor in self.neighbors(node) {
                if depths[neighbor] < 0.0 {
                    depths[neighbor] = d + 1.0;
                    queue.push_back(neighbor);
                }
            }
        }
        
        depths
    }

    pub fn mean_depth(&self, node_id: usize) -> f64 {
        let depths = self.depth(node_id);
        let mut sum = 0.0;
        let mut count = 0;
        for &d in &depths {
            if d >= 0.0 {
                sum += d;
                count += 1;
            }
        }
        if count <= 1 {
            return 0.0;
        }
        sum / (count - 1) as f64
    }

    pub fn total_depth(&self, node_id: usize) -> f64 {
        let depths = self.depth(node_id);
        let mut sum = 0.0;
        for &d in &depths {
            if d >= 0.0 {
                sum += d;
            }
        }
        sum
    }

    pub fn integration_global(&self, node_id: usize) -> f64 {
        let n = self.node_count() as f64;
        let total_depth = self.total_depth(node_id);
        if total_depth <= 0.0 || n <= 1.0 {
            return 0.0;
        }
        let mean_depth = total_depth / (n - 1.0);
        let ra = 2.0 * (mean_depth - 1.0) / (n - 2.0);
        let rra = ra / Self::diamond_value(n);
        if rra <= 0.0 {
            0.0
        } else {
            1.0 / rra
        }
    }

    pub fn integration_local(&self, node_id: usize, radius: usize) -> f64 {
        let depths = self.depth(node_id);
        let mut total_depth = 0.0;
        let mut count = 0;
        
        for &d in &depths {
            if d >= 0.0 && d <= radius as f64 {
                total_depth += d;
                count += 1;
            }
        }
        
        let n = count as f64;
        if n <= 1.0 {
            return 0.0;
        }
        
        let mean_depth = total_depth / (n - 1.0);
        let ra = 2.0 * (mean_depth - 1.0) / (n - 2.0);
        let rra = ra / Self::diamond_value(n);
        if rra <= 0.0 {
            0.0
        } else {
            1.0 / rra
        }
    }

    fn diamond_value(n: f64) -> f64 {
        if n <= 3.0 {
            return 1.0;
        }
        2.0 * (n * (n.ln() + 0.5772156649) + 0.5) / (n - 1.0) / (n - 2.0)
    }

    pub fn choice_global_brandes(&self) -> Vec<f64> {
        let n = self.node_count();
        let mut betweenness = vec![0.0_f64; n];

        if n < 3 {
            return betweenness;
        }

        for s in 0..n {
            let mut stack: Vec<usize> = Vec::with_capacity(n);
            let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); n];
            let mut sigma: Vec<f64> = vec![0.0; n];
            let mut dist: Vec<i32> = vec![-1; n];
            let mut queue: VecDeque<usize> = VecDeque::with_capacity(n / 4);

            sigma[s] = 1.0;
            dist[s] = 0;
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                for &w in self.neighbors(v) {
                    if dist[w] < 0 {
                        dist[w] = dist[v] + 1;
                        queue.push_back(w);
                    }
                    if dist[w] == dist[v] + 1 {
                        sigma[w] += sigma[v];
                        predecessors[w].push(v);
                    }
                }
            }

            let mut delta: Vec<f64> = vec![0.0; n];
            while let Some(w) = stack.pop() {
                for &v in &predecessors[w] {
                    if sigma[w] > 0.0 {
                        delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                    }
                }
                if w != s {
                    betweenness[w] += delta[w];
                }
            }
        }

        if n > 2 {
            let norm = ((n - 1) * (n - 2)) as f64;
            if norm > 0.0 {
                for b in &mut betweenness {
                    *b /= norm;
                }
            }
        }

        betweenness
    }

    pub fn choice_local_brandes(&self, radius: usize) -> Vec<f64> {
        let n = self.node_count();
        let mut betweenness = vec![0.0_f64; n];

        if n < 3 {
            return betweenness;
        }

        for s in 0..n {
            let mut stack: Vec<usize> = Vec::with_capacity(n);
            let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); n];
            let mut sigma: Vec<f64> = vec![0.0; n];
            let mut dist: Vec<i32> = vec![-1; n];
            let mut queue: VecDeque<usize> = VecDeque::with_capacity(n / 4);

            sigma[s] = 1.0;
            dist[s] = 0;
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                if dist[v] >= radius as i32 {
                    stack.push(v);
                    continue;
                }
                stack.push(v);
                for &w in self.neighbors(v) {
                    if dist[w] < 0 {
                        dist[w] = dist[v] + 1;
                        if dist[w] <= radius as i32 {
                            queue.push_back(w);
                        }
                    }
                    if dist[w] == dist[v] + 1 {
                        sigma[w] += sigma[v];
                        predecessors[w].push(v);
                    }
                }
            }

            let mut delta: Vec<f64> = vec![0.0; n];
            while let Some(w) = stack.pop() {
                for &v in &predecessors[w] {
                    if sigma[w] > 0.0 {
                        delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                    }
                }
                if w != s && dist[w] > 0 {
                    betweenness[w] += delta[w];
                }
            }
        }

        betweenness
    }

    pub fn control(&self, node_id: usize) -> f64 {
        let mut control_value = 0.0;
        for &neighbor in self.neighbors(node_id) {
            let conn = self.connectivity(neighbor);
            if conn > 0.0 {
                control_value += 1.0 / conn;
            }
        }
        control_value
    }

    pub fn compute_all_metrics(&self, local_radius: usize) -> Vec<SyntaxMetrics> {
        let n = self.node_count();
        let mut results = Vec::with_capacity(n);

        let choice_global = if n > 500 {
            self.choice_global_brandes_chunked(32)
        } else {
            self.choice_global_brandes()
        };

        let choice_local = self.choice_local_brandes(local_radius);

        for i in 0..n {
            results.push(SyntaxMetrics {
                integration: self.integration_global(i),
                choice: choice_global[i],
                depth: self.mean_depth(i),
                connectivity: self.connectivity(i) as i32,
                control: self.control(i),
            });
        }

        results
    }

    pub fn choice_global_brandes_chunked(&self, chunk_size: usize) -> Vec<f64> {
        let n = self.node_count();
        let mut betweenness = vec![0.0_f64; n];

        if n < 3 {
            return betweenness;
        }

        let chunks: Vec<usize> = (0..n).collect();
        for chunk in chunks.chunks(chunk_size) {
            let chunk_results = self.compute_chunk_betweenness(chunk);
            for (i, &val) in chunk_results.iter().enumerate() {
                betweenness[i] += val;
            }
        }

        if n > 2 {
            let norm = ((n - 1) * (n - 2)) as f64;
            if norm > 0.0 {
                for b in &mut betweenness {
                    *b /= norm;
                }
            }
        }

        betweenness
    }

    fn compute_chunk_betweenness(&self, sources: &[usize]) -> Vec<f64> {
        let n = self.node_count();
        let mut betweenness = vec![0.0_f64; n];

        for &s in sources {
            let mut stack: Vec<usize> = Vec::with_capacity(n / 2);
            let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); n];
            let mut sigma: Vec<f64> = vec![0.0; n];
            let mut dist: Vec<i32> = vec![-1; n];
            let mut queue: VecDeque<usize> = VecDeque::with_capacity(n / 8);

            sigma[s] = 1.0;
            dist[s] = 0;
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                for &w in self.neighbors(v) {
                    if dist[w] < 0 {
                        dist[w] = dist[v] + 1;
                        queue.push_back(w);
                    }
                    if dist[w] == dist[v] + 1 {
                        sigma[w] += sigma[v];
                        predecessors[w].push(v);
                    }
                }
            }

            let mut delta: Vec<f64> = vec![0.0; n];
            while let Some(w) = stack.pop() {
                for &v in &predecessors[w] {
                    if sigma[w] > 0.0 {
                        delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                    }
                }
                if w != s {
                    betweenness[w] += delta[w];
                }
            }
        }

        betweenness
    }

    pub fn average_integration_global(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 0..n {
            total += self.integration_global(i);
        }
        total / n as f64
    }

    pub fn average_connectivity(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 0..n {
            total += self.connectivity(i);
        }
        total / n as f64
    }

    pub fn average_mean_depth(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 0..n {
            total += self.mean_depth(i);
        }
        total / n as f64
    }

    pub fn average_total_depth(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 0..n {
            total += self.total_depth(i);
        }
        total / n as f64
    }

    pub fn get_component_partition(&self) -> Vec<Vec<usize>> {
        let n = self.node_count();
        let mut visited = vec![false; n];
        let mut components = Vec::new();

        for start in 0..n {
            if visited[start] {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited[start] = true;

            while let Some(node) = queue.pop_front() {
                component.push(node);
                for &neighbor in self.neighbors(node) {
                    if !visited[neighbor] {
                        visited[neighbor] = true;
                        queue.push_back(neighbor);
                    }
                }
            }

            components.push(component);
        }

        components
    }

    pub fn spatial_partition(&self, grid_size: f64) -> HashMap<(i64, i64), Vec<usize>> {
        let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        
        for node in &self.nodes {
            let gx = (node.x / grid_size).floor() as i64;
            let gy = (node.y / grid_size).floor() as i64;
            grid.entry((gx, gy)).or_default().push(node.id);
        }
        
        grid
    }

    pub fn memory_estimate(&self) -> usize {
        let mut total = 0;
        total += self.nodes.len() * std::mem::size_of::<GraphNode>();
        for node in &self.nodes {
            total += node.connections.capacity() * std::mem::size_of::<usize>();
        }
        if let Some(compact) = &self.compact_adj {
            total += compact.len() * std::mem::size_of::<Vec<u32>>();
            for c in compact {
                total += c.capacity() * std::mem::size_of::<u32>();
            }
        }
        total
    }
}

impl Default for SpatialGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AxialLine {
    pub id: usize,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub bbox: (f64, f64, f64, f64),
}

impl AxialLine {
    pub fn new(id: usize, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Self {
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);
        AxialLine {
            id,
            start_x,
            start_y,
            end_x,
            end_y,
            bbox: (min_x, max_x, min_y, max_y),
        }
    }

    pub fn midpoint(&self) -> (f64, f64) {
        ((self.start_x + self.end_x) / 2.0, (self.start_y + self.end_y) / 2.0)
    }

    pub fn length(&self) -> f64 {
        let dx = self.end_x - self.start_x;
        let dy = self.end_y - self.start_y;
        (dx * dx + dy * dy).sqrt()
    }

    #[inline]
    pub fn bbox_overlaps(&self, other: &AxialLine) -> bool {
        self.bbox.0 <= other.bbox.1 && self.bbox.1 >= other.bbox.0 &&
        self.bbox.2 <= other.bbox.3 && self.bbox.3 >= other.bbox.2
    }
}

pub fn lines_intersect(
    a1x: f64, a1y: f64, a2x: f64, a2y: f64,
    b1x: f64, b1y: f64, b2x: f64, b2y: f64,
) -> bool {
    let d1 = direction(b1x, b1y, b2x, b2y, a1x, a1y);
    let d2 = direction(b1x, b1y, b2x, b2y, a2x, a2y);
    let d3 = direction(a1x, a1y, a2x, a2y, b1x, b1y);
    let d4 = direction(a1x, a1y, a2x, a2y, b2x, b2y);
    
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) &&
       ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0)) {
        return true;
    }
    
    if d1 == 0.0 && on_segment(b1x, b1y, b2x, b2y, a1x, a1y) { return true; }
    if d2 == 0.0 && on_segment(b1x, b1y, b2x, b2y, a2x, a2y) { return true; }
    if d3 == 0.0 && on_segment(a1x, a1y, a2x, a2y, b1x, b1y) { return true; }
    if d4 == 0.0 && on_segment(a1x, a1y, a2x, a2y, b2x, b2y) { return true; }
    
    false
}

#[inline]
fn direction(px: f64, py: f64, qx: f64, qy: f64, rx: f64, ry: f64) -> f64 {
    (rx - px) * (qy - py) - (qx - px) * (ry - py)
}

#[inline]
fn on_segment(px: f64, py: f64, qx: f64, qy: f64, rx: f64, ry: f64) -> bool {
    rx <= px.max(qx) && rx >= px.min(qx) &&
    ry <= py.max(qy) && ry >= py.min(qy)
}

pub fn build_axial_graph_optimized(lines: &[AxialLine]) -> SpatialGraph {
    let n = lines.len();
    let mut graph = SpatialGraph::with_capacity(n);
    
    for line in lines {
        let (mx, my) = line.midpoint();
        graph.add_node(mx, my);
    }

    let grid_size = estimate_grid_size(lines);
    let mut spatial_index: HashMap<(i64, i64), Vec<usize>> = HashMap::new();

    for (idx, line) in lines.iter().enumerate() {
        let (min_x, max_x, min_y, max_y) = line.bbox;
        let gx_min = (min_x / grid_size).floor() as i64;
        let gx_max = (max_x / grid_size).floor() as i64;
        let gy_min = (min_y / grid_size).floor() as i64;
        let gy_max = (max_y / grid_size).floor() as i64;

        for gx in gx_min..=gx_max {
            for gy in gy_min..=gy_max {
                spatial_index.entry((gx, gy)).or_default().push(idx);
            }
        }
    }

    for (i, line_i) in lines.iter().enumerate() {
        let (min_x, max_x, min_y, max_y) = line_i.bbox;
        let gx_min = (min_x / grid_size).floor() as i64;
        let gx_max = (max_x / grid_size).floor() as i64;
        let gy_min = (min_y / grid_size).floor() as i64;
        let gy_max = (max_y / grid_size).floor() as i64;

        let mut candidates = Vec::new();
        for gx in gx_min..=gx_max {
            for gy in gy_min..=gy_max {
                if let Some(cell) = spatial_index.get(&(gx, gy)) {
                    for &j in cell {
                        if j > i {
                            candidates.push(j);
                        }
                    }
                }
            }
        }

        candidates.sort_unstable();
        candidates.dedup();

        for &j in &candidates {
            if line_i.bbox_overlaps(&lines[j]) {
                if lines_intersect(
                    line_i.start_x, line_i.start_y, line_i.end_x, line_i.end_y,
                    lines[j].start_x, lines[j].start_y, lines[j].end_x, lines[j].end_y,
                ) {
                    graph.add_edge(i, j);
                }
            }
        }
    }

    graph.optimize_memory();
    graph
}

fn estimate_grid_size(lines: &[AxialLine]) -> f64 {
    if lines.is_empty() {
        return 0.01;
    }
    
    let mut total_len = 0.0;
    for line in lines {
        total_len += line.length();
    }
    let avg_len = total_len / lines.len() as f64;
    (avg_len * 2.0).max(0.01)
}
