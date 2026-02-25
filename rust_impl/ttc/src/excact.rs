// Implementation of the excact algorithm finding cycles, then using residual graph from cycles finds extension of existing cycles or new cycles
// All of this until otimal solution is found

use crate::{Doctor, Patient};

#[derive(Clone, Debug)]
struct Edge {
    to: usize,
    capacity: usize, // Number of times we can use this edge
    cost: i32,       // +1 for original, -1 for residual
    rev: usize,      // Index in adj[to]
}

pub struct CyclePacker {
    adj: Vec<Vec<Edge>>,
}

impl CyclePacker {
    pub fn new(patients: &Vec<Patient>, doctors: &Vec<Doctor>) -> Self {
        let mut adj = Vec::with_capacity(doctors.len());
        for _i in 0..doctors.len() {
            adj.push(Vec::with_capacity(doctors.len()));
        }

        let mut g = Self { adj };

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
            capacity,
            rev: self.adj[v].len(),
            cost: 1,
        };
        let back: Edge = Edge {
            to: u,
            capacity: 0,
            rev: self.adj[u].len(),
            cost: -1,
        };

        self.adj[u].push(forward);
        self.adj[v].push(back);
    }

    pub fn pack_cycles(&mut self) {
        let n = self.adj.len();
        let mut total_gain = 0;
        let mut iteration = 0;
        let mut last_reported_gain = 0;
        let report_interval = 1000;

        loop {
            let mut path = Vec::new();
            let mut visited_cost = vec![None; n];
            let mut on_stack = vec![false; n];
            let mut found_in_this_pass = false;

            for start_node in 0..n {
                if let Some(cycle) = self.find_positive_cycle(
                    start_node,
                    &mut path,
                    &mut visited_cost,
                    &mut on_stack,
                    0,
                ) {
                    iteration += 1;

                    // Calculate the cost of this specific cycle
                    let cycle_cost: i32 = cycle.iter().map(|&(u, idx)| self.adj[u][idx].cost).sum();

                    total_gain += cycle_cost;

                    if total_gain >= last_reported_gain + report_interval {
                        println!("Total gain: {}", total_gain);
                        last_reported_gain = total_gain;
                    }
                    // DEBUG PRINT
                    // println!(
                    //     "[Iteration {}] Found Cycle! Gain: +{}, Total Edges Covered: {}",
                    //     iteration, cycle_cost, total_gain
                    // );

                    // cycle_edges is built in reverse; display in forward order
                    /*
                    let cycle_str: String = cycle
                        .iter()
                        .rev()
                        .map(|&(u, idx)| {
                            let v = self.adj[u][idx].to;
                            let c = self.adj[u][idx].cost;
                            format!("{}--({})-->{}", u, if c > 0 { "+" } else { "-" }, v)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("   Cycle (len {}): {}", cycle.len(), cycle_str);
                    */

                    self.apply_cycle(cycle);
                    found_in_this_pass = true;
                    break;
                }
            }

            if !found_in_this_pass {
                println!("Finished. No more positive cycles found.");
                break;
            }
        }
    }

    fn find_positive_cycle(
    &self,
    u: usize,
    path: &mut Vec<(usize, usize)>,
    visited_cost: &mut Vec<Option<i32>>,
    on_stack: &mut Vec<bool>,
    current_cost: i32,
) -> Option<Vec<(usize, usize)>> {
    visited_cost[u] = Some(current_cost);
    on_stack[u] = true;

    for (idx, edge) in self.adj[u].iter().enumerate() {
        if edge.capacity > 0 {
            let v = edge.to;

            if on_stack[v] {
                // Potential Cycle Found!
                let mut cycle_edges = Vec::new();
                cycle_edges.push((u, idx));
                let mut cycle_sum = edge.cost;

                // SNIP: Only collect edges back until we hit node 'v'
                let mut found_start = false;
                if u == v {
                    // This is a self-loop (u -> u)
                    found_start = true;
                } else {
                    for &(node, e_idx) in path.iter().rev() {
                        cycle_edges.push((node, e_idx));
                        cycle_sum += self.adj[node][e_idx].cost;
                        if node == v {
                            found_start = true;
                            break;
                        }
                    }
                }

                if found_start && cycle_sum > 0 {
                    return Some(cycle_edges);
                }
            } else if visited_cost[v].is_none() {
                path.push((u, idx));
                if let Some(res) = self.find_positive_cycle(v, path, visited_cost, on_stack, current_cost + edge.cost) {
                    return Some(res);
                }
                path.pop();
            }
        }
    }

    on_stack[u] = false;
    None
}
    /// Flips the capacity between the edge and its residual counterpart
    fn apply_cycle(&mut self, cycle: Vec<(usize, usize)>) {
        for (u, idx) in cycle {
            let v = self.adj[u][idx].to;
            let rev_idx = self.adj[u][idx].rev;

            self.adj[u][idx].capacity -= 1;
            self.adj[v][rev_idx].capacity += 1;
        }
    }

    pub fn get_solution_edges(&self) -> Vec<(usize, usize, usize)> {
        let mut results = Vec::new();

        for (u, list) in self.adj.iter().enumerate() {
            for edge in list {
                if edge.cost == 1 {
                    let v = edge.to;
                    let rev_idx = edge.rev;
                    let used_count = self.adj[v][rev_idx].capacity;

                    if used_count > 0 {
                        results.push((u, v, used_count));
                    }
                }
            }
        }
        results
    }

    /// Verify that the solution forms a valid circulation (flow conservation at every node).
    pub fn verify_solution(&self) -> bool {
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
}
