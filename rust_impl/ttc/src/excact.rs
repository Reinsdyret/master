// Implementation of the excact algorithm finding cycles, then using residual graph from cycles finds extension of existing cycles or new cycles
// All of this until otimal solution is found

use crate::{AssignmentState, CycleStats, Doctor, Patient, dinic::Dinic};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
struct Edge {
    to: usize,
    start_capacity: usize,
    capacity: usize, // Number of times we can use this edge
    cost: i128,      // negative for original, positive for residual
    rev: usize,      // Index in adj[to]
}

pub struct CardCyclePacker {
    adj: Vec<Vec<Edge>>,
    dist: Vec<i128>,
    pred_node: Vec<usize>,
    pred_edge: Vec<usize>,
}

impl CardCyclePacker {
    pub fn new(state: &AssignmentState) -> Self {
        let patients = &state.patients;
        let doctors = &state.doctors;
        let mut adj = Vec::with_capacity(doctors.len());
        for _i in 0..doctors.len() {
            adj.push(Vec::with_capacity(doctors.len()));
        }

        let n = doctors.len();
        let mut g = Self {
            adj,
            dist: vec![0i128; n],
            pred_node: vec![n; n],
            pred_edge: vec![0; n],
        };

        let mut edges: Vec<(usize, usize)> =
            Vec::with_capacity(doctors.len() * doctors.len());

        for p in patients {
            let curr_doc = p.current_doctor.unwrap();
            let pref_doc = p.preferred_doctor;
            if curr_doc == pref_doc || !p.wants_to_switch{
                continue;
            }

            edges.push((curr_doc, pref_doc));
        }

        edges.sort();

        let mut merged: Vec<(usize, usize, usize)> =
            Vec::with_capacity(doctors.len() * doctors.len());
        for (u, v) in edges {
            if let Some(last) = merged.last_mut() {
                if last.0 == u && last.1 == v {
                    last.2 += 1;
                    continue;
                }
            }
            merged.push((u, v, 1));
        }

        for (u, v, cap) in merged {
            g.add_edge(u, v, cap, -1);
        }

        g
    }

    pub fn add_edge(&mut self, u: usize, v: usize, capacity: usize, cost: i128) {
        let forward: Edge = Edge {
            to: v,
            start_capacity: capacity,
            capacity,
            rev: self.adj[v].len(),
            cost,
        };
        let back: Edge = Edge {
            to: u,
            start_capacity: 0,
            capacity: 0,
            rev: self.adj[u].len(),
            cost: -cost,
        };

        self.adj[u].push(forward);
        self.adj[v].push(back);
    }

    pub fn pack_cycles(&mut self) -> CycleStats {
        let mut stats = CycleStats::new();
        loop {
            match self.find_negative_cycle() {
                Some(cycle) => {
                    let len = cycle.len();
                    let bottleneck = cycle.iter()
                        .map(|&(u, idx)| self.adj[u][idx].capacity)
                        .min()
                        .unwrap_or(1);
                    for _ in 0..bottleneck {
                        stats.record_cycle(len);
                    }
                    self.apply_cycle(cycle);
                }
                None => break,
            }
        }
        stats
    }

    fn find_negative_cycle(&mut self) -> Option<Vec<(usize, usize)>> {
        let n = self.adj.len();

        // Initialize: all distances 0, no predecessors set
        self.dist.iter_mut().for_each(|x| *x = 0);
        self.pred_node.iter_mut().for_each(|x| *x = n);
        self.pred_edge.iter_mut().for_each(|x| *x = 0);

        // Relax all edges n-1 times
        for _ in 0..n - 1 {
            for u in 0..n {
                for (idx, edge) in self.adj[u].iter().enumerate() {
                    if edge.capacity == 0 { continue; }
                    let new_dist = self.dist[u] + edge.cost;
                    if new_dist < self.dist[edge.to] {
                        self.dist[edge.to] = new_dist;
                        self.pred_node[edge.to] = u;
                        self.pred_edge[edge.to] = idx;
                    }
                }
            }
        }

        // Check: nth relaxation — any edge that still improves means a negative cycle exists
        let mut cycle_node = n;
        'outer: for u in 0..n {
            for (idx, edge) in self.adj[u].iter().enumerate() {
                if edge.capacity == 0 { continue; }
                if self.dist[u] + edge.cost < self.dist[edge.to] {
                    self.pred_node[edge.to] = u;
                    self.pred_edge[edge.to] = idx;
                    cycle_node = edge.to;
                    break 'outer;
                }
            }
        }

