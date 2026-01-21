// use crate::{CycleStats, TTCResultWithStats, TTCState, scc::TarjanSCC, execute_cycle};
// use rustc_hash::FxHashMap;
// use std::{collections::HashSet, hash::Hash};

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum GraphNode {
//     Patient(usize),
//     Doctor(usize),
// }

// pub struct TTCSCCSolver {
//     tarjan: TarjanSCC,
//     graph: Vec<Vec<usize>>,
//     patient_index: FxHashMap<usize, usize>,
//     doctor_index: FxHashMap<usize, usize>,
//     index_to_node: Vec<Option<GraphNode>>,
//     used_indices: Vec<usize>,
// }

// pub fn scc_algorithm(state: &mut TTCState) -> TTCResultWithStats {
//     let mut solver = TTCSCCSolver::new();
//     solver.solve(state)
// }

// impl TTCSCCSolver {
//     pub fn new() -> Self {
//         TTCSCCSolver {
//             tarjan: TarjanSCC::new(),
//             graph: Vec::new(),
//             patient_index: FxHashMap::default(),
//             doctor_index: FxHashMap::default(),
//             index_to_node: Vec::new(),
//             used_indices: Vec::new(),
//         }
//     }

//     pub fn solve(&mut self, state: &mut TTCState) -> TTCResultWithStats {
//         let mut stats = TTCResultWithStats {
//             solution: HashSet::new();
//             cycles_found: 0,
//             patients_reassigned: 0,
//             patients_pruned: 0,
//             remaining_capacity: state.get_total_availability(),
//             cycle_stats: CycleStats::new(),
//             // These are computed in benchmarking layer to avoid runtime overhead
//             initial_unsatisfied: 0,
//             final_unsatisfied: 0,
//             initial_unassigned: 0,
//             final_unassigned: 0,
//             total_capacity: 0,
//             initial_capacity_used: 0,
//         };

//         let mut num_sccs = 1;
//         let mut iteration = 0;

//         println!("[SCC] Starting SCC algorithm...");

//         while num_sccs > 0 {
//             iteration += 1;

//             // Build graph with both patients and doctors
//             let active_nodes = self.build_graph(state);
//             println!("[SCC] Iteration {}: Built graph with {} nodes", iteration, active_nodes);

//             // Find SCCs
//             let sccs = self.find_sccs(active_nodes);
//             num_sccs = sccs.len();

//             println!("[SCC] Iteration {}: Found {} SCCs with patients", iteration, num_sccs);

//             if num_sccs == 0 {
//                 break;
//             }

//             // Process all SCCs
//             let sccs_to_process = sccs;

//             for (scc_idx, scc) in sccs_to_process.iter().enumerate() {
//                 println!("[SCC] Processing SCC {}/{} with {} patients", scc_idx + 1, 1, scc.len());
                
//                 if scc.len() == 1 {
//                     // Single patient - mark as stuck if not happy
//                     let patient_id = scc[0];
//                     let Some(patient) = state.get_patient(patient_id) else {
//                         continue;
//                     };

//                     if patient.is_stuck || !patient.wants_to_switch {
//                         continue;
//                     }

//                     if patient.current_doctor == Some(patient.preferred_doctor) {
//                         let pat = state.get_patient_mut(patient_id).unwrap();
//                         pat.wants_to_switch = false;
//                     } else {
//                         let pat = state.get_patient_mut(patient_id).unwrap();
//                         pat.is_stuck = true;
//                         pat.wants_to_switch = false;
//                     }
//                     continue;
//                 }

//                 // Find and execute a cycle in this SCC
//                 let scc_set: HashSet<usize> = scc.iter().copied().collect();

//                 // Find lowest priority patient in the SCC
//                 let min_patient = scc
//                     .iter()
//                     .filter_map(|id| state.get_patient(*id))
//                     .min_by(|p1, p2| p1.priority.cmp(&p2.priority))
//                     .unwrap();
//                 let min_patient_id = min_patient.id;

//                 // Find a cycle starting from this patient
//                 let mut cycle = Vec::with_capacity(scc.len());
//                 let mut visited = HashSet::with_capacity(scc.len());
                
//                 // Note: We search in the logical patient-to-patient graph, 
//                 // but using the state to traverse
//                 if self.dfs_find_cycle(
//                     min_patient_id,
//                     min_patient_id,
//                     &scc_set,
//                     state,
//                     &mut visited,
//                     &mut cycle,
//                 ) {
//                     stats.cycles_found += 1;
                    
//                     // Record cycle statistics
//                     stats.cycle_stats.record_cycle(cycle.len());
                    
//                     // Count only real patients (not dummy capacity nodes)
//                     let real_patients_in_cycle = cycle.iter().filter(|&&pid| {
//                         state.get_patient(pid).map_or(false, |p| !p.is_dummy)
//                     }).count();
                    
//                     stats.patients_reassigned += real_patients_in_cycle;
//                     println!("[SCC] Executing cycle of length {}", cycle.len());
//                     execute_cycle(&cycle, state);
//                 } else {
//                     println!("[SCC] WARNING: No cycle found in SCC of size {}", scc.len());
//                     // Mark all in SCC as stuck to avoid infinite loop
//                     for pid in scc {
//                          if let Some(pat) = state.get_patient_mut(*pid) {
//                              pat.is_stuck = true;
//                          }
//                     }
//                 }
//             }
//         }

//         println!("[SCC] Finished after {} iterations", iteration);
//         println!("[SCC] Total cycles found: {}", stats.cycles_found);
//         println!("[SCC] Total patients reassigned: {}", stats.patients_reassigned);

//         stats.remaining_capacity = state.get_total_availability();
//         stats
//     }

//     fn ensure_node_slot(&mut self, idx: usize) {
//         if self.graph.len() <= idx {
//             self.graph.resize_with(idx + 1, Vec::new);
//             self.index_to_node.resize(idx + 1, None);
//         }
//     }

//     fn assign_patient_node(&mut self, patient_id: usize, next_index: &mut usize) -> usize {
//         if let Some(&idx) = self.patient_index.get(&patient_id) {
//             return idx;
//         }

//         let idx = *next_index;
//         *next_index += 1;
//         self.ensure_node_slot(idx);
//         self.graph[idx].clear();
//         self.index_to_node[idx] = Some(GraphNode::Patient(patient_id));
//         self.patient_index.insert(patient_id, idx);
//         self.used_indices.push(idx);
//         idx
//     }

//     fn assign_doctor_node(&mut self, doctor_id: usize, next_index: &mut usize) -> usize {
//         if let Some(&idx) = self.doctor_index.get(&doctor_id) {
//             return idx;
//         }

//         let idx = *next_index;
//         *next_index += 1;
//         self.ensure_node_slot(idx);
//         self.graph[idx].clear();
//         self.index_to_node[idx] = Some(GraphNode::Doctor(doctor_id));
//         self.doctor_index.insert(doctor_id, idx);
//         self.used_indices.push(idx);
//         idx
//     }

//     // Build graph with both patients and doctors
//     // Patient -> Preferred Doctor
//     // Doctor -> Patient (if patient is currently at doctor and wants to switch)
//     fn build_graph(&mut self, state: &TTCState) -> usize {
//         self.used_indices.clear();
//         self.patient_index.clear();
//         self.doctor_index.clear();
        
//         // We don't clear graph/index_to_node to reuse capacity, 
//         // but we will overwrite used slots
        
//         let mut next_index = 0;

//         // Build reverse_preferred map: DoctorID -> Vec<PatientID>
//         // This maps a doctor to all patients who prefer them
//         let mut reverse_preferred: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
//         for patient in &state.patients {
//             if !patient.wants_to_switch || patient.is_stuck {
//                 continue;
//             }
//             reverse_preferred
//                 .entry(patient.preferred_doctor)
//                 .or_default()
//                 .push(patient.id);
//         }

//         for patient in &state.patients {
//             if !patient.wants_to_switch || patient.is_stuck {
//                 continue;
//             }

//             let patient_idx = self.assign_patient_node(patient.id, &mut next_index);
//             let preferred_idx = self.assign_doctor_node(patient.preferred_doctor, &mut next_index);
            
//             // Edge: Patient -> Preferred Doctor
//             self.graph[patient_idx].push(preferred_idx);

//             // Only create edge from current doctor if patient is assigned
//             if let Some(current_doctor_id) = patient.current_doctor {
//                 let current_idx = self.assign_doctor_node(current_doctor_id, &mut next_index);
//                 // Edge: Current Doctor -> Patient
//                 self.graph[current_idx].push(patient_idx);

//                 // Special handling for dummy patients (capacity slots)
//                 // If this patient is a dummy, it represents an empty slot at current_doctor_id.
//                 // It should point to ANY doctor Y that has a patient P who wants current_doctor_id.
//                 // Cycle: P (at Y) -> current_doctor_id -> Dummy (at current_doctor_id) -> Y -> P
//                 if patient.is_dummy {
//                     if let Some(wanting_patients) = reverse_preferred.get(&current_doctor_id) {
//                         for &p_id in wanting_patients {
//                             // p_id wants current_doctor_id.
//                             // Find p_id's current doctor (Y)
//                             if let Some(p) = state.get_patient(p_id) {
//                                 if !p.wants_to_switch || p.is_stuck { continue; }
//                                 if let Some(y_id) = p.current_doctor {
//                                     // Add edge: Dummy (patient_idx) -> Y (y_idx)
//                                     let y_idx = self.assign_doctor_node(y_id, &mut next_index);
//                                     self.graph[patient_idx].push(y_idx);
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }

//         next_index
//     }

//     // Find SCCs and return as patient IDs
//     fn find_sccs(&mut self, active_nodes: usize) -> Vec<Vec<usize>> {
//         let all_sccs = self.tarjan.find_sccs(&self.graph[..active_nodes]);

//         // Filter: only return SCCs that contain at least one patient node
//         // And convert back to patient IDs
//         all_sccs
//             .into_iter()
//             .filter_map(|scc| {
//                 let valid_ids: Vec<usize> = scc
//                     .into_iter()
//                     .filter_map(|idx| {
//                         self.index_to_node.get(idx).and_then(|node_opt| {
//                             if let Some(GraphNode::Patient(patient_id)) = node_opt {
//                                 Some(*patient_id)
//                             } else {
//                                 None
//                             }
//                         })
//                     })
//                     .collect();

//                 if valid_ids.is_empty() {
//                     None
//                 } else {
//                     Some(valid_ids)
//                 }
//             })
//             .collect()
//     }

//     // DFS to find one cycle within an SCC
//     // This traverses the logical patient-to-patient structure
//     fn dfs_find_cycle(
//         &self,
//         current_patient_id: usize,
//         target_patient_id: usize,
//         scc_set: &HashSet<usize>,
//         state: &TTCState,
//         visited: &mut HashSet<usize>,
//         path: &mut Vec<usize>,
//     ) -> bool {
//         if !scc_set.contains(&current_patient_id) {
//             return false;
//         }

//         if path.len() > 1 && current_patient_id == target_patient_id {
//             return true;
//         }

//         if visited.contains(&current_patient_id) {
//             return false;
//         }

//         path.push(current_patient_id);
//         visited.insert(current_patient_id);

//         // Get neighbors directly from state
//         // In the mixed graph: Patient -> Preferred Doctor -> Switching Patients
//         let current_patient = state.get_patient(current_patient_id).unwrap();
//         let doctor = state.get_doctor(current_patient.preferred_doctor).unwrap();

//         for target_patient in &doctor.switching_patients {
//             if !target_patient.wants_to_switch || target_patient.is_stuck {
//                 continue;
//             }
            
//             // Only follow edges that are part of the SCC
//             if !scc_set.contains(&target_patient.id) {
//                 continue;
//             }

//             if self.dfs_find_cycle(
//                 target_patient.id,
//                 target_patient_id,
//                 scc_set,
//                 state,
//                 visited,
//                 path,
//             ) {
//                 return true;
//             }
//         }

//         path.pop();
//         false
//     }
// }

// impl Default for TTCSCCSolver {
//     fn default() -> Self {
//         Self::new()
//     }
// }
