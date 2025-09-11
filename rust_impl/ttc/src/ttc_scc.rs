use crate::scc::{self, KosajaruSCC};
use crate::{execute_cycle, TTCResultWithStats, TTCState};
use std::collections::{HashMap, HashSet};
use indicatif::{ProgressBar, ProgressStyle};

pub struct TTCSCCSolver {
    tarjan: KosajaruSCC,
    stats: SCCStats,
}

#[derive(Debug)]
pub struct SCCStats {
    pub iterations: usize,
    pub sccs_found: usize,
    pub largest_scc_size: usize,
    pub cycles_processed: usize,
    pub total_graph_nodes: usize,
    pub total_graph_edges: usize,
}

impl TTCSCCSolver {
    pub fn new() -> Self {
        TTCSCCSolver {
            tarjan: KosajaruSCC::new(),
            stats: SCCStats {
                iterations: 0,
                sccs_found: 0,
                largest_scc_size: 0,
                cycles_processed: 0,
                total_graph_nodes: 0,
                total_graph_edges: 0,
            },
        }
    }

    pub fn get_stats(&self) -> &SCCStats {
        &self.stats
    }

    pub fn solve(&mut self, state: &mut TTCState) -> TTCResultWithStats {
        let mut stats = TTCResultWithStats { cycles_found: 0, patients_reassigned: 0, patients_pruned: 0};
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
            // Create adjacency list for tarjans
            let mut adjacency_list: HashMap<usize, Vec<usize>> = HashMap::new();
            for patient in &state.patients {
                if !patient.wants_to_switch || patient.is_stuck {
                    continue; // Skip patients who don't want to switch
                }
                let doctor = state.get_doctor(patient.preferred_doctor).unwrap();
                let mut switching_ids = Vec::with_capacity(doctor.switching_patients.len());
                for pat in &doctor.switching_patients {
                    if !pat.wants_to_switch || patient.is_stuck {
                        continue;
                    }
                    switching_ids.push(pat.id);
                }
                adjacency_list.insert(patient.id, switching_ids);
            }

            // Get SCCs
            let sccs = self.tarjan.find_sccs(&adjacency_list);
            num_sccs = sccs.len();
            self.stats.sccs_found += num_sccs;

            // println!("Number of sccs: {}", sccs.len());

            let mut larger_than_one_scc = 0;
            for scc in sccs.clone() {
                if scc.len() > 1 {
                    larger_than_one_scc += 1;
                }
                if scc.len() > self.stats.largest_scc_size {
                    self.stats.largest_scc_size = scc.len();
                }
            }
            // println!("Number of large sccs: {}", larger_than_one_scc);
            self.solve_once(state, &adjacency_list, &mut stats, sccs);
        }

        stats
    }

    pub fn solve_once(&mut self, state: &mut TTCState,adjacency_list: &HashMap<usize, Vec<usize>>, stats: &mut TTCResultWithStats, sccs: Vec<Vec<usize>>) {
        // Dfs from each lowest prio node in each scc
        for scc in sccs {
            if scc.len() == 1 {
                let patient_id = scc[0];
                let patient = state.get_patient(patient_id).unwrap();

                if patient.is_stuck || !patient.wants_to_switch {
                    continue;
                }
                
                if patient.current_doctor == patient.preferred_doctor {
                    // Patient already has preferred doctor - truly happy
                    let pat = state.get_patient_mut(patient_id).unwrap();
                    pat.wants_to_switch = false;
                } else {
                    // Patient wants different doctor but can't form cycle - stuck
                    let pat = state.get_patient_mut(patient_id).unwrap();
                    pat.is_stuck = true;
                    pat.wants_to_switch = false;
                }
                continue;
            }

            let scc_set: HashSet<usize> = scc.clone().into_iter().collect();

            // Find lowest prio in the scc
            let min_patient = scc.iter().map(|id| state.get_patient(*id).unwrap()).min_by(|p1, p2| p1.priority.cmp(&p2.priority)).unwrap();
            let min_patient_id = min_patient.id;
            
            // Run dfs from that patient and find cycle to execute
            let mut cycle = Vec::new();
            let mut visited = HashSet::new();
            self.dfs_find_one_cycle_from(min_patient_id, min_patient_id, &scc_set, &adjacency_list, &mut visited, &mut cycle);
            stats.cycles_found += 1;
            stats.patients_reassigned += cycle.len();
            
            // Mark patients in the cycle as no longer wanting to switch
            for pat in &cycle {
                let patient = state.get_patient_mut(*pat).unwrap();
                patient.wants_to_switch = false;
            }

            // println!("Solving {} length cycle for patient with prio {}", cycle.len(), min_patient_priority);
        }
    }

    fn dfs_find_one_cycle_from(
        &mut self,
        current_patient_id: usize,
        target_patient_id: usize,
        scc_set: &HashSet<usize>,
        graph: &HashMap<usize, Vec<usize>>,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>
    ) -> bool {
        if !scc_set.contains(&current_patient_id) {
            return false;
        }

        if path.len() > 1 && current_patient_id == target_patient_id {
            return true; // Found cycle back to start
        }
    
        if visited.contains(&current_patient_id) {
            return false; // Cycle detected but not target
        }

        path.push(current_patient_id);
        visited.insert(current_patient_id);

        for neighbor in graph.get(&current_patient_id).unwrap() {
            if self.dfs_find_one_cycle_from(*neighbor, target_patient_id, scc_set, graph, visited, path) {
                return true;
            }
        }

        path.pop();

        false
        
    }

    fn resolve_patients(cycle: &Vec<usize>, state: &mut TTCState) {
        
    }

}

impl Default for TTCSCCSolver {
    fn default() -> Self {
        Self::new()
    }
}