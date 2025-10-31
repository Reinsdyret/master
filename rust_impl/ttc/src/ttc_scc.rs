use crate::{TTCResultWithStats, TTCState, scc::TarjanSCC, execute_cycle};
use rustc_hash::FxHashMap;
use std::collections::HashSet;


pub struct TTCSCCSolver {
    stats: SCCStats,
    tarjan: TarjanSCC,
    graph: Vec<Vec<usize>>,
    index_to_node: Vec<Option<GraphNode>>,
    patient_index: FxHashMap<usize, usize>,
    doctor_index: FxHashMap<usize, usize>,
    capacity_slot_index: FxHashMap<(usize, usize), usize>,
    dummy_doctor_index: Option<usize>,
    used_indices: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphNode {
    Patient(usize),
    Doctor(usize),
    DummyDoctor,
    CapacitySlot { doctor_id: usize, slot_id: usize },
}

#[derive(Debug)]
pub struct SCCStats {
    pub iterations: usize,
    pub sccs_found: usize,
    pub largest_scc_size: usize,
    pub cycles_processed: usize,
    pub total_graph_nodes: usize,
    pub total_graph_edges: usize,
    // Timing breakdown
    pub time_graph_building: std::time::Duration,
    pub time_scc_finding: std::time::Duration,
    pub time_cycle_finding: std::time::Duration,
    pub time_cycle_execution: std::time::Duration,
}

impl TTCSCCSolver {
    pub fn new() -> Self {
        TTCSCCSolver {
            stats: SCCStats {
                iterations: 0,
                sccs_found: 0,
                largest_scc_size: 0,
                cycles_processed: 0,
                total_graph_nodes: 0,
                total_graph_edges: 0,
                time_graph_building: std::time::Duration::ZERO,
                time_scc_finding: std::time::Duration::ZERO,
                time_cycle_finding: std::time::Duration::ZERO,
                time_cycle_execution: std::time::Duration::ZERO,
            },
            tarjan: TarjanSCC::new(),
            graph: Vec::new(),
            index_to_node: Vec::new(),
            patient_index: FxHashMap::default(),
            doctor_index: FxHashMap::default(),
            capacity_slot_index: FxHashMap::default(),
            dummy_doctor_index: None,
            used_indices: Vec::new(),
        }
    }

    pub fn get_stats(&self) -> &SCCStats {
        &self.stats
    }

    pub fn solve(&mut self, state: &mut TTCState) -> TTCResultWithStats {
        let mut stats = TTCResultWithStats {
            cycles_found: 0,
            patients_reassigned: 0,
            patients_pruned: 0,
        };

        let mut num_sccs = 1;

        while num_sccs > 0 {
            self.stats.iterations += 1;

            // Time: Graph building (adjacency list)
            let start = std::time::Instant::now();
            let active_nodes = self.build_adjacency_list(state);
            self.stats.time_graph_building += start.elapsed();

            // Time: SCC finding
            let start = std::time::Instant::now();
            let sccs = self.find_sccs_with_tarjan(active_nodes);
            self.stats.time_scc_finding += start.elapsed();

            num_sccs = sccs.len();

            self.solve_once(state, &mut stats, &sccs);
        }

        stats
    }

    pub fn solve_once(
        &mut self,
        state: &mut TTCState,
        stats: &mut TTCResultWithStats,
        sccs: &Vec<Vec<usize>>,
    ) {
        for scc in sccs {
            if scc.len() == 1 {
                let patient_id = scc[0];
                let Some(patient) = state.get_patient(patient_id) else {
                    continue;
                };

                if patient.is_stuck || !patient.wants_to_switch {
                    continue;
                }

                if patient.current_doctor == Some(patient.preferred_doctor) {
                    let pat = state.get_patient_mut(patient_id).unwrap();
                    pat.wants_to_switch = false;
                } else {
                    let pat = state.get_patient_mut(patient_id).unwrap();
                    pat.is_stuck = true;
                    pat.wants_to_switch = false;
                }
                continue;
            }

            let scc_set: HashSet<usize> = scc.iter().copied().collect();

            // Find lowest priority in the SCC
            let min_patient = scc
                .iter()
                .filter_map(|id| state.get_patient(*id))
                .min_by(|p1, p2| p1.priority.cmp(&p2.priority))
                .unwrap();
            let min_patient_id = min_patient.id;

            // Time: Cycle finding (DFS)
            let start = std::time::Instant::now();
            let mut cycle = Vec::with_capacity(scc.len());
            let mut visited = HashSet::with_capacity(scc.len());
            self.dfs_find_one_cycle_from(
                min_patient_id,
                min_patient_id,
                &scc_set,
                state,
                &mut visited,
                &mut cycle,
            );
            self.stats.time_cycle_finding += start.elapsed();

            stats.cycles_found += 1;
            stats.patients_reassigned += cycle.len();

            // Time: Cycle execution
            let start = std::time::Instant::now();
            execute_cycle(&cycle, state);
            // for pat in &cycle {
            //     let patient = state.get_patient_mut(*pat).unwrap();
            //     patient.wants_to_switch = false;
            // }
            self.stats.time_cycle_execution += start.elapsed();
        }
    }

