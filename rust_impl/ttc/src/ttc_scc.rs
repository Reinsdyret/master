use crate::{TTCResultWithStats, TTCState, scc::TarjanSCC, execute_cycle};
use rustc_hash::FxHashMap;
use std::collections::HashSet;

pub struct TTCSCCSolver {
    tarjan: TarjanSCC,
    graph: Vec<Vec<usize>>,
    patient_index: FxHashMap<usize, usize>,
    index_to_patient: Vec<usize>,
}

pub fn scc_algorithm(state: &mut TTCState) -> TTCResultWithStats {
    let mut solver = TTCSCCSolver::new();
    solver.solve(state)
}

impl TTCSCCSolver {
    pub fn new() -> Self {
        TTCSCCSolver {
            tarjan: TarjanSCC::new(),
            graph: Vec::new(),
            patient_index: FxHashMap::default(),
            index_to_patient: Vec::new(),
        }
    }

    pub fn solve(&mut self, state: &mut TTCState) -> TTCResultWithStats {
        let mut stats = TTCResultWithStats {
            cycles_found: 0,
            patients_reassigned: 0,
            patients_pruned: 0,
            remaining_capacity: state.get_total_availability(),
        };

        let mut num_sccs = 1;
        let mut iteration = 0;

        println!("[SCC] Starting SCC algorithm...");

        while num_sccs > 0 {
            iteration += 1;

            // Build patient-to-patient graph
            self.build_patient_graph(state);
            println!("[SCC] Iteration {}: Built graph with {} nodes", iteration, self.graph.len());

            // Find SCCs
            let sccs = self.find_sccs();
            num_sccs = sccs.len();

            println!("[SCC] Iteration {}: Found {} SCCs", iteration, num_sccs);

            if num_sccs == 0 {
                break;
            }

            // Process only the first SCC (like DFS processes one patient at a time)
            // This makes progress more incremental and matches DFS behavior
            let sccs_to_process = sccs.into_iter().take(1);

            // Process each SCC
            for (scc_idx, scc) in sccs_to_process.enumerate() {
                println!("[SCC] Processing SCC {}/{} with {} patients", scc_idx + 1, 1, scc.len());
                if scc.len() == 1 {
                    // Single patient - mark as stuck if not happy
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

                // Find and execute a cycle in this SCC
                let scc_set: HashSet<usize> = scc.iter().copied().collect();

                // Find lowest priority patient in the SCC
                let min_patient = scc
                    .iter()
                    .filter_map(|id| state.get_patient(*id))
                    .min_by(|p1, p2| p1.priority.cmp(&p2.priority))
                    .unwrap();
                let min_patient_id = min_patient.id;

                // Find a cycle starting from this patient
                let mut cycle = Vec::with_capacity(scc.len());
                let mut visited = HashSet::with_capacity(scc.len());
                self.dfs_find_cycle(
                    min_patient_id,
                    min_patient_id,
                    &scc_set,
                    state,
                    &mut visited,
                    &mut cycle,
                );

                if !cycle.is_empty() {
                    stats.cycles_found += 1;
                    stats.patients_reassigned += cycle.len();
                    println!("[SCC] Executing cycle of length {}", cycle.len());
                    execute_cycle(&cycle, state);
                } else {
                    println!("[SCC] WARNING: No cycle found in SCC of size {}", scc.len());
                }
            }
        }

        println!("[SCC] Finished after {} iterations", iteration);
        println!("[SCC] Total cycles found: {}", stats.cycles_found);
        println!("[SCC] Total patients reassigned: {}", stats.patients_reassigned);

        stats.remaining_capacity = state.get_total_availability();
        stats
    }

    // Build patient-to-patient adjacency list
    // Edge from A to B means: A wants the doctor that B currently has
    fn build_patient_graph(&mut self, state: &TTCState) {
        self.patient_index.clear();
        self.index_to_patient.clear();
        self.graph.clear();

        let mut next_index = 0;

        // First pass: assign indices to all active patients
        for patient in &state.patients {
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            let idx = next_index;
            next_index += 1;
            self.patient_index.insert(patient.id, idx);
            self.index_to_patient.push(patient.id);
            self.graph.push(Vec::new());
        }

        // Second pass: create edges
        for patient_a in &state.patients {
            if !patient_a.wants_to_switch || patient_a.is_stuck {
                continue;
            }

            let Some(&idx_a) = self.patient_index.get(&patient_a.id) else {
                continue;
            };

            let preferred_doctor_id = patient_a.preferred_doctor;

            // Find all patients currently at this preferred doctor
            if let Some(doctor) = state.get_doctor(preferred_doctor_id) {
                for patient_b in &doctor.switching_patients {
                    if !patient_b.wants_to_switch || patient_b.is_stuck {
                        continue;
                    }

                    if let Some(&idx_b) = self.patient_index.get(&patient_b.id) {
                        // A wants B's doctor, so edge A → B
                        self.graph[idx_a].push(idx_b);
                    }
                }
            }
        }
    }

    // Find SCCs and return as patient IDs
    fn find_sccs(&mut self) -> Vec<Vec<usize>> {
        let sccs_by_index = self.tarjan.find_sccs(&self.graph);

        // Convert indices back to patient IDs
        sccs_by_index
            .into_iter()
            .map(|scc| {
                scc.into_iter()
                    .map(|idx| self.index_to_patient[idx])
                    .collect()
            })
            .collect()
    }

    // DFS to find one cycle within an SCC
    fn dfs_find_cycle(
        &self,
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
            if self.dfs_find_cycle(
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

impl Default for TTCSCCSolver {
    fn default() -> Self {
        Self::new()
    }
}
