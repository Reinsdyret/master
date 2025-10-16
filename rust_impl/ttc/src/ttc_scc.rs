use crate::{TTCResultWithStats, TTCState, scc::TarjanSCC};
use indicatif::{ProgressBar, ProgressStyle};
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use rustc_hash::FxHashMap;
use std::collections::HashSet;

pub struct TTCSCCSolver {
    stats: SCCStats,
}

pub struct TTCSCCSolverV2 {
    stats: SCCStats,
    tarjan: TarjanSCC,
    graph: Vec<Vec<usize>>,
    index_to_patient: Vec<Option<usize>>,
    patient_index: FxHashMap<usize, usize>,
    doctor_index: FxHashMap<usize, usize>,
    used_indices: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GraphNode {
    Patient(usize),
    Doctor(usize),
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
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} Iteration {pos} [{elapsed_precise}] {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        let mut num_sccs = 1;

        while num_sccs > 0 {
            self.stats.iterations += 1;
            pb.inc(1);

            // Time: Graph building (with doctor nodes)
            let start = std::time::Instant::now();
            let (graph, node_to_id) = self.build_graph_with_doctors(state);
            self.stats.time_graph_building += start.elapsed();

            // Time: SCC finding
            let start = std::time::Instant::now();
            let sccs = self.find_sccs_tarjan(&graph, &node_to_id);
            self.stats.time_scc_finding += start.elapsed();

            num_sccs = sccs.len();

            let mut _larger_than_one_scc = 0;
            for scc in &sccs {
                if scc.len() > 1 {
                    _larger_than_one_scc += 1;
                }
                if scc.len() > self.stats.largest_scc_size {
                    self.stats.largest_scc_size = scc.len();
                }
            }
            // println!("Number of large sccs (With Doctors): {}", _larger_than_one_scc);

            self.solve_once(state, &mut stats, sccs);
        }

        stats
    }

    /// Build petgraph DiGraph with both Patient and Doctor nodes from a set of patients
    pub fn build_graph_with_doctors_from_patients(
        &self,
        patients: &HashSet<usize>,
        state: &TTCState,
    ) -> (DiGraph<GraphNode, ()>, FxHashMap<NodeIndex, usize>) {
        let mut active_patients_in_set_count = 0;
        let mut doctors_involved: HashSet<usize> = HashSet::new();

        let mut max_doctor_id = 0;
        let mut max_patient_id = 0;

        for patient_id in patients {
            let patient = state.get_patient(*patient_id).unwrap();
            if patient.wants_to_switch && !patient.is_stuck {
                active_patients_in_set_count += 1;
                doctors_involved.insert(patient.current_doctor);
                doctors_involved.insert(patient.preferred_doctor);

                if max_doctor_id < patient.current_doctor.max(patient.preferred_doctor) {
                    max_doctor_id = patient.current_doctor.max(patient.preferred_doctor);
                }

                if max_patient_id < patient.id {
                    max_patient_id = patient.id;
                }
            }
        }

        let total_nodes = active_patients_in_set_count + doctors_involved.len();
        let edge_estimate = active_patients_in_set_count * 2;

        let mut graph = DiGraph::with_capacity(total_nodes, edge_estimate);
        let mut doctor_to_node: Vec<Option<NodeIndex>> = vec![None; max_doctor_id + 1];
        let mut patient_to_node: Vec<Option<NodeIndex>> = vec![None; max_patient_id + 1];

        let mut node_to_id: FxHashMap<NodeIndex, usize> =
            FxHashMap::with_capacity_and_hasher(active_patients_in_set_count, Default::default());

        for &doctor_id in &doctors_involved {
            let graph_node = GraphNode::Doctor(doctor_id);
            let node = graph.add_node(graph_node);
            doctor_to_node[doctor_id] = Some(node);
        }

        for patient_id in patients {
            let patient = state.get_patient(*patient_id).unwrap();
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            // Add patient node
            let graph_node = GraphNode::Patient(patient.id);
            let patient_node = graph.add_node(graph_node);
            patient_to_node[patient.id] = Some(patient_node);
            node_to_id.insert(patient_node, patient.id);

            // Add edge: Patient -> Doctor (patient wants doctor)
            let preferred_doctor_node = doctor_to_node[patient.preferred_doctor].unwrap();
            graph.add_edge(patient_node, preferred_doctor_node, ());

            // Add edge: Doctor -> Patient (doctor has patient)
            let current_doctor_node = doctor_to_node[patient.current_doctor].unwrap();
            graph.add_edge(current_doctor_node, patient_node, ());
        }

        (graph, node_to_id)
    }