        if cycle_node == n {
            return None;
        }

        // Walk back n steps to land inside the cycle, then collect it
        let mut v = cycle_node;
        for _ in 0..n { v = self.pred_node[v]; }

        let cycle_start = v;
        let mut cycle_edges = Vec::new();
        loop {
            let u = self.pred_node[v];
            let idx = self.pred_edge[v];
            cycle_edges.push((u, idx));
            v = u;
            if v == cycle_start { break; }
        }

        Some(cycle_edges)
    }
    fn apply_cycle(&mut self, cycle: Vec<(usize, usize)>) {
        let bottleneck = cycle.iter()
            .map(|&(u, idx)| self.adj[u][idx].capacity)
            .min()
            .unwrap_or(1);

        for (u, idx) in cycle {
            let v = self.adj[u][idx].to;
            let rev_idx = self.adj[u][idx].rev;
            self.adj[u][idx].capacity -= bottleneck;
            self.adj[v][rev_idx].capacity += bottleneck;
        }
    }

    pub fn get_solution_edges(&self) -> Vec<(usize, usize, usize)> {
        let mut results = Vec::new();

        for (u, list) in self.adj.iter().enumerate() {
            for edge in list {
                if edge.cost < 0 {
                    let used_count = edge.start_capacity - edge.capacity;

                    if used_count > 0 {
                        results.push((u, edge.to, used_count));
                    }
                }
            }
        }
        results
    }

    /// Verify that the solution forms a valid circulation (flow conservation at every node).
    pub fn verify_solution(&self, patients: &Vec<Patient>, doctors: &Vec<Doctor>) -> bool {
        let n = self.adj.len();
        let mut in_flow = vec![0i64; n];
        let mut out_flow = vec![0i64; n];

        for (u, list) in self.adj.iter().enumerate() {
            for edge in list {
                if edge.cost < 0 {
                    let v = edge.to;
                    let rev_idx = edge.rev;
                    let used = self.adj[v][rev_idx].capacity as i64;
                    if used > 0 {
                        out_flow[u] += used;
                        in_flow[v] += used;
                    }
                }
            }
        }

        let mut valid = true;
        for node in 0..n {
            if in_flow[node] != out_flow[node] {
                println!(
                    "VIOLATION: node {} has in_flow={} but out_flow={}",
                    node, in_flow[node], out_flow[node]
                );
                valid = false;
            }
        }
        if valid {
            println!("Solution verified: flow conservation holds at all {} nodes.", n);
        }
        valid
    }

    /// Count how many real (non-dummy) patients are satisfied by the solution.
    /// Maps each used edge (from_doc -> to_doc, count) back to actual patients.
    pub fn count_satisfied_real_patients(&self, patients: &[Patient]) -> usize {
        self.satisfied_patients(patients).len()
    }

    /// Return references to every real patient satisfied by the solution,
    /// in the order they appear in `patients` (caller can sort as needed).
    pub fn satisfied_patients<'a>(&self, patients: &'a [Patient]) -> Vec<&'a Patient> {
        // Build remaining quota for each used edge, aggregating multiple parallel edges
        let mut edge_quota: HashMap<(usize, usize), usize> = HashMap::new();
        for (u, v, count) in self.get_solution_edges() {
            *edge_quota.entry((u, v)).or_insert(0) += count;
        }

        let mut result = Vec::new();
        for p in patients {
            if p.is_dummy {
                continue;
            }
            let curr = match p.current_doctor {
                Some(d) => d,
                None => continue,
            };
            if curr == p.preferred_doctor {
                continue;
            }
            if let Some(quota) = edge_quota.get_mut(&(curr, p.preferred_doctor)) {
                if *quota > 0 {
                    *quota -= 1;
                    result.push(p);
                }
            }
        }
        result
    }

    /// Verify that every used edge (u->v, count) is backed by enough real patients
    /// who have current_doctor=u and preferred_doctor=v.
    pub fn verify_patient_edges(&self, patients: &[Patient]) -> bool {
        // Aggregate counts across parallel edges for same (u,v)
        let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
        for (u, v, count) in self.get_solution_edges() {
            *edge_counts.entry((u, v)).or_insert(0) += count;
        }

        let mut valid = true;
        for ((u, v), count) in &edge_counts {
            let available = patients
                .iter()
                .filter(|p| !p.is_dummy && p.current_doctor == Some(*u) && p.preferred_doctor == *v)
                .count();
            if available < *count {
                println!(
                    "VIOLATION: edge ({}->{}) claims {} patients but only {} real patients exist for it",
                    u, v, count, available
                );
                valid = false;
            }
        }
        if valid {
            println!("Patient edge check passed: all solution edges are backed by real patients.");
        }
        valid
    }
}

