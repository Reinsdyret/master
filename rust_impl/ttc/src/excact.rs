// Implementation of the excact algorithm finding cycles, then using residual graph from cycles finds extension of existing cycles or new cycles
// All of this until otimal solution is found

use crate::{Doctor, Patient};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
struct Edge {
    to: usize,
    start_capacity: usize,
    capacity: usize, // Number of times we can use this edge
    cost: i32,       // +1 for original, -1 for residual
    rev: usize,      // Index in adj[to]
}

pub struct CyclePacker {
    adj: Vec<Vec<Edge>>,
    // Scratch space reused across SPFA calls to avoid repeated allocation
    dist: Vec<i64>,
    pred_node: Vec<usize>,
    pred_edge: Vec<usize>,
    in_queue: Vec<bool>,
    enqueue_count: Vec<usize>,
}

impl CyclePacker {
    pub fn new(patients: &Vec<Patient>, doctors: &Vec<Doctor>) -> Self {
        let mut adj = Vec::with_capacity(doctors.len());
        for _i in 0..doctors.len() {
            adj.push(Vec::with_capacity(doctors.len()));
        }

        let n = doctors.len();
        let mut g = Self {
            adj,
            dist: vec![0i64; n],
            pred_node: vec![n; n],
            pred_edge: vec![0; n],
            in_queue: vec![false; n],
            enqueue_count: vec![0; n],
        };

        let mut edges: Vec<(usize, usize)> =
            Vec::with_capacity(doctors.len() * doctors.len());

        for p in patients {
            let curr_doc = p.current_doctor.unwrap();
            let pref_doc = p.preferred_doctor;
            if curr_doc == pref_doc {
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
            g.add_edge(u, v, cap);
        }

        g
    }

    pub fn add_edge(&mut self, u: usize, v: usize, capacity: usize) {
        let forward: Edge = Edge {
            to: v,
            start_capacity: capacity,
            capacity,
            rev: self.adj[v].len(),
            cost: 1,
        };
        let back: Edge = Edge {
            to: u,
            start_capacity: 0,
            capacity: 0,
            rev: self.adj[u].len(),
            cost: -1,
        };

        self.adj[u].push(forward);
        self.adj[v].push(back);
    }

    pub fn pack_cycles(&mut self) {
        let mut total_gain = 0i32;
        let mut last_reported_gain = 0i32;
        let report_interval = 1000;

        loop {
            match self.find_positive_cycle() {
                Some(cycle) => {
                    let (cost, bottleneck) = cycle.iter().fold((0i32, usize::MAX), |(c, b), &(u, idx)| {
                        (c + self.adj[u][idx].cost, b.min(self.adj[u][idx].capacity))
                    });
                    let bottleneck = bottleneck.max(1);
                    total_gain += cost * bottleneck as i32;
                    if total_gain >= last_reported_gain + report_interval {
                        println!("Total gain: {}", total_gain);
                        last_reported_gain = total_gain;
                    }
                    self.apply_cycle(cycle);
                }
                None => {
                    println!("Finished. No more positive cycles found.");
                    break;
                }
            }
        }
    }

    fn find_positive_cycle(&mut self) -> Option<Vec<(usize, usize)>> {
        let n = self.adj.len();

        // Reset scratch arrays (reuse allocations)
        self.dist.iter_mut().for_each(|x| *x = 0);
        self.pred_node.iter_mut().for_each(|x| *x = n);
        self.pred_edge.iter_mut().for_each(|x| *x = 0);
        self.in_queue.iter_mut().for_each(|x| *x = false);
        self.enqueue_count.iter_mut().for_each(|x| *x = 0);

        // Seed all nodes
        let mut queue: VecDeque<usize> = (0..n).collect();
        for v in 0..n {
            self.in_queue[v] = true;
            self.enqueue_count[v] = 1;
        }

        let mut cycle_node = n;

        'outer: while let Some(u) = queue.pop_front() {
            self.in_queue[u] = false;

            for (idx, edge) in self.adj[u].iter().enumerate() {
                if edge.capacity == 0 {
                    continue;
                }
                let v = edge.to;
                let new_dist = self.dist[u] + edge.cost as i64;
                if new_dist > self.dist[v] {
                    self.dist[v] = new_dist;
                    self.pred_node[v] = u;
                    self.pred_edge[v] = idx;
                    if !self.in_queue[v] {
                        self.enqueue_count[v] += 1;
                        if self.enqueue_count[v] >= n {
                            cycle_node = v;
                            break 'outer;
                        }
                        // SLF: push to front if dist[v] > dist of current front
                        if queue.front().map_or(true, |&f| new_dist > self.dist[f]) {
                            queue.push_front(v);
                        } else {
                            queue.push_back(v);
                        }
                        self.in_queue[v] = true;
                    }
                }
            }
        }

        if cycle_node == n {
            return None;
        }

        // Walk back n steps to land inside the cycle
        let mut v = cycle_node;
        for _ in 0..n {
            v = self.pred_node[v];
        }

        // Collect the cycle
        let cycle_start = v;
        let mut cycle_edges = Vec::new();
        loop {
            let u = self.pred_node[v];
            let idx = self.pred_edge[v];
            cycle_edges.push((u, idx));
            v = u;
            if v == cycle_start {
                break;
            }
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
                if edge.cost == 1 {
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
                if edge.cost == 1 {
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
        // Build remaining quota for each used edge
        let mut edge_quota: HashMap<(usize, usize), usize> = HashMap::new();
        for (u, v, count) in self.get_solution_edges() {
            edge_quota.insert((u, v), count);
        }

        let mut satisfied = 0;
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
                    satisfied += 1;
                }
            }
        }
        satisfied
    }

    /// Verify that every used edge (u->v, count) is backed by enough real patients
    /// who have current_doctor=u and preferred_doctor=v.
    pub fn verify_patient_edges(&self, patients: &[Patient]) -> bool {
        let mut valid = true;
        for (u, v, count) in self.get_solution_edges() {
            let available = patients
                .iter()
                .filter(|p| !p.is_dummy && p.current_doctor == Some(u) && p.preferred_doctor == v)
                .count();
            if available < count {
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
