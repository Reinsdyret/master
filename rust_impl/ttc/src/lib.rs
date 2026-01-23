pub mod benchmarking;
pub mod scc;
pub mod ttc_scc;
use std::collections::{HashMap, HashSet};
use std::fs;

/// Strategy for ordering patients during TTC algorithm execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityStrategy {
    /// Process strictly by priority number (lower = higher priority)
    StrictPriority,
    /// Put unassigned patients (no current doctor) first, then by priority
    UnassignedFirst,
    /// Shuffle randomly (useful for baseline comparisons)
    Random,
    /// Process patients wanting popular doctors first (high demand → processed first)
    HighDemandFirst,
    /// Process patients wanting unpopular doctors first (low demand → processed first)  
    LowDemandFirst,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Patient {
    pub id: usize,
    pub is_dummy: bool,
    pub priority: usize,
    pub preferred_doctor: usize,
    pub current_doctor: Option<usize>, // Changed to Option to support unassigned patients
    pub wants_to_switch: bool,
    pub is_stuck: bool, // Marked as unable to be satisfied (pruned)
}

impl Patient {
    pub fn new(id: usize, is_dummy: bool, priority: usize, preferred_doctor: usize, current_doctor: Option<usize>) -> Self {
        let wants_to_switch = match current_doctor {
            Some(doctor_id) => preferred_doctor != doctor_id,
            None => true, // Unassigned patients always want to switch
        };
        Patient {
            id,
            is_dummy,
            priority,
            preferred_doctor,
            current_doctor,
            wants_to_switch,
            is_stuck: false,
        }
    }

    pub fn set_switch_preference(&mut self, wants_to_switch: bool) {
        self.wants_to_switch = wants_to_switch;
    }

    pub fn prefers_switch(&self) -> bool {
        self.wants_to_switch && match self.current_doctor {
            Some(doctor_id) => self.preferred_doctor != doctor_id,
            None => true, // Unassigned patients always prefer to switch
        }
    }