/// Scale applied before rounding base^prio to an integer, so low-priority
/// weights stay distinct (e.g. 1.1^2 vs 1.1^3 don't both collapse to 1).
pub const UTIL_SCALE: f64 = 1_000_000.0;
/// Headroom factor for summing many edge weights in Bellman-Ford without
/// overflowing i128 (covers graphs up to ~10k nodes).
const UTIL_SUM_MARGIN: f64 = 10_000.0;

/// Largest exponent `k` for which `round(base^k * UTIL_SCALE)` stays safely
/// within i128 even after summing a long path of edges. Slow bases (1.1) get
/// a large cap (~682); fast bases (1.9) hit the integer ceiling first (~101).
pub fn util_exp_cap(base: f64) -> usize {
    let max_weight = (i128::MAX as f64) / UTIL_SUM_MARGIN;
    (max_weight / UTIL_SCALE).log(base).floor() as usize
}

/// Scaled, exponent-capped utility weight for `base^prio`. Capping the exponent
/// (not the weight) is required because base^prio overflows f64 to +inf for
/// large prio, so we must clamp before exponentiating.
pub fn util_exp_weight(base: f64, prio: usize) -> i128 {
    let k = prio.min(util_exp_cap(base));
    (base.powi(k as i32) * UTIL_SCALE).round() as i128
}

#[derive(Clone, Debug)]
struct UtilEdge {
    to: usize,
    start_capacity: usize,
    capacity: usize,
    cost: i128,
    rev: usize,
}

pub struct UtilCyclePacker {
    adj: Vec<Vec<UtilEdge>>,
    dist: Vec<i128>,
    pred_node: Vec<usize>,
    pred_edge: Vec<usize>,
}

impl UtilCyclePacker {
    pub fn new(state: &AssignmentState, prio: impl Fn(&Patient) -> i128) -> Self {
        let patients = &state.patients;
        let doctors = &state.doctors;
        let n = doctors.len();
        let mut g = Self {
            adj: (0..n).map(|_| Vec::new()).collect(),
            dist: vec![0; n],
            pred_node: vec![n; n],
            pred_edge: vec![0; n],
        };

        for p in patients {
            let curr_doc = p.current_doctor.unwrap();
            let pref_doc = p.preferred_doctor;
            if curr_doc == pref_doc || !p.wants_to_switch {
                continue;
            }
            g.add_edge(curr_doc, pref_doc, 1, -prio(p));
        }

        g
    }

    fn add_edge(&mut self, u: usize, v: usize, capacity: usize, cost: i128) {
        let fwd = UtilEdge { to: v, start_capacity: capacity, capacity, rev: self.adj[v].len(), cost };
        let back = UtilEdge { to: u, start_capacity: 0, capacity: 0, rev: self.adj[u].len(), cost: -cost };
        self.adj[u].push(fwd);
        self.adj[v].push(back);
    }

