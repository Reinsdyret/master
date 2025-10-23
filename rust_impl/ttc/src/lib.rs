pub mod benchmarking;
pub mod graph;
pub mod scc;
pub mod ttc_scc;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Patient {
    pub id: usize,
    pub priority: usize,
    pub preferred_doctor: usize,
    pub current_doctor: Option<usize>, // Changed to Option to support unassigned patients
    pub wants_to_switch: bool,
    pub is_stuck: bool, // Marked as unable to be satisfied (pruned)
}

impl Patient {
    pub fn new(id: usize, priority: usize, preferred_doctor: usize, current_doctor: Option<usize>) -> Self {
        let wants_to_switch = match current_doctor {
            Some(doctor_id) => preferred_doctor != doctor_id,
            None => true, // Unassigned patients always want to switch
        };
        Patient {
            id,
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
    pub capacity: usize, // Maximum number of patients this doctor can serve
    pub switching_patients: Vec<Patient>,
    pub assigned_patients: Vec<usize>, // All currently assigned patient IDs
}

impl Doctor {
    pub fn new(id: usize) -> Self {
        Doctor {
            id,
            capacity: 0, // Will be set during initialization
            switching_patients: Vec::new(),
            assigned_patients: Vec::new(),
        }
    }

    pub fn new_with_capacity(id: usize, capacity: usize) -> Self {
        Doctor {
            id,
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
    pub fn available_capacity(&self) -> usize {
        self.capacity.saturating_sub(self.assigned_patients.len())
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

use std::collections::HashMap;
use std::fs;
use std::io::Write;

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

    // Parse current doctors, supporting 0 or empty string for unassigned patients
    let current_doctors: Vec<Option<usize>> = lines[2]
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "0" {
                Ok(None) // Unassigned patient
            } else {
                trimmed.parse::<usize>().map(Some)
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid current doctor values")?;

    let priorities: Vec<usize> = lines[3]
        .split(',')
        .map(|s| s.parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid priority values")?;

    // Parse optional capacity information (5th line)
    let capacities: Option<Vec<usize>> = if lines.len() >= 5 {
        Some(
            lines[4]
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

    let mut patients = Vec::new();
    for i in 0..num_patients {
        let patient = Patient::new(
            i + 1, // Patient ID starts from 1
            priorities[i],
            preferred_doctors[i],
            current_doctors[i],
        );
        patients.push(patient);
    }

    // Initialize doctors with capacity information
    let mut doctors = Vec::new();
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
        
        let mut doctor = Doctor::new_with_capacity(i, capacity);
        
        // Initialize assigned_patients list based on current assignments
        for patient in &patients {
            if patient.current_doctor == Some(i) {
                doctor.assigned_patients.push(patient.id);
            }
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
            // Note: Unassigned patients (current_doctor = None) will be handled 
            // by the dummy doctor mechanism in the enhanced graph building
        }
    }

    Ok((patients, doctors))
}

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
        self.doctors.get(id - 1) // Assuming IDs start from 1
    }

    pub fn get_doctor_mut(&mut self, id: usize) -> Option<&mut Doctor> {
        self.doctors.get_mut(id - 1)
    }
}

pub fn ttc_algorithm(state: &mut TTCState) -> TTCResult {
    use indicatif::{ProgressBar, ProgressStyle};

    let mut cycles_found = 0;
    let mut total_patients_reassigned = 0;

    let switching_patients: Vec<usize> = state
        .patients_by_priority
        .iter()
        .filter(|&&id| state.get_patient(id).map_or(false, |p| p.wants_to_switch))
        .copied()
        .collect();

    let pb = ProgressBar::new(switching_patients.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos:>7}/{len:7} [{elapsed_precise}] {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message("Searching for cycles...");

    for (_i, &patient_id) in switching_patients.iter().enumerate() {
        let _patient = match state.get_patient(patient_id) {
            Some(p) if p.wants_to_switch => p,
            _ => {
                pb.inc(1);
                continue; // Skip happy patients
            }
        };

        pb.set_message(format!(
            "Processing Patient {} (prio {})",
            patient_id,
            state.get_patient(patient_id).map_or(0, |p| p.priority)
        ));

        // Run DFS to find cycle starting from this patient
        if let Some(cycle) = find_cycle_from_patient(patient_id, state) {
            cycles_found += 1;
            total_patients_reassigned += cycle.len();

            pb.set_message(format!(
                "Found cycle #{} with {} patients!",
                cycles_found,
                cycle.len()
            ));

            // Execute the cycle - reassign patients
            execute_cycle(&cycle, state);

            // Mark all patients in cycle as happy
            for &cycle_patient_id in &cycle {
                if let Some(p) = state.get_patient_mut(cycle_patient_id) {
                    p.wants_to_switch = false;
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message(format!(
        "✅ Completed! Found {} cycles, {} patients reassigned",
        cycles_found, total_patients_reassigned
    ));

    TTCResult {
        cycles_found,
        patients_reassigned: total_patients_reassigned,
    }
}

// Optimized version with pruning
pub fn ttc_algorithm_with_pruning(state: &mut TTCState) -> TTCResultWithStats {
    use indicatif::{ProgressBar, ProgressStyle};

    let mut cycles_found = 0;
    let mut total_patients_reassigned = 0;

    println!("🔍 [DFS] Starting DFS algorithm...");

    // Create output file for DFS cycles
    let mut dfs_cycles_file =
        std::fs::File::create("dfs_cycles.txt").expect("Could not create dfs_cycles.txt");

    // Create progress bar
    let switching_patients: Vec<usize> = state
        .patients_by_priority
        .iter()
        .filter(|&&id| state.get_patient(id).map_or(false, |p| p.wants_to_switch))
        .copied()
        .collect();

    let pb = ProgressBar::new(switching_patients.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos:>7}/{len:7} [{elapsed_precise}] {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message("Searching for cycles (with pruning)...");

    for (_i, &patient_id) in switching_patients.iter().enumerate() {
        let _patient = match state.get_patient(patient_id) {
            Some(p) if p.wants_to_switch => p,
            _ => {
                pb.inc(1);
                continue; // Skip happy patients
            }
        };

        pb.set_message(format!(
            "Processing Patient {} (prio {})",
            patient_id,
            state.get_patient(patient_id).map_or(0, |p| p.priority)
        ));

        if let Some(cycle) = find_cycle_from_patient_with_direct_pruning(patient_id, state) {
            cycles_found += 1;
            total_patients_reassigned += cycle.len();

            // Write cycle to file
            writeln!(dfs_cycles_file, "{:?}", cycle)
                .expect("Failed to write cycle to dfs_cycles.txt");

            // println!("🔍 [DFS] Cycle #{}: {} patients: {:?}", cycles_found, cycle.len(), cycle);

            pb.set_message(format!(
                "Found cycle #{} with {} patients!",
                cycles_found,
                cycle.len()
            ));

            execute_cycle(&cycle, state);

            for &cycle_patient_id in &cycle {
                if let Some(p) = state.get_patient_mut(cycle_patient_id) {
                    p.wants_to_switch = false;
                }
            }
        }

        pb.inc(1);
    }

    let patients_pruned = state.patients.iter().filter(|p| p.is_stuck).count();

    TTCResultWithStats {
        cycles_found,
        patients_reassigned: total_patients_reassigned,
        patients_pruned,
    }
}

pub struct TTCResultWithStats {
    pub cycles_found: usize,
    pub patients_reassigned: usize,
    pub patients_pruned: usize,
}

pub struct TTCResult {
    pub cycles_found: usize,
    pub patients_reassigned: usize,
}

// Re-export the SCC-based TTC solver for easy access
pub use ttc_scc::{SCCStats, TTCSCCSolver};

fn find_cycle_from_patient(start_patient_id: usize, state: &TTCState) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    let mut path_set = std::collections::HashSet::new(); // O(1) cycle detection
    let mut visited = std::collections::HashSet::new();

    if dfs_for_cycle(
        start_patient_id,
        start_patient_id,
        &mut path,
        &mut path_set,
        &mut visited,
        state,
    ) {
        Some(path)
    } else {
        None
    }
}

// Optimized version with direct marking (no HashSet needed)
fn find_cycle_from_patient_with_direct_pruning(
    start_patient_id: usize,
    state: &mut TTCState,
) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    let mut path_set = std::collections::HashSet::new(); // O(1) cycle detection
    let mut visited = std::collections::HashSet::new();
    let mut found_any_cycle = false;

    let found_target_cycle = dfs_for_cycle_with_tracking(
        start_patient_id,
        start_patient_id,
        &mut path,
        &mut path_set,
        &mut visited,
        &mut found_any_cycle,
        state,
    );

    if found_target_cycle {
        Some(path)
    } else {
        // No cycle found - patients were already marked during DFS backtrack
        None
    }
}

fn dfs_for_cycle(
    current_patient_id: usize,
    target_patient_id: usize,
    path: &mut Vec<usize>,
    path_set: &mut std::collections::HashSet<usize>, // O(1) cycle detection
    visited: &mut std::collections::HashSet<usize>,
    state: &TTCState,
) -> bool {
    if path.len() > 1 && current_patient_id == target_patient_id {
        return true; // Found cycle back to start
    }

    // O(1) cycle detection within current path
    if path_set.contains(&current_patient_id) {
        return false; // Cycle detected but not target
    }

    if visited.contains(&current_patient_id) {
        return false; // Visited but not target - no cycle
    }

    visited.insert(current_patient_id);
    path.push(current_patient_id);
    path_set.insert(current_patient_id);

    // Find current patient
    let current_patient = match state.get_patient(current_patient_id) {
        Some(p) => p,
        None => {
            path.pop();
            return false;
        }
    };

    // Go to preferred doctor
    let preferred_doctor = match state.get_doctor(current_patient.preferred_doctor) {
        Some(d) => d,
        None => {
            path.pop();
            return false;
        }
    };

    // Visit switching patients of this doctor in priority order (already sorted)
    for next_patient in &preferred_doctor.switching_patients {
        if dfs_for_cycle(
            next_patient.id,
            target_patient_id,
            path,
            path_set,
            visited,
            state,
        ) {
            return true;
        }
    }

    path.pop();
    path_set.remove(&current_patient_id);
    false
}

// DFS that detects ANY cycles (not just ones containing target) for pruning optimization
fn dfs_for_cycle_with_tracking(
    current_patient_id: usize,
    target_patient_id: usize,
    path: &mut Vec<usize>,
    path_set: &mut std::collections::HashSet<usize>, // O(1) cycle detection
    visited: &mut std::collections::HashSet<usize>,
    found_any_cycle: &mut bool,
    state: &mut TTCState,
) -> bool {
    if path.len() > 1 && current_patient_id == target_patient_id {
        *found_any_cycle = true;
        return true; // Found cycle back to start
    }

    // O(1) cycle detection instead of O(n) path.contains()
    if path_set.contains(&current_patient_id) {
        *found_any_cycle = true;
        return false;
    }

    if visited.contains(&current_patient_id) {
        return false;
    }

    let current_patient = match state.get_patient(current_patient_id) {
        Some(p) => p,
        None => {
            path.pop();
            return false;
        }
    };

    if current_patient.is_stuck || !current_patient.wants_to_switch {
        return false;
    }

    visited.insert(current_patient_id);
    path.push(current_patient_id);
    path_set.insert(current_patient_id);

    // Get preferred doctor ID and collect switching patient IDs to avoid borrowing conflicts
    let preferred_doctor_id = current_patient.preferred_doctor;
    let switching_patient_ids: Vec<usize> = match state.get_doctor(preferred_doctor_id) {
        Some(d) => d.switching_patients.iter().map(|p| p.id).collect(),
        None => {
            path.pop();
            return false;
        }
    };

    // Visit switching patients of this doctor in priority order (already sorted)
    for next_patient_id in switching_patient_ids {
        if dfs_for_cycle_with_tracking(
            next_patient_id,
            target_patient_id,
            path,
            path_set,
            visited,
            found_any_cycle,
            state,
        ) {
            return true;
        }
    }

    if !*found_any_cycle {
        if let Some(patient) = state.get_patient_mut(current_patient_id) {
            patient.is_stuck = true;
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

    // Create a mapping of old assignments
    let mut patient_to_new_doctor = std::collections::HashMap::new();

    // In a cycle, each patient gets the current doctor of the next patient in cycle
    for i in 0..cycle.len() {
        let current_patient_id = cycle[i];
        let next_patient_id = cycle[(i + 1) % cycle.len()];

        if let Some(next_patient) = state.get_patient(next_patient_id) {
            patient_to_new_doctor.insert(current_patient_id, next_patient.current_doctor);
        }
    }

    // Remove patients from old doctors' switching lists
    for &patient_id in cycle {
        if let Some(patient) = state.get_patient(patient_id) {
            if let Some(current_doctor_id) = patient.current_doctor {
                if let Some(doctor) = state.get_doctor_mut(current_doctor_id) {
                    doctor.switching_patients.retain(|p| p.id != patient_id);
                }
            }
        }
    }

    // Update patient assignments
    for &patient_id in cycle {
        if let (Some(patient), Some(&new_doctor_id)) = (
            state.get_patient_mut(patient_id),
            patient_to_new_doctor.get(&patient_id),
        ) {
            patient.current_doctor = new_doctor_id;
        }
    }
}