    /// Build petgraph DiGraph with both Patient and Doctor nodes
    fn build_graph_with_doctors(
        &self,
        state: &TTCState,
    ) -> (DiGraph<GraphNode, ()>, FxHashMap<NodeIndex, usize>) {
        // First pass: count active patients and collect doctors
        let mut active_patients_count = 0;
        let mut doctors_involved: HashSet<usize> = HashSet::new();

        for patient in &state.patients {
            if patient.wants_to_switch && !patient.is_stuck {
                active_patients_count += 1;
                doctors_involved.insert(patient.preferred_doctor);
                doctors_involved.insert(patient.current_doctor);
            }
        }

        let total_nodes = active_patients_count + doctors_involved.len();
        let edge_estimate = active_patients_count * 2; // 2 edges per patient

        let mut graph = DiGraph::with_capacity(total_nodes, edge_estimate);

        // Use Vec for O(1) lookup instead of HashMap
        // IDs are 1-indexed, so we need len + 1 to include index 0 (unused)
        let max_doctor_id = state.doctors.len() + 1;
        let max_patient_id = state.patients.len() + 1;
        let mut doctor_to_node: Vec<Option<NodeIndex>> = vec![None; max_doctor_id];
        let mut patient_to_node: Vec<Option<NodeIndex>> = vec![None; max_patient_id];

        // node_to_id ONLY stores patient nodes (for extracting patient IDs from SCCs)
        let mut node_to_id: FxHashMap<NodeIndex, usize> =
            FxHashMap::with_capacity_and_hasher(active_patients_count, Default::default());

        // Add doctor nodes first
        for &doctor_id in &doctors_involved {
            let graph_node = GraphNode::Doctor(doctor_id);
            let node = graph.add_node(graph_node);
            doctor_to_node[doctor_id] = Some(node);
        }

        // Second pass: add patient nodes and edges in one go
        for patient in &state.patients {
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            // Add patient node
            let graph_node = GraphNode::Patient(patient.id);
            let patient_node = graph.add_node(graph_node);
            patient_to_node[patient.id] = Some(patient_node);
            node_to_id.insert(patient_node, patient.id);

            // Add edge: Patient -> Doctor (patient wants doctor)
            let preferred_doctor_node = doctor_to_node[patient.preferred_doctor].unwrap();
            graph.add_edge(patient_node, preferred_doctor_node, ());

            // Add edge: Doctor -> Patient (doctor has patient)
            let current_doctor_node = doctor_to_node[patient.current_doctor].unwrap();
            graph.add_edge(current_doctor_node, patient_node, ());
        }

        (graph, node_to_id)
    }

    /// Find SCCs and extract patient-only cycles
    fn find_sccs_tarjan(
        &mut self,
        graph: &DiGraph<GraphNode, ()>,
        node_to_id: &FxHashMap<NodeIndex, usize>,
    ) -> Vec<Vec<usize>> {
        let sccs_nodes = tarjan_scc(&graph);

        // Extract only patient IDs from SCCs that contain patients
        sccs_nodes
            .into_iter()
            .filter_map(|scc| {
                let patient_ids: Vec<usize> = scc
                    .into_iter()
                    .filter_map(|node| node_to_id.get(&node).copied())
                    .collect();

                if patient_ids.is_empty() {
                    None
                } else {
                    Some(patient_ids)
                }
            })
            .collect()
    }

