use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub connections: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct SpatialGraph {
    pub nodes: Vec<GraphNode>,
}

impl SpatialGraph {
    pub fn new() -> Self {
        SpatialGraph { nodes: Vec::new() }
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

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn connectivity(&self, node_id: usize) -> f64 {
        if node_id < self.nodes.len() {
            self.nodes[node_id].connections.len() as f64
        } else {
            0.0
        }
    }

    pub fn depth(&self, start: usize) -> Vec<f64> {
        let n = self.nodes.len();
        let mut depths = vec![-1.0; n];
        let mut queue = VecDeque::new();
        
        depths[start] = 0.0;
        queue.push_back(start);
        
        while let Some(node) = queue.pop_front() {
            for &neighbor in &self.nodes[node].connections {
                if depths[neighbor] < 0.0 {
                    depths[neighbor] = depths[node] + 1.0;
                    queue.push_back(neighbor);
                }
            }
        }
        
        depths
    }

    pub fn total_depth(&self, node_id: usize) -> f64 {
        let depths = self.depth(node_id);
        depths.iter().filter(|&&d| d >= 0.0).sum()
    }

    pub fn mean_depth(&self, node_id: usize) -> f64 {
        let depths = self.depth(node_id);
        let reachable: Vec<&f64> = depths.iter().filter(|&&d| d >= 0.0).collect();
        if reachable.is_empty() {
            0.0
        } else {
            reachable.iter().map(|&&d| d).sum::<f64>() / reachable.len() as f64
        }
    }

    pub fn integration_global(&self, node_id: usize) -> f64 {
        let n = self.node_count() as f64;
        let total_depth = self.total_depth(node_id);
        if total_depth <= 0.0 || n <= 1.0 {
            return 0.0;
        }
        let mean_depth = total_depth / (n - 1.0);
        let ra = 2.0 * (mean_depth - 1.0) / (n - 2.0);
        let rra = ra / self.diamond_value(n);
        if rra <= 0.0 {
            0.0
        } else {
            1.0 / rra
        }
    }

    pub fn integration_local(&self, node_id: usize, radius: usize) -> f64 {
        let depths = self.depth(node_id);
        let local_nodes: Vec<usize> = depths
            .iter()
            .enumerate()
            .filter(|(_, &d)| d >= 0.0 && d <= radius as f64)
            .map(|(i, _)| i)
            .collect();
        
        let n = local_nodes.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }
        
        let total_depth: f64 = local_nodes.iter().map(|&i| depths[i]).sum();
        let mean_depth = total_depth / (n - 1.0);
        let ra = 2.0 * (mean_depth - 1.0) / (n - 2.0);
        let rra = ra / self.diamond_value(n);
        if rra <= 0.0 {
            0.0
        } else {
            1.0 / rra
        }
    }

    fn diamond_value(&self, n: f64) -> f64 {
        if n <= 3.0 {
            return 1.0;
        }
        2.0 * (n * (n.ln() + 0.5772156649) + 0.5) / (n - 1.0) / (n - 2.0)
    }

    pub fn choice_global(&self, node_id: usize) -> f64 {
        let n = self.node_count();
        let mut count = 0.0;
        
        for start in 0..n {
            if start == node_id {
                continue;
            }
            let paths = self.all_shortest_paths(start);
            for end in 0..n {
                if end == start || end == node_id {
                    continue;
                }
                if let Some(path_nodes) = paths.get(&end) {
                    let pass_through = path_nodes.iter().filter(|&&p| p == node_id).count() > 0;
                    if pass_through {
                        count += 1.0;
                    }
                }
            }
        }
        
        count
    }

    pub fn choice_local(&self, node_id: usize, radius: usize) -> f64 {
        let depths = self.depth(node_id);
        let local_nodes: HashSet<usize> = depths
            .iter()
            .enumerate()
            .filter(|(_, &d)| d >= 0.0 && d <= radius as f64)
            .map(|(i, _)| i)
            .collect();
        
        let mut count = 0.0;
        
        for &start in &local_nodes {
            if start == node_id {
                continue;
            }
            let paths = self.all_shortest_paths(start);
            for &end in &local_nodes {
                if end == start || end == node_id {
                    continue;
                }
                if let Some(path_nodes) = paths.get(&end) {
                    let pass_through = path_nodes.iter().filter(|&&p| p == node_id).count() > 0;
                    if pass_through {
                        count += 1.0;
                    }
                }
            }
        }
        
        count
    }

    fn all_shortest_paths(&self, start: usize) -> HashMap<usize, Vec<usize>> {
        let n = self.nodes.len();
        let mut dist = vec![-1i32; n];
        let mut prev: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut queue = VecDeque::new();
        
        dist[start] = 0;
        queue.push_back(start);
        
        while let Some(u) = queue.pop_front() {
            for &v in &self.nodes[u].connections {
                if dist[v] == -1 {
                    dist[v] = dist[u] + 1;
                    prev[v].push(u);
                    queue.push_back(v);
                } else if dist[v] == dist[u] + 1 {
                    prev[v].push(u);
                }
            }
        }
        
        let mut paths = HashMap::new();
        for end in 0..n {
            if dist[end] >= 0 {
                let path = self.reconstruct_path(start, end, &prev);
                paths.insert(end, path);
            }
        }
        
        paths
    }

    fn reconstruct_path(&self, start: usize, end: usize, prev: &[Vec<usize>]) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current = end;
        while current != start {
            path.push(current);
            if prev[current].is_empty() {
                break;
            }
            current = prev[current][0];
        }
        path.push(start);
        path.reverse();
        path
    }

    pub fn control(&self, node_id: usize) -> f64 {
        let mut control_value = 0.0;
        for &neighbor in &self.nodes[node_id].connections {
            let conn = self.connectivity(neighbor);
            if conn > 0.0 {
                control_value += 1.0 / conn;
            }
        }
        control_value
    }

    pub fn average_integration_global(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let total: f64 = (0..n).map(|i| self.integration_global(i)).sum();
        total / n as f64
    }

    pub fn average_choice_global(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let total: f64 = (0..n).map(|i| self.choice_global(i)).sum();
        total / n as f64
    }

    pub fn average_connectivity(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let total: f64 = (0..n).map(|i| self.connectivity(i)).sum();
        total / n as f64
    }

    pub fn average_mean_depth(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let total: f64 = (0..n).map(|i| self.mean_depth(i)).sum();
        total / n as f64
    }

    pub fn average_total_depth(&self) -> f64 {
        let n = self.node_count();
        if n == 0 {
            return 0.0;
        }
        let total: f64 = (0..n).map(|i| self.total_depth(i)).sum();
        total / n as f64
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
    pub intersects: Vec<usize>,
}

impl AxialLine {
    pub fn new(id: usize, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Self {
        AxialLine {
            id,
            start_x,
            start_y,
            end_x,
            end_y,
            intersects: Vec::new(),
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

fn direction(px: f64, py: f64, qx: f64, qy: f64, rx: f64, ry: f64) -> f64 {
    (rx - px) * (qy - py) - (qx - px) * (ry - py)
}

fn on_segment(px: f64, py: f64, qx: f64, qy: f64, rx: f64, ry: f64) -> bool {
    rx <= px.max(qx) && rx >= px.min(qx) &&
    ry <= py.max(qy) && ry >= py.min(qy)
}

pub fn build_axial_graph(lines: &[AxialLine]) -> SpatialGraph {
    let mut graph = SpatialGraph::new();
    
    for line in lines {
        let (mx, my) = line.midpoint();
        graph.add_node(mx, my);
    }
    
    for i in 0..lines.len() {
        for j in (i + 1)..lines.len() {
            if lines_intersect(
                lines[i].start_x, lines[i].start_y, lines[i].end_x, lines[i].end_y,
                lines[j].start_x, lines[j].start_y, lines[j].end_x, lines[j].end_y,
            ) {
                graph.add_edge(i, j);
            }
        }
    }
    
    graph
}