    pub fn pack_cycles(&mut self) -> CycleStats {
        let mut stats = CycleStats::new();
        loop {
            match self.find_negative_cycle() {
                Some(cycle) => {
                    let len = cycle.len();
                    let bottleneck = cycle.iter()
                        .map(|&(u, idx)| self.adj[u][idx].capacity)
                        .min()
                        .unwrap_or(1);
                    for _ in 0..bottleneck {
                        stats.record_cycle(len);
                    }
                    self.apply_cycle(cycle);
                }
                None => break,
            }
        }
        stats
    }

    fn find_negative_cycle(&mut self) -> Option<Vec<(usize, usize)>> {
        let n = self.adj.len();
        self.dist.iter_mut().for_each(|x| *x = 0);
        self.pred_node.iter_mut().for_each(|x| *x = n);
        self.pred_edge.iter_mut().for_each(|x| *x = 0);

        for _ in 0..n - 1 {
            for u in 0..n {
                for (idx, edge) in self.adj[u].iter().enumerate() {
                    if edge.capacity == 0 { continue; }
                    let new_dist = self.dist[u] + edge.cost;
                    if new_dist < self.dist[edge.to] {
                        self.dist[edge.to] = new_dist;
                        self.pred_node[edge.to] = u;
                        self.pred_edge[edge.to] = idx;
                    }
                }
            }
        }

        let mut cycle_node = n;
        'outer: for u in 0..n {
            for (idx, edge) in self.adj[u].iter().enumerate() {
                if edge.capacity == 0 { continue; }
                if self.dist[u] + edge.cost < self.dist[edge.to] {
                    self.pred_node[edge.to] = u;
                    self.pred_edge[edge.to] = idx;
                    cycle_node = edge.to;
                    break 'outer;
                }
            }
        }

        if cycle_node == n {
            return None;
        }

        let mut v = cycle_node;
        for _ in 0..n { v = self.pred_node[v]; }

        let cycle_start = v;
        let mut cycle_edges = Vec::new();
        loop {
            let u = self.pred_node[v];
            let idx = self.pred_edge[v];
            cycle_edges.push((u, idx));
            v = u;
            if v == cycle_start { break; }
        }

        Some(cycle_edges)
    }

    fn apply_cycle(&mut self, cycle: Vec<(usize, usize)>) {
        let bottleneck = cycle.iter()
            .map(|&(u, idx)| self.adj[u][idx].capacity)
            .min()
            .unwrap_or(1);

        for (u, idx) in cycle {
            let v = self.adj[u][idx].to;
            let rev_idx = self.adj[u][idx].rev;
            self.adj[u][idx].capacity -= bottleneck;
            self.adj[v][rev_idx].capacity += bottleneck;
        }
    }

    pub fn get_solution_edges(&self) -> Vec<(usize, usize, usize)> {
        let mut results = Vec::new();
        for (u, list) in self.adj.iter().enumerate() {
            for edge in list {
                if edge.cost < 0 {
                    let used_count = edge.start_capacity - edge.capacity;
                    if used_count > 0 {
                        results.push((u, edge.to, used_count));
                    }
                }
            }
        }
        results
    }

    pub fn count_satisfied_real_patients(&self, patients: &[Patient]) -> usize {
        self.satisfied_patients(patients).len()
    }

    pub fn satisfied_patients<'a>(&self, patients: &'a [Patient]) -> Vec<&'a Patient> {
        let mut edge_quota: HashMap<(usize, usize), usize> = HashMap::new();
        for (u, v, count) in self.get_solution_edges() {
            *edge_quota.entry((u, v)).or_insert(0) += count;
        }

        let mut result = Vec::new();
        for p in patients {
            if p.is_dummy { continue; }
            let curr = match p.current_doctor {
                Some(d) => d,
                None => continue,
            };
            if curr == p.preferred_doctor { continue; }
            if let Some(quota) = edge_quota.get_mut(&(curr, p.preferred_doctor)) {
                if *quota > 0 {
                    *quota -= 1;
                    result.push(p);
                }
            }
        }
        result
    }
}


#[derive(Clone, Debug)]
struct PwEdge {
    to: usize,
    capacity: usize,
    rev: usize,
}

pub struct PwCyclePacker {
    adj: Vec<Vec<PwEdge>>,
    /// (curr_doc, edge_idx_in_adj_curr_doc) per patient,
    /// sorted rank-descending (most important first), dummies last.
    patient_fwd: Vec<(usize, usize)>,
    /// Committed solution edges (curr_doc, pref_doc), one entry per satisfied patient.
    /// Populated by pack_cycles().
    solution_edges: Vec<(usize, usize)>,
    // BFS/DFS scratch — reused across every find_path call
    visit_gen: Vec<u32>,             // visit_gen[v] == current_gen iff v was visited
    bfs_parent: Vec<(usize, usize)>, // (prev_node, edge_idx), valid iff visit_gen[v]==current_gen
    bfs_queue: VecDeque<usize>,      // BFS queue
    current_gen: u32,
}

impl PwCyclePacker {
    pub fn new(state: &AssignmentState) -> Self {
        let patients = &state.patients;
        let doctors = &state.doctors;
        let n = doctors.len();

        // rank 0 = least important, rank k-1 = most important
        let mut priority_vals: Vec<usize> = patients
            .iter()
            .filter(|p| !p.is_dummy && p.wants_to_switch)
            .map(|p| p.priority)
            .collect();
        priority_vals.sort_unstable();
        priority_vals.dedup();

        let priority_to_rank: HashMap<usize, usize> = priority_vals
            .iter()
            .enumerate()
            .map(|(i, &p)| (p, i))
            .collect();

        let mut g = Self {
            adj: (0..n).map(|_| Vec::new()).collect(),
            patient_fwd: Vec::new(),
            solution_edges: Vec::new(),
            visit_gen: vec![0u32; n],
            bfs_parent: vec![(0, 0); n],
            bfs_queue: VecDeque::new(),
            current_gen: 0,
        };

        // (rank, curr_doc, edge_idx) — rank=None for dummies
        let mut patient_fwd_unsorted: Vec<(Option<usize>, usize, usize)> = Vec::new();

        for p in patients {
            if !p.wants_to_switch { continue; }
            let curr_doc = p.current_doctor.unwrap();
            let pref_doc = p.preferred_doctor;
            if curr_doc == pref_doc { continue; }

            let rank = if p.is_dummy {
                None
            } else {
                priority_to_rank.get(&p.priority).copied()
            };

            let edge_idx = g.adj[curr_doc].len();
            g.add_edge(curr_doc, pref_doc);
            patient_fwd_unsorted.push((rank, curr_doc, edge_idx));
        }

        // Sort: highest rank first (most important), None (dummies) last
        patient_fwd_unsorted.sort_by(|a, b| {
            let ra = a.0.unwrap_or(usize::MIN);
            let rb = b.0.unwrap_or(usize::MIN);
            rb.cmp(&ra)
        });

        g.patient_fwd = patient_fwd_unsorted
            .into_iter()
            .map(|(_, node, idx)| (node, idx))
            .collect();

        g
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        let rev_in_v = self.adj[v].len();
        let fwd_in_u = self.adj[u].len();
        self.adj[u].push(PwEdge { to: v, capacity: 1, rev: rev_in_v });
        self.adj[v].push(PwEdge { to: u, capacity: 0, rev: fwd_in_u });
    }

    /// BFS from `start` to `end`, only traversing edges with cap > 0 and !frozen.
    /// Returns the path as Vec<(node_u, edge_idx)>, or None if unreachable.
    /// Uses pre-allocated scratch fields — no heap allocation per call.
    fn find_path(&mut self, start: usize, end: usize) -> Option<Vec<(usize, usize)>> {
        self.current_gen = self.current_gen.wrapping_add(1);
        let bfs_gen = self.current_gen;

        self.bfs_queue.clear();
        self.visit_gen[start] = bfs_gen;
        self.bfs_queue.push_back(start);

        while let Some(node) = self.bfs_queue.pop_front() {
            if node == end { break }
            for (idx, edge) in self.adj[node].iter().enumerate() {
                if edge.capacity > 0 && self.visit_gen[edge.to] != bfs_gen {
                    self.visit_gen[edge.to] = bfs_gen;
                    self.bfs_parent[edge.to] = (node, idx);
                    self.bfs_queue.push_back(edge.to);
                }
            }
        }

        if self.visit_gen[end] != bfs_gen { return None; }

        let mut path = Vec::new();
        let mut curr = end;
        while curr != start {
            let (prev, idx) = self.bfs_parent[curr];
            path.push((prev, idx));
            curr = prev;
        }
        path.reverse();
        Some(path)
    }

    /// Commit patient at (primary_node, primary_idx) using the BFS routing path.
    /// Primary arc is consumed (no residual created) — it's removed from the graph.
    /// Routing arcs are consumed and their residuals created; lower-priority
    /// patients can still undo routing by traversing the residuals.
    fn commit_primary(&mut self, primary_node: usize, primary_idx: usize, path: Vec<(usize, usize)>) {
        let pref_doc = self.adj[primary_node][primary_idx].to;
        // Consume primary arc — leave its residual at 0 (not created)
        self.adj[primary_node][primary_idx].capacity -= 1;
        self.solution_edges.push((primary_node, pref_doc));

        // Commit routing arcs
        for (u, idx) in path {
            let v = self.adj[u][idx].to;
            let rev = self.adj[u][idx].rev;
            self.adj[u][idx].capacity -= 1;
            self.adj[v][rev].capacity += 1;
        }
    }

    pub fn pack_cycles(&mut self) -> CycleStats {
        let mut stats = CycleStats::new();

        let patient_fwd = std::mem::take(&mut self.patient_fwd);
        for &(node_u, edge_idx) in &patient_fwd {
            let fwd_cap = self.adj[node_u][edge_idx].capacity;
            if fwd_cap > 0 {
                let pref_doc = self.adj[node_u][edge_idx].to;
                if let Some(path) = self.find_path(pref_doc, node_u) {
                    let cycle_len = path.len() + 1;
                    self.commit_primary(node_u, edge_idx, path);
                    stats.record_cycle(cycle_len);
                }
            } else {
                let pref_doc = self.adj[node_u][edge_idx].to;
                let rev_idx = self.adj[node_u][edge_idx].rev;
                let rev = &mut self.adj[pref_doc][rev_idx];
                if rev.capacity > 0 {
                    rev.capacity = 0;
                    self.solution_edges.push((node_u, pref_doc));
                }
            }
        }
        self.patient_fwd = patient_fwd;
        stats
    }

    pub fn count_satisfied_real_patients(&self, patients: &[Patient]) -> usize {
        self.satisfied_patients(patients).len()
    }

    pub fn satisfied_patients<'a>(&self, patients: &'a [Patient]) -> Vec<&'a Patient> {
        let mut edge_quota: HashMap<(usize, usize), usize> = HashMap::new();
        for &(u, v) in &self.solution_edges {
            *edge_quota.entry((u, v)).or_insert(0) += 1;
        }

        // Claim slots for highest-priority patients first
        let mut switching: Vec<&Patient> = patients.iter()
            .filter(|p| !p.is_dummy)
            .filter(|p| p.current_doctor.map_or(false, |d| d != p.preferred_doctor))
            .collect();
        switching.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut result = Vec::new();
        for p in switching {
            let curr = p.current_doctor.unwrap();
            if let Some(quota) = edge_quota.get_mut(&(curr, p.preferred_doctor)) {
                if *quota > 0 {
                    *quota -= 1;
                    result.push(p);
                }
            }
        }
        result
    }
}