    pub fn solve_once(
        &mut self,
        state: &mut TTCState,
        stats: &mut TTCResultWithStats,
        sccs: Vec<Vec<usize>>,
    ) {
        for scc in sccs {
            if scc.len() == 1 {
                let patient_id = scc[0];
                let patient = state.get_patient(patient_id).unwrap();

                if patient.is_stuck || !patient.wants_to_switch {
                    continue;
                }

                if patient.current_doctor == patient.preferred_doctor {
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
                .map(|id| state.get_patient(*id).unwrap())
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

            // println!("🔍 [With Doctors] Cycle #{}: {} patients: {:?} (from SCC size {})",
            //     stats.cycles_found, cycle.len(), cycle, scc.len());

            // Time: Cycle execution
            let start = std::time::Instant::now();
            for pat in &cycle {
                let patient = state.get_patient_mut(*pat).unwrap();
                patient.wants_to_switch = false;
            }
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
}

impl TTCSCCSolverV2 {
    pub fn new() -> Self {
        TTCSCCSolverV2 {
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
            index_to_patient: Vec::new(),
            patient_index: FxHashMap::default(),
            doctor_index: FxHashMap::default(),
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
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} Iteration {pos} [{elapsed_precise}] {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        let mut num_sccs = 1;

        while num_sccs > 0 {
            self.stats.iterations += 1;
            pb.inc(1);

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

    /// Build petgraph DiGraph with both Patient and Doctor nodes from a set of patients
    fn build_graph_with_doctors_from_patients(
        &self,
        patients: &HashSet<usize>,
        state: &TTCState,
    ) -> (DiGraph<GraphNode, ()>, FxHashMap<NodeIndex, usize>) {
        let mut active_patients_in_set_count = 0;
        let mut doctors_involved: HashSet<usize> = HashSet::new();

        let mut max_doctor_id = 0;
        let mut max_patient_id = 0;

        for patient_id in patients {
            let patient = state.get_patient(*patient_id).unwrap();
            if patient.wants_to_switch && !patient.is_stuck {
                active_patients_in_set_count += 1;
                doctors_involved.insert(patient.current_doctor);
                doctors_involved.insert(patient.preferred_doctor);

                if max_doctor_id < patient.current_doctor.max(patient.preferred_doctor) {
                    max_doctor_id = patient.current_doctor.max(patient.preferred_doctor);
                }

                if max_patient_id < patient.id {
                    max_patient_id = patient.id;
                }
            }
        }

        let total_nodes = active_patients_in_set_count + doctors_involved.len();
        let edge_estimate = active_patients_in_set_count * 2;

        let mut graph = DiGraph::with_capacity(total_nodes, edge_estimate);
        let mut doctor_to_node: Vec<Option<NodeIndex>> = vec![None; max_doctor_id + 1];
        let mut patient_to_node: Vec<Option<NodeIndex>> = vec![None; max_patient_id + 1];

        let mut node_to_id: FxHashMap<NodeIndex, usize> =
            FxHashMap::with_capacity_and_hasher(active_patients_in_set_count, Default::default());

        for &doctor_id in &doctors_involved {
            let graph_node = GraphNode::Doctor(doctor_id);
            let node = graph.add_node(graph_node);
            doctor_to_node[doctor_id] = Some(node);
        }

        for patient_id in patients {
            let patient = state.get_patient(*patient_id).unwrap();
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            // Add patient node
            let graph_node = GraphNode::Patient(patient.id);
            let patient_node = graph.add_node(graph_node);
            patient_to_node[patient.id] = Some(patient_node);
            node_to_id.insert(patient_node, patient.id);

            // Add edge: Patient -> Doctor (patient wants doctor)
            let preferred_doctor_node = doctor_to_node[patient.preferred_doctor].unwrap();
            graph.add_edge(patient_node, preferred_doctor_node, ());

            // Add edge: Doctor -> Patient (doctor has patient)
            let current_doctor_node = doctor_to_node[patient.current_doctor].unwrap();
            graph.add_edge(current_doctor_node, patient_node, ());
        }

        (graph, node_to_id)
    }

    /// Build petgraph DiGraph with both Patient and Doctor nodes
    fn build_graph_with_doctors(
        &self,
        state: &TTCState,
    ) -> (DiGraph<GraphNode, ()>, FxHashMap<NodeIndex, usize>) {
        // First pass: count active patients and collect doctors
        let mut active_patients_count = 0;
        let mut doctors_involved: HashSet<usize> = HashSet::new();

        for patient in &state.patients {
            if patient.wants_to_switch && !patient.is_stuck {
                active_patients_count += 1;
                doctors_involved.insert(patient.preferred_doctor);
                doctors_involved.insert(patient.current_doctor);
            }
        }

        let total_nodes = active_patients_count + doctors_involved.len();
        let edge_estimate = active_patients_count * 2; // 2 edges per patient

        let mut graph = DiGraph::with_capacity(total_nodes, edge_estimate);

        // Use Vec for O(1) lookup instead of HashMap
        // IDs are 1-indexed, so we need len + 1 to include index 0 (unused)
        let max_doctor_id = state.doctors.len() + 1;
        let max_patient_id = state.patients.len() + 1;
        let mut doctor_to_node: Vec<Option<NodeIndex>> = vec![None; max_doctor_id];
        let mut patient_to_node: Vec<Option<NodeIndex>> = vec![None; max_patient_id];

        // node_to_id ONLY stores patient nodes (for extracting patient IDs from SCCs)
        let mut node_to_id: FxHashMap<NodeIndex, usize> =
            FxHashMap::with_capacity_and_hasher(active_patients_count, Default::default());

        // Add doctor nodes first
        for &doctor_id in &doctors_involved {
            let graph_node = GraphNode::Doctor(doctor_id);
            let node = graph.add_node(graph_node);
            doctor_to_node[doctor_id] = Some(node);
        }

        // Second pass: add patient nodes and edges in one go
        for patient in &state.patients {
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            // Add patient node
            let graph_node = GraphNode::Patient(patient.id);
            let patient_node = graph.add_node(graph_node);
            patient_to_node[patient.id] = Some(patient_node);
            node_to_id.insert(patient_node, patient.id);

            // Add edge: Patient -> Doctor (patient wants doctor)
            let preferred_doctor_node = doctor_to_node[patient.preferred_doctor].unwrap();
            graph.add_edge(patient_node, preferred_doctor_node, ());

            // Add edge: Doctor -> Patient (doctor has patient)
            let current_doctor_node = doctor_to_node[patient.current_doctor].unwrap();
            graph.add_edge(current_doctor_node, patient_node, ());
        }

        (graph, node_to_id)
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

                if patient.current_doctor == patient.preferred_doctor {
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
            for pat in &cycle {
                let patient = state.get_patient_mut(*pat).unwrap();
                patient.wants_to_switch = false;
            }
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
        for &idx in &self.used_indices {
            if idx < self.graph.len() {
                self.graph[idx].clear();
            }
            if idx < self.index_to_patient.len() {
                self.index_to_patient[idx] = None;
            }
        }
        self.used_indices.clear();
        self.patient_index.clear();
        self.doctor_index.clear();

        let mut next_index = 0;

        for patient in &state.patients {
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            let patient_idx = self.assign_patient_node(patient.id, &mut next_index);
            let preferred_idx = self.assign_doctor_node(patient.preferred_doctor, &mut next_index);
            let current_idx = self.assign_doctor_node(patient.current_doctor, &mut next_index);

            self.graph[patient_idx].push(preferred_idx);
            self.graph[current_idx].push(patient_idx);
        }

        next_index
    }

    fn ensure_node_slot(&mut self, idx: usize) {
        if self.graph.len() <= idx {
            self.graph.resize_with(idx + 1, Vec::new);
            self.index_to_patient.resize(idx + 1, None);
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
        self.index_to_patient[idx] = Some(patient_id);
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
        self.index_to_patient[idx] = None;
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
                    .filter_map(|idx| self.index_to_patient.get(idx).and_then(|entry| *entry))
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
