use crate::scc::{self, KosajaruSCC};
use crate::{dfs_for_cycle, execute_cycle, TTCResultWithStats, TTCState};
use core::num;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use indicatif::{ProgressBar, ProgressStyle};

pub struct TTCSCCSolver {
    kosajaru: KosajaruSCC,
    stats: SCCStats,
}

pub struct TTCSCCSolverV2 {
    kosajaru: KosajaruSCC,
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
            kosajaru: KosajaruSCC::new(),
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
            let sccs = self.kosajaru.find_sccs(&adjacency_list);
            num_sccs = sccs.len();

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
            println!("Number of large sccs: {}", larger_than_one_scc);
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

impl TTCSCCSolverV2 {
    pub fn new() -> Self {
        TTCSCCSolverV2 {
            kosajaru: KosajaruSCC::new(),
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
            let adjacency_list = self.create_adjacency_list(state);
            let sccs = self.kosajaru.find_sccs(&adjacency_list);
            let mut num_invalid_patients_in_sccs = 0;
            for scc in &sccs {
                for patient_id in scc {
                    if let Some(patient) = state.get_patient(*patient_id) {
                        if !patient.wants_to_switch || patient.is_stuck {
                            num_invalid_patients_in_sccs += 1;
                        }
                    }
                }
            }

            // println!("NUMBER OF INVALID PATIENTS IN SCCS: {}", num_invalid_patients_in_sccs);
            // We can mark all 1 size sccs as stuck
            // And all size 2 as happy and resolve them instantly
            for scc in &sccs {
                if scc.len() == 1 {
                    if scc[0] <= state.patients.len() {
                        let patient_id = scc[0];
                        // println!("Pruning patient {} (out of {} patients)", patient_id, state.patients.len());
                        let patient = state.get_patient_mut(patient_id).unwrap();
                        patient.is_stuck = true;
                        stats.patients_pruned += 1;
                        
                        // Remove this patient from all doctors' switching_patients lists
                        for doctor in &mut state.doctors {
                            doctor.switching_patients.retain(|p| p.id != patient_id);
                        }
                    } else {
                        // println!("Skipping doctor {} (patient range: 1-{})", scc[0], state.patients.len());
                    }
                } else if scc.len() == 2 {
                    if scc[0] <= state.patients.len() {
                        // First is patient
                        let patient_id = scc[0];
                        let patient = state.get_patient_mut(patient_id).unwrap();
                        patient.wants_to_switch = false;
                        stats.patients_reassigned += 1;
                        
                        // Remove this patient from all doctors' switching_patients lists
                        for doctor in &mut state.doctors {
                            doctor.switching_patients.retain(|p| p.id != patient_id);
                        }
                    } else {
                        // Second is patient
                        let patient_id = scc[1];
                        let patient = state.get_patient_mut(patient_id).unwrap();
                        patient.wants_to_switch = false;
                        stats.patients_reassigned += 1;
                        
                        // Remove this patient from all doctors' switching_patients lists
                        for doctor in &mut state.doctors {
                            doctor.switching_patients.retain(|p| p.id != patient_id);
                        }
                    }
                } else {
                    // DFS through each scc
                    let mut path: Vec<usize> = Vec::with_capacity(scc.len());
                    let scc_set: HashSet<usize> = HashSet::from_iter(scc.iter().cloned());

                    let min_patient_id = scc.iter()
                        .filter(|&&id| id <= state.patients.len())
                        .min_by(|a, b| 
                            state.get_patient(**a).unwrap().priority.cmp(&state.get_patient(**b).unwrap().priority))
                        .unwrap();

                    let mut visited = HashSet::with_capacity(scc.len());
                    
                    if self.dfs_through_scc(&adjacency_list, *min_patient_id, *min_patient_id, &mut path, &mut visited, &scc_set, state) {
                        stats.cycles_found += 1;
                        // Mark all in the path as happy
                        for id in path {
                            if id <= state.patients.len() {
                                let patient = state.get_patient_mut(id).unwrap();
                                patient.wants_to_switch = false;
                                stats.patients_reassigned += 1;
                                
                                // Remove this patient from all doctors' switching_patients lists
                                for doctor in &mut state.doctors {
                                    doctor.switching_patients.retain(|p| p.id != id);
                                }
                            }
                        }
                    }
                }
            }
            
            num_sccs = sccs.iter().filter(|scc| scc.len() > 1).count();

            // println!("FOUND {} TOTAL SCCS: {:?}", sccs.len(), sccs);
            // println!("FOUND {} SCCS LARGER THAN 1", num_sccs);       
        }
        
        stats
    }

    pub fn create_adjacency_list(&self, state: &mut TTCState) -> HashMap<usize, Vec<usize>> {
        // Make adj list
        let mut adjacency_list: HashMap<usize, Vec<usize>> = HashMap::with_capacity(state.patients.len() + state.doctors.len());

        // Add all patients
        for patient in &state.patients {
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }
            adjacency_list.insert(patient.id, vec![patient.preferred_doctor + state.patients.len()]);
        }

        // Add all doctors
        for doctor in &state.doctors {
            let new_doctor_id = doctor.id + state.patients.len();
            let valid_patients: Vec<usize> = doctor.switching_patients
                .iter()
                .filter(|p| p.wants_to_switch && !p.is_stuck)
                .map(|p| p.id)
                .collect();
            
            // Only add doctor if they have valid switching patients
            if !valid_patients.is_empty() {
                adjacency_list.insert(new_doctor_id, valid_patients);
            }
        }

        return adjacency_list;
    }

    pub fn dfs_through_scc(
        &mut self,
        graph: &HashMap<usize, Vec<usize>>,
        curr: usize,
        goal: usize,
        path: &mut Vec<usize>,
        visited: &mut HashSet<usize>,
        scc_set: &HashSet<usize>,
        state: &TTCState,
    ) -> bool {
        if !scc_set.contains(&curr) {
            return false;
        }

        if path.len() > 1 && curr == goal {
            return true;
        }
    
        if visited.contains(&curr) {
            return false;
        }

        path.push(curr);
        visited.insert(curr);

        if let Some(neighbors) = graph.get(&curr) {
            // If current node is a doctor (id > patient_count), sort neighbors by priority
            let sorted_neighbors: Vec<usize> = if curr > state.patients.len() {
                // This is a doctor - sort patient neighbors by priority (lowest first)
                let mut patient_neighbors: Vec<usize> = neighbors.iter()
                    .filter(|&&id| id <= state.patients.len()) // Only patients
                    .copied()
                    .collect();
                patient_neighbors.sort_by(|a, b| {
                    let priority_a = state.get_patient(*a).map_or(usize::MAX, |p| p.priority);
                    let priority_b = state.get_patient(*b).map_or(usize::MAX, |p| p.priority);
                    priority_a.cmp(&priority_b)
                });
                patient_neighbors
            } else {
                // This is a patient - no need to sort, just go to preferred doctor
                neighbors.clone()
            };
            
            for neighbor in sorted_neighbors {
                if self.dfs_through_scc(&graph, neighbor, goal, path, visited, scc_set, state) {
                    return true;
                }
            }
        } else {
            // println!("DEBUG: Node {} not found in graph! Graph keys: {:?}", curr, graph.keys().collect::<Vec<_>>());
        }

        path.pop();
        return false;
    }
}

impl Default for TTCSCCSolver {
    fn default() -> Self {
        Self::new()
    }
}