    /// Check if this patient is currently unassigned
    pub fn is_unassigned(&self) -> bool {
        self.current_doctor.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct Doctor {
    pub id: usize,
    pub is_dummy: bool,
    pub capacity: usize, // Maximum number of patients this doctor can serve
    pub switching_patients: Vec<Patient>,
    pub assigned_patients: Vec<usize>, // All currently assigned patient IDs
}

impl Doctor {
    pub fn new(id: usize, is_dummy: bool) -> Self {
        Doctor {
            id,
            is_dummy,
            capacity: 0, // Will be set during initialization
            switching_patients: Vec::new(),
            assigned_patients: Vec::new(),
        }
    }

    pub fn new_with_capacity(id: usize, is_dummy: bool, capacity: usize) -> Self {
        Doctor {
            id,
            is_dummy,
            capacity,
            switching_patients: Vec::new(),
            assigned_patients: Vec::new(),
        }
    }

    pub fn add_switching_patient(&mut self, patient: Patient) {
        let insert_pos = self
            .switching_patients
            .binary_search_by(|p| p.priority.cmp(&patient.priority))
            .unwrap_or_else(|pos| pos);
        self.switching_patients.insert(insert_pos, patient);
    }

    pub fn get_next_patient(&self) -> Option<&Patient> {
        self.switching_patients.first()
    }

    pub fn has_switching_patients(&self) -> bool {
        !self.switching_patients.is_empty()
    }

    pub fn get_patient(&self, patient_id: usize) -> Option<&Patient> {
        self.switching_patients.iter().find(|p| p.id == patient_id)
    }

    /// Calculate the number of available capacity slots
    /// Counts dummy patients in switching_patients as available slots
    pub fn available_capacity(&self) -> usize {
        self.switching_patients.iter()
            .filter(|p| p.is_dummy)
            .count()
    }

    /// Check if this doctor has any available capacity
    pub fn has_available_capacity(&self) -> bool {
        self.available_capacity() > 0
    }

    /// Add a patient to the assigned patients list
    pub fn assign_patient(&mut self, patient_id: usize) -> Result<(), String> {
        if self.assigned_patients.len() >= self.capacity {
            return Err(format!("Doctor {} is at full capacity", self.id));
        }
        if !self.assigned_patients.contains(&patient_id) {
            self.assigned_patients.push(patient_id);
        }
        Ok(())
    }

    /// Remove a patient from the assigned patients list
    pub fn unassign_patient(&mut self, patient_id: usize) {
        self.assigned_patients.retain(|&id| id != patient_id);
    }

    /// Set the capacity and update assigned patients based on current assignments
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }
}

pub fn parse_data_file(file_path: &str) -> Result<(Vec<Patient>, Vec<Doctor>), String> {
    let contents =
        fs::read_to_string(file_path).map_err(|e| format!("Error reading file: {}", e))?;

    let lines: Vec<&str> = contents.lines().collect();

    if lines.len() < 4 {
        return Err("File must have at least 4 lines".to_string());
    }

    let first_line: Vec<&str> = lines[0].split(',').collect();
    if first_line.len() != 2 {
        return Err("First line must be in format: num_patient,num_doctor".to_string());
    }

    let num_patients: usize = first_line[0]
        .parse()
        .map_err(|_| "Invalid number of patients")?;
    let num_doctors: usize = first_line[1]
        .parse()
        .map_err(|_| "Invalid number of doctors")?;

    let preferred_doctors: Vec<usize> = lines[1]
        .split(',')
        .map(|s| s.parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid preferred doctor values")?;

    // Parse current doctors, 0 means dummy doctor (unassigned)
    let current_doctors: Vec<Option<usize>> = lines[2]
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None) // Empty means truly unassigned
            } else {
                trimmed.parse::<usize>().map(Some) // 0 -> Some(0) for dummy doctor
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid current doctor values")?;

    let priorities: Vec<usize> = lines[3]
        .split(',')
        .map(|s| s.parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid priority values")?;

    // Parse optional capacity information (line 6, index 5)
    let capacities: Option<Vec<usize>> = if lines.len() >= 6 {
        Some(
            lines[5]
                .split(',')
                .map(|s| s.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "Invalid capacity values")?,
        )
    } else {
        None
    };

    if preferred_doctors.len() != num_patients
        || current_doctors.len() != num_patients
        || priorities.len() != num_patients
    {
        return Err("Mismatch in patient data lengths".to_string());
    }

    if let Some(ref caps) = capacities {
        if caps.len() != num_doctors {
            return Err("Mismatch in doctor capacity data length".to_string());
        }
    }

    let mut patients = Vec::with_capacity(num_patients);
    for i in 0..num_patients {
        let patient = Patient::new(
            i + 1, // Patient ID starts from 1,
            false,
            priorities[i],
            preferred_doctors[i],
            current_doctors[i],
        );
        patients.push(patient);
    }

    // Create dummy doctor (ID 0) for unassigned patients
    let unassigned_count = current_doctors.iter().filter(|d| **d == Some(0)).count();
    let mut dummy_doctor = Doctor::new_with_capacity(0, true, unassigned_count);

    // Assign unassigned patients to dummy doctor
    for patient in &patients {
        if patient.current_doctor == Some(0) {
            dummy_doctor.assigned_patients.push(patient.id);
        }
    }

    // Initialize doctors with capacity information
    let mut doctors = Vec::with_capacity(num_doctors);
    doctors.push(dummy_doctor); // Add dummy doctor at index 0

    // Track next dummy patient ID (starting after real patients)
    let mut next_dummy_patient_id = num_patients + 1;

    for i in 1..=num_doctors {
        let capacity = if let Some(ref caps) = capacities {
            caps[i - 1] // Capacity array is 0-indexed
        } else {
            // Default capacity: count current assignments for backward compatibility
            patients
                .iter()
                .filter(|p| p.current_doctor == Some(i))
                .count()
        };

        let mut doctor = Doctor::new_with_capacity(i, false, capacity);

        // Initialize assigned_patients list based on current assignments
        for patient in &patients {
            if patient.current_doctor == Some(i) {
                doctor.assigned_patients.push(patient.id);
            }
        }

        // Create dummy patients for available capacity slots
        let available_capacity = capacity.saturating_sub(doctor.assigned_patients.len());
        for _ in 0..available_capacity {
            // Create a dummy patient that:
            // - Is currently assigned to this doctor (filling the empty slot)
            // - Wants to go to the dummy doctor (ID 0)
            // - Has lowest priority (to be processed last)
            let dummy_patient = Patient::new(
                next_dummy_patient_id,
                true,
                usize::MAX, // Lowest priority
                0,          // Wants dummy doctor
                Some(i),    // Currently at this real doctor
            );
            doctor.assigned_patients.push(next_dummy_patient_id);
            patients.push(dummy_patient);
            next_dummy_patient_id += 1;
        }

        doctors.push(doctor);
    }

    // Validate that current assignments don't exceed capacity
    for doctor in &doctors {
        if doctor.assigned_patients.len() > doctor.capacity {
            return Err(format!(
                "Doctor {} has {} assigned patients but capacity is only {}",
                doctor.id,
                doctor.assigned_patients.len(),
                doctor.capacity
            ));
        }
    }

    let mut doctor_map: HashMap<usize, &mut Doctor> = HashMap::new();
    for doctor in &mut doctors {
        doctor_map.insert(doctor.id, doctor);
    }

    for patient in &patients {
        if patient.prefers_switch() {
            if let Some(current_doctor_id) = patient.current_doctor {
                if let Some(doctor) = doctor_map.get_mut(&current_doctor_id) {
                    doctor.add_switching_patient(patient.clone());
                }
            }
            // Note: patients with current_doctor == Some(0) are already handled above
        }
    }

    Ok((patients, doctors))
}

#[derive(Clone)]
pub struct TTCState {
    pub patients: Vec<Patient>,
    pub doctors: Vec<Doctor>,
    pub patients_by_priority: Vec<usize>, // Patient IDs sorted by priority
}

impl TTCState {
    pub fn new(patients: Vec<Patient>, doctors: Vec<Doctor>) -> Self {
        let mut patient_priority_pairs: Vec<(usize, usize)> =
            patients.iter().map(|p| (p.id, p.priority)).collect();

        patient_priority_pairs.sort_by(|a, b| a.1.cmp(&b.1)); // Sort by priority
        let patients_by_priority: Vec<usize> = patient_priority_pairs
            .into_iter()
            .map(|(id, _)| id)
            .collect();

        Self {
            patients,
            doctors,
            patients_by_priority,
        }
    }

    pub fn get_patient(&self, id: usize) -> Option<&Patient> {
        self.patients.get(id - 1) // Assuming IDs start from 1
    }

    pub fn get_patient_mut(&mut self, id: usize) -> Option<&mut Patient> {
        self.patients.get_mut(id - 1)
    }

    pub fn get_doctor(&self, id: usize) -> Option<&Doctor> {
        // Doctor ID 0 is the dummy doctor at index 0
        // Doctor IDs 1..=N are at indices 1..=N
        self.doctors.get(id)
    }

    pub fn get_doctor_mut(&mut self, id: usize) -> Option<&mut Doctor> {
        self.doctors.get_mut(id)
    }

    pub fn get_total_availability(&self) -> usize {
        let result: usize = self.doctors.iter().map(|doctor| {
            if doctor.is_dummy {0} else {doctor.available_capacity()}
        }).sum();
        result
    }

    /// Get total capacity across all non-dummy doctors
    pub fn get_total_capacity(&self) -> usize {
        self.doctors.iter()
            .filter(|d| !d.is_dummy)
            .map(|d| d.capacity)
            .sum()
    }

    /// Get current capacity utilization (slots filled)
    pub fn get_capacity_used(&self) -> usize {
        self.get_total_capacity() - self.get_total_availability()
    }

    /// Count patients who want to switch (excluding dummy patients)
    pub fn count_unsatisfied_patients(&self) -> usize {
        self.patients.iter()
            .filter(|p| !p.is_dummy && p.wants_to_switch)
            .count()
    }

    /// Count patients without a doctor (current_doctor == Some(0), excluding dummies)
    pub fn count_unassigned_patients(&self) -> usize {
        self.patients.iter()
            .filter(|p| !p.is_dummy && p.current_doctor == Some(0))
            .count()
    }

    pub fn resolve_patient(&mut self, id: usize) {
        let current_doctor_id = self.get_patient(id).and_then(|p| p.current_doctor);

        // Remove patient from current doctor
        if let Some(current_doctor_id) = current_doctor_id {
            if let Some(current_doctor) = self.get_doctor_mut(current_doctor_id) {
                current_doctor.switching_patients.retain(|p| p.id != id);
            }
        }

        // Add patient to preferred doctor and update patient's current_doctor
        let preferred_doctor_id = self.get_patient(id).and_then(|p| Some(p.preferred_doctor));

        if let Some(preferred_doctor_id) = preferred_doctor_id {
            if let Some(preferred_doctor) = self.get_doctor_mut(preferred_doctor_id) {
                let _ = preferred_doctor.assign_patient(id);
            }
            // Update patient's current_doctor field
            if let Some(patient) = self.get_patient_mut(id) {
                patient.current_doctor = Some(preferred_doctor_id);
            }
        }

        // Mark as do not want to switch
        if let Some(patient) = self.get_patient_mut(id) {
            patient.wants_to_switch = false;
        }
        
    }
}


/// Backward compatibility wrapper: Unassigned patients first
pub fn ttc_algorithm_with_pruning(state: &mut TTCState) -> TTCResultWithStats {
    ttc_algorithm(state, PriorityStrategy::UnassignedFirst)
}

/// Backward compatibility wrapper: Strict priority order
pub fn ttc_algorithm_without_prioritization(state: &mut TTCState) -> TTCResultWithStats {
    ttc_algorithm(state, PriorityStrategy::StrictPriority)
}

/// Main TTC algorithm with configurable patient ordering strategy
pub fn ttc_algorithm(state: &mut TTCState, strategy: PriorityStrategy) -> TTCResultWithStats {
    let mut solution: HashSet<usize> = HashSet::with_capacity(state.patients.len() + state.doctors.len());
    let mut cycles_found = 0;
    let mut total_patients_reassigned = 0;
    let mut cycle_stats = CycleStats::new();

    println!("[TTC] Starting with strategy: {:?}", strategy);

    // Get all switching patients (base list, sorted by priority)
    let mut switching_patients: Vec<usize> = state
        .patients_by_priority
        .iter()
        .filter(|&&id| {
            state.get_patient(id).map_or(false, |p| p.wants_to_switch)
        })
        .copied()
        .collect();

    // Apply the ordering strategy
    match strategy {
        PriorityStrategy::StrictPriority => {
            // Already sorted by priority, nothing to do
        }
        PriorityStrategy::UnassignedFirst => {
            // Partition: unassigned first, then rest (both groups maintain priority order)
            let mut unassigned = Vec::new();
            let mut assigned = Vec::new();
            for &pid in &switching_patients {
                if let Some(p) = state.get_patient(pid) {
                    if p.current_doctor == Some(0) {
                        unassigned.push(pid);
                    } else {
                        assigned.push(pid);
                    }
                }
            }
            switching_patients = unassigned;
            switching_patients.extend(assigned);
        }
        PriorityStrategy::Random => {
            use rand::seq::SliceRandom;
            use rand::thread_rng;
            switching_patients.shuffle(&mut thread_rng());
        }
        PriorityStrategy::HighDemandFirst | PriorityStrategy::LowDemandFirst => {
            // Build demand map: count how many patients want each doctor
            let mut doctor_demand: HashMap<usize, usize> = HashMap::new();
            for patient in &state.patients {
                if patient.wants_to_switch {
                    *doctor_demand.entry(patient.preferred_doctor).or_insert(0) += 1;
                }
            }
            
            let ascending = matches!(strategy, PriorityStrategy::LowDemandFirst);
            switching_patients.sort_by_cached_key(|&pid| {
                let demand = state
                    .get_patient(pid)
                    .map(|p| doctor_demand.get(&p.preferred_doctor).copied().unwrap_or(0))
                    .unwrap_or(if ascending { usize::MAX } else { 0 });
                let priority = state.get_patient(pid).map(|p| p.priority).unwrap_or(usize::MAX);
                if ascending {
                    (demand, priority)
                } else {
                    (usize::MAX - demand, priority) // Reverse by subtracting from MAX
                }
            });
        }
    }

    // Process patients in the computed order
    for &patient_id in &switching_patients {
        let _patient = match state.get_patient(patient_id) {
            Some(p) if p.wants_to_switch => p,
            _ => continue,
        };
        
        if let Some(cycle) = find_cycle_from_patient_with_direct_pruning(patient_id, state) {
            cycles_found += 1;
            
            // Count only real patients (not dummy capacity nodes), also adding to solution set
            let real_patients_in_cycle = cycle.iter().filter(|&&pid| {
                if let Some(p) = state.get_patient(pid) {
                    if !p.is_dummy {
                        solution.insert(p.priority);
                    }
                }
                state.get_patient(pid).map_or(false, |p| !p.is_dummy)
            }).count();
            
            cycle_stats.record_cycle(cycle.len());
            total_patients_reassigned += real_patients_in_cycle;
            execute_cycle(&cycle, state);
        }
    }

    TTCResultWithStats {
        solution,
        cycles_found,
        patients_reassigned: total_patients_reassigned,
        patients_pruned: 0,
        remaining_capacity: state.get_total_availability(),
        cycle_stats,
        initial_unsatisfied: 0,
        final_unsatisfied: 0,
        initial_unassigned: 0,
        final_unassigned: 0,
        total_capacity: 0,
        initial_capacity_used: 0,
    }
}

pub fn restricted_ttc_algorithm(initState: &mut TTCState) -> TTCResultWithStats {
    let mut state = initState.clone();
    let mut solution: HashSet<usize> = HashSet::with_capacity(state.patients.len());

    let mut cycles_found = 0;
    let mut total_patients_reassigned = 0;
    let mut cycle_stats = CycleStats::new();
    // Get all switching patients (base list, sorted by priority)
    let switching_patients: Vec<usize> = state
        .patients_by_priority
        .iter()
        .filter(|&&id| {
            state.get_patient(id).map_or(false, |p| p.wants_to_switch)
        })
        .copied()
        .collect();

    // Start cycle search on highest prio patient
    for &patient_id in &switching_patients {
        let _patient = match state.get_patient(patient_id) {
            Some(p) if p.wants_to_switch => p,
            _ => continue,
        };

        // search from patient
        if let Some(cycle) = restricted_dfs(patient_id, &mut state) {
            cycles_found += 1;
            cycle_stats.cycle_lengths.push(cycle.len());
            // We have simple cycle, add patients to solution and remove patients from doctor preffered
            let mut is_doctor = false;
            for id in cycle {
                if is_doctor {
                    if let Some(doctor) = state.get_doctor_mut(id) {
                        doctor.switching_patients.remove(0);
                    }
                } else {
                    // skip dummy patients
                    if let Some(patient) = state.get_patient(id) {
                        if patient.is_dummy {continue;}
                        solution.insert(patient.priority);
                        total_patients_reassigned += 1;
                    }
                }
                is_doctor = !is_doctor;

            }
        }

    }

    TTCResultWithStats {
        solution,
        cycles_found,
        patients_reassigned: total_patients_reassigned,
        patients_pruned: 0,
        remaining_capacity: state.get_total_availability(),
        cycle_stats,
        initial_unsatisfied: 0,
        final_unsatisfied: 0,
        initial_unassigned: 0,
        final_unassigned: 0,
        total_capacity: 0,
        initial_capacity_used: 0,
    }
}

pub fn restricted_dfs(patient_id: usize, state: &mut TTCState) -> Option<Vec<usize>>{
    let mut path = Vec::new();
    let mut visited = HashSet::new();

    if actual_restricted_dfs(patient_id, patient_id, state, &mut path, &mut visited) {
        Some(path)
    } else {
        None
    }
}

pub fn actual_restricted_dfs(current_id: usize, goal_id: usize, state: &mut TTCState, path: &mut Vec<usize>, visited: &mut HashSet<usize>) -> bool{
    // if patient:
    if let Some(patient) = state.get_patient(current_id) {
        // if equal to goal id: cycle
        if current_id == goal_id && path.len() > 0{
            return true
        }
        // if in visited: no cycle
        if visited.contains(&current_id) {return false}
        
        // else: add to path and visited then visit doctor
        path.push(current_id);
        visited.insert(current_id);
        return actual_restricted_dfs(patient.preferred_doctor, goal_id, state, path, visited);
    }

    // if doctor:
    if let Some(doctor) = state.get_doctor(current_id) {
        // if a next preffered not in visited: visit that patient
        if let Some(preferred_patient) = doctor.switching_patients.iter().find(|p| !visited.contains(&p.id)) {
            return actual_restricted_dfs(preferred_patient.id, goal_id, state, path, visited);
        } else {
            // else: no cycle
            return false;
        }
    }

    assert!(false); // Shouldnt hit
    return false;
}

/// Statistics about cycle lengths found during TTC execution
#[derive(Debug, Clone, Default)]
pub struct CycleStats {
    pub cycle_lengths: Vec<usize>,
}

impl CycleStats {
    pub fn new() -> Self {
        Self {
            cycle_lengths: Vec::new(),
        }
    }

    pub fn record_cycle(&mut self, length: usize) {
        self.cycle_lengths.push(length);
    }

    pub fn total_cycles(&self) -> usize {
        self.cycle_lengths.len()
    }

    pub fn avg_cycle_length(&self) -> f64 {
        if self.cycle_lengths.is_empty() {
            0.0
        } else {
            self.cycle_lengths.iter().sum::<usize>() as f64 / self.cycle_lengths.len() as f64
        }
    }

    pub fn max_cycle_length(&self) -> usize {
        self.cycle_lengths.iter().copied().max().unwrap_or(0)
    }

    pub fn min_cycle_length(&self) -> usize {
        self.cycle_lengths.iter().copied().min().unwrap_or(0)
    }

    /// Returns a histogram of cycle lengths: (length, count)
    pub fn length_distribution(&self) -> Vec<(usize, usize)> {
        let mut counts: HashMap<usize, usize> = HashMap::new();
        for &len in &self.cycle_lengths {
            *counts.entry(len).or_insert(0) += 1;
        }
        let mut dist: Vec<_> = counts.into_iter().collect();
        dist.sort_by_key(|(len, _)| *len);
        dist
    }
}

pub struct TTCResultWithStats {
    pub solution: HashSet<usize>,
    pub cycles_found: usize,
    pub patients_reassigned: usize,
    pub patients_pruned: usize,
    pub remaining_capacity: usize,
    // New metrics
    pub cycle_stats: CycleStats,
    pub initial_unsatisfied: usize,      // Patients wanting to switch at start
    pub final_unsatisfied: usize,        // Patients still wanting to switch at end
    pub initial_unassigned: usize,       // Patients without doctor at start
    pub final_unassigned: usize,         // Patients without doctor at end
    pub total_capacity: usize,           // Total doctor capacity
    pub initial_capacity_used: usize,    // Capacity used at start
}

impl TTCResultWithStats {
    /// Calculate satisfaction rate: proportion of unhappy patients who got satisfied
    pub fn satisfaction_rate(&self) -> f64 {
        if self.initial_unsatisfied == 0 {
            1.0
        } else {
            let satisfied = self.initial_unsatisfied - self.final_unsatisfied;
            satisfied as f64 / self.initial_unsatisfied as f64
        }
    }

    /// Calculate unassigned resolution rate
    pub fn unassigned_resolution_rate(&self) -> f64 {
        if self.initial_unassigned == 0 {
            1.0
        } else {
            let resolved = self.initial_unassigned - self.final_unassigned;
            resolved as f64 / self.initial_unassigned as f64
        }
    }

    /// Calculate capacity utilization (proportion of capacity filled)
    pub fn capacity_utilization(&self) -> f64 {
        if self.total_capacity == 0 {
            0.0
        } else {
            let final_used = self.total_capacity - self.remaining_capacity;
            final_used as f64 / self.total_capacity as f64
        }
    }

    /// Calculate initial capacity utilization
    pub fn initial_capacity_utilization(&self) -> f64 {
        if self.total_capacity == 0 {
            0.0
        } else {
            self.initial_capacity_used as f64 / self.total_capacity as f64
        }
    }
}

pub struct TTCResult {
    pub cycles_found: usize,
    pub patients_reassigned: usize,
}

// Re-export the SCC-based TTC solver for easy access
// pub use ttc_scc::{SCCStats, TTCSCCSolver};


// Find cycle starting from a patient using DFS with local pruning
fn find_cycle_from_patient_with_direct_pruning(
    start_patient_id: usize,
    state: &mut TTCState,
) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    let mut path_set = std::collections::HashSet::new();
    let mut visited = std::collections::HashSet::new();

    let found_target_cycle = dfs_for_cycle_with_tracking(
        start_patient_id,
        start_patient_id,
        &mut path,
        &mut path_set,
        &mut visited,
        state,
    );

    if found_target_cycle {
        Some(path)
    } else {
        None
    }
}


fn dfs_for_cycle_with_tracking(
    current_patient_id: usize,
    target_patient_id: usize,
    path: &mut Vec<usize>,
    path_set: &mut std::collections::HashSet<usize>,
    visited: &mut std::collections::HashSet<usize>,
    state: &mut TTCState,
) -> bool {
    if path.len() > 1 && current_patient_id == target_patient_id {
        return true; // Found cycle back to start
    }

    if path_set.contains(&current_patient_id) {
        return false;
    }

    if visited.contains(&current_patient_id) {
        return false;
    }

    let (preferred_doctor_id, is_dummy) = match state.get_patient(current_patient_id) {
        Some(p) => {
            if !p.wants_to_switch || p.is_stuck {
                return false;
            }
            (p.preferred_doctor, p.is_dummy)
        },
        None => return false,
    };

    // Check if this is a structural dead end: preferred doctor has no switching patients
    // But DON'T mark dummy patients as stuck - they represent capacity slots
    let num_switching = match state.get_doctor(preferred_doctor_id) {
        Some(d) => d.switching_patients.len(),
        None => 0,
    };

    if num_switching == 0 && !is_dummy {
        // This patient can NEVER be in a cycle - their preferred doctor has no one to trade
        // Mark globally stuck and remove from current doctor's switching list
        let current_doctor_id = state.get_patient(current_patient_id).and_then(|p| p.current_doctor);
        if let Some(p) = state.get_patient_mut(current_patient_id) {
            p.is_stuck = true;
        }
        // Remove from current doctor's switching_patients so the stuck status propagates
        if let Some(doc_id) = current_doctor_id {
            if let Some(doc) = state.get_doctor_mut(doc_id) {
                doc.switching_patients.retain(|p| p.id != current_patient_id);
            }
        }
        return false;
    }

    visited.insert(current_patient_id);
    path.push(current_patient_id);
    path_set.insert(current_patient_id);

    // Visit switching patients of this doctor in priority order
    // Use while loop since list may shrink during iteration (stuck patients get removed)
    let mut i = 0;
    while let Some(num) = state.get_doctor(preferred_doctor_id).map(|d| d.switching_patients.len()) {
        if i >= num {
            break;
        }
        
        let next_patient_id = match state.get_doctor(preferred_doctor_id) {
            Some(d) => d.switching_patients[i].id,
            None => break,
        };

        let found_cycle = dfs_for_cycle_with_tracking(
            next_patient_id,
            target_patient_id,
            path,
            path_set,
            visited,
            state,
        );

        if found_cycle {
            return true;
        }
        
        // Only increment if the patient wasn't removed (still at same index)
        let still_there = state.get_doctor(preferred_doctor_id)
            .map(|d| d.switching_patients.get(i).map(|p| p.id) == Some(next_patient_id))
            .unwrap_or(false);
        if still_there {
            i += 1;
        }
        // If patient was removed, don't increment - next patient shifted into this index
    }

    // Re-check: if preferred doctor now has 0 switching patients, mark current as stuck
    // But DON'T mark dummy patients as stuck - they represent capacity slots
    let final_num_switching = state.get_doctor(preferred_doctor_id)
        .map(|d| d.switching_patients.len())
        .unwrap_or(0);
    
    if final_num_switching == 0 && !is_dummy {
        let current_doctor_id = state.get_patient(current_patient_id).and_then(|p| p.current_doctor);
        if let Some(p) = state.get_patient_mut(current_patient_id) {
            p.is_stuck = true;
        }
        if let Some(doc_id) = current_doctor_id {
            if let Some(doc) = state.get_doctor_mut(doc_id) {
                doc.switching_patients.retain(|p| p.id != current_patient_id);
            }
        }
    }

    path.pop();
    path_set.remove(&current_patient_id);
    false
}

fn execute_cycle(cycle: &[usize], state: &mut TTCState) {
    if cycle.len() < 2 {
        return;
    }

    // Remove patients from old doctors' switching lists
    for &patient_id in cycle {
        state.resolve_patient(patient_id);
    }
}


// ====== Solution comparing ========
/// Takes two solutions, compares lexicographically
/// returns 1 if first is largest
/// 2 if second is largest 
/// 0 is equal
fn compare_solutions_lexicographic_priority(s1: HashSet<usize>, s2: HashSet<usize>) -> usize {
    let mut s1_prios: Vec<usize> = s1.iter().map(|id| {*id}).collect();
    let mut s2_prios: Vec<usize> = s2.iter().map(|id| {*id}).collect();

    s1_prios.sort();
    s1_prios.reverse();
    s2_prios.sort();
    s2_prios.reverse();

    let first_length = s1_prios.len() - 1;
    let second_length = s2_prios.len() - 1;
    if first_length < second_length {println!("First is smallest")}
    else if second_length < first_length {println!("Second is smallest")}
    else {println!("They are equal long")}

    let min_length = usize::min(first_length,second_length);
    if min_length < s1_prios.len() && min_length < s2_prios.len() {
        println!("first: \n{:?}", &s1_prios[0..=min_length]);
        println!("second: \n{:?}", &s2_prios[0..=min_length]);
    }
    

    let mut i = 0;
    while i < s1_prios.len() && i < s2_prios.len() {
        if s1_prios[i] > s2_prios[i] {
            return 1;
        }
        
        if s2_prios[i] > s1_prios[i] {
            return 2;
        }
        i += 1;
    }

    if i >= s1_prios.len() && i < s2_prios.len() {
        return 2;
    }

    if i >= s2_prios.len() && i < s1_prios.len() {
        return 1;
    }

    return 0;
}