    fn dfs_find_one_cycle_from(
        &mut self,
        current_patient_id: usize,
        target_patient_id: usize,
        scc_set: &HashSet<usize>,
        state: &TTCState,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
    ) -> bool {
        if !scc_set.contains(&current_patient_id) {
            return false;
        }

        if path.len() > 1 && current_patient_id == target_patient_id {
            return true;
        }

        if visited.contains(&current_patient_id) {
            return false;
        }

        path.push(current_patient_id);
        visited.insert(current_patient_id);

        // Get neighbors directly from state
        let current_patient = state.get_patient(current_patient_id).unwrap();
        let doctor = state.get_doctor(current_patient.preferred_doctor).unwrap();

        for target_patient in &doctor.switching_patients {
            if !target_patient.wants_to_switch || target_patient.is_stuck {
                continue;
            }
            if self.dfs_find_one_cycle_from(
                target_patient.id,
                target_patient_id,
                scc_set,
                state,
                visited,
                path,
            ) {
                return true;
            }
        }

        path.pop();

        false
    }

    // Build adjacency list directly (patients and active doctors only)
    fn build_adjacency_list(&mut self, state: &TTCState) -> usize {

        self.used_indices.clear();
        self.patient_index.clear();
        self.doctor_index.clear();
        self.capacity_slot_index.clear();
        self.dummy_doctor_index = None;

        let mut next_index = 0;

        for patient in &state.patients {
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            let patient_idx = self.assign_patient_node(patient.id, &mut next_index);
            let preferred_idx = self.assign_doctor_node(patient.preferred_doctor, &mut next_index);
            
            // Only create edge from current doctor if patient is assigned
            if let Some(current_doctor_id) = patient.current_doctor {
                let current_idx = self.assign_doctor_node(current_doctor_id, &mut next_index);
                self.graph[current_idx].push(patient_idx);
            }

            self.graph[patient_idx].push(preferred_idx);
        }

        next_index
    }

    fn ensure_node_slot(&mut self, idx: usize) {
        if self.graph.len() <= idx {
            self.graph.resize_with(idx + 1, Vec::new);
            self.index_to_node.resize(idx + 1, None);
        }
    }

    fn assign_patient_node(&mut self, patient_id: usize, next_index: &mut usize) -> usize {
        if let Some(&idx) = self.patient_index.get(&patient_id) {
            return idx;
        }

        let idx = *next_index;
        *next_index += 1;
        self.ensure_node_slot(idx);
        self.graph[idx].clear();
        self.index_to_node[idx] = Some(GraphNode::Patient(patient_id));
        self.patient_index.insert(patient_id, idx);
        self.used_indices.push(idx);
        idx
    }

    fn assign_doctor_node(&mut self, doctor_id: usize, next_index: &mut usize) -> usize {
        if let Some(&idx) = self.doctor_index.get(&doctor_id) {
            return idx;
        }

        let idx = *next_index;
        *next_index += 1;
        self.ensure_node_slot(idx);
        self.graph[idx].clear();
        self.index_to_node[idx] = Some(GraphNode::Doctor(doctor_id));
        self.doctor_index.insert(doctor_id, idx);
        self.used_indices.push(idx);
        idx
    }

    // Run Tarjan's SCC algorithm and return patient IDs
    fn find_sccs_with_tarjan(&mut self, active_nodes: usize) -> Vec<Vec<usize>> {
        let all_sccs = self.tarjan.find_sccs(&self.graph[..active_nodes]);

        // Filter: only return SCCs that contain at least one patient node
        all_sccs
            .into_iter()
            .filter_map(|scc| {
                let valid_ids: Vec<usize> = scc
                    .into_iter()
                    .filter_map(|idx| {
                        self.index_to_node.get(idx).and_then(|node_opt| {
                            if let Some(GraphNode::Patient(patient_id)) = node_opt {
                                Some(*patient_id)
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                if valid_ids.is_empty() {
                    None
                } else {
                    Some(valid_ids)
                }
            })
            .collect()
    }
}

impl Default for TTCSCCSolver {
    fn default() -> Self {
        Self::new()
    }
}