pub mod benchmarking;
pub mod scc;
pub mod ttc_scc;
use std::{collections::HashMap};
use std::fs;

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
            i + 1, // Patient ID starts from 1
            priorities[i],
            preferred_doctors[i],
            current_doctors[i],
        );
        patients.push(patient);
    }

    // Create dummy doctor (ID 0) for unassigned patients
    let unassigned_count = current_doctors.iter().filter(|d| **d == Some(0)).count();
    let mut dummy_doctor = Doctor::new_with_capacity(0, unassigned_count);

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

        let mut doctor = Doctor::new_with_capacity(i, capacity);

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


// Optimized version with pruning
pub fn ttc_algorithm_with_pruning(state: &mut TTCState) -> TTCResultWithStats {
    let mut cycles_found = 0;
    let mut total_patients_reassigned = 0;

    println!("[DFS] Starting DFS algorithm...");

    // Only include real patients and unassigned patients, not dummy capacity patients
    let switching_patients: Vec<usize> = state
        .patients_by_priority
        .iter()
        .filter(|&&id| {
            state.get_patient(id).map_or(false, |p| {
                p.wants_to_switch && p.priority != usize::MAX  // Exclude dummy capacity patients
            })
        })
        .copied()
        .collect();

    for (_i, &patient_id) in switching_patients.iter().enumerate() {
        let _patient = match state.get_patient(patient_id) {
            Some(p) if p.wants_to_switch => p,
            _ => {
                continue; // Skip happy patients
            }
        };
        if let Some(cycle) = find_cycle_from_patient_with_direct_pruning(patient_id, state) {
            cycles_found += 1;
            total_patients_reassigned += cycle.len();

            // println!("🔍 [DFS] Cycle #{}: {} patients: {:?}", cycles_found, cycle.len(), cycle);


            execute_cycle(&cycle, state);
        }
    }

    // let patients_pruned = state.patients.iter().filter(|p| p.is_stuck).count();

    TTCResultWithStats {
        cycles_found,
        patients_reassigned: total_patients_reassigned,
        patients_pruned: 5,
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


// Optimized version with direct marking (no HashSet needed)
fn find_cycle_from_patient_with_direct_pruning(
    start_patient_id: usize,
    state: &mut TTCState,
) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    let mut path_set = std::collections::HashSet::new();
    let mut visited = std::collections::HashSet::new();
    let mut found_any_cycle = false;

    let (found_target_cycle, _) = dfs_for_cycle_with_tracking(
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
        // No cycle found - patients were already marked during DFS backtrack
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
) -> (bool, bool) {
    if path.len() > 1 && current_patient_id == target_patient_id {
        return (true, true); // Found cycle back to start
    }

    if path_set.contains(&current_patient_id) {
        return (false, true);
    }

    if visited.contains(&current_patient_id) {
        // We've explored this node before in a previous branch
        // Check if it was marked as stuck - if so, it leads nowhere
        if let Some(patient) = state.get_patient(current_patient_id) {
            return (false, !patient.is_stuck);
        }
        return (false, false);
    }

    let current_patient = match state.get_patient(current_patient_id) {
        Some(p) => p,
        None => {
            return (false, false);
        }
    };

    if current_patient.is_stuck || !current_patient.wants_to_switch {
        return (false, false);
    }

    visited.insert(current_patient_id);
    path.push(current_patient_id);
    path_set.insert(current_patient_id);

    // Get preferred doctor ID and number of switching patients to avoid borrowing conflicts
    let preferred_doctor_id = current_patient.preferred_doctor;
    let num_switching = match state.get_doctor(preferred_doctor_id) {
        Some(d) => d.switching_patients.len(),
        None => {
            path.pop();
            return (false, false);
        }
    };

    let mut found_any_in_any_subtree = false;

    // Visit switching patients of this doctor in priority order (already sorted)
    // Re-fetch doctor each iteration to avoid Vec allocation
    for i in 0..num_switching {
        let next_patient_id = match state.get_doctor(preferred_doctor_id) {
            Some(d) => d.switching_patients[i].id,
            None => continue,
        };

        let (found_cycle, found_any_cycle_in_subtree) = dfs_for_cycle_with_tracking(
            next_patient_id,
            target_patient_id,
            path,
            path_set,
            visited,
            state,
        );

        if found_cycle {
            return (true, true);
        }

        if found_any_cycle_in_subtree {
            found_any_in_any_subtree = true;
        } else {
            if let Some(patient) = state.get_patient_mut(next_patient_id) {
                patient.is_stuck = true;
            }
        }
    }


    path.pop();
    path_set.remove(&current_patient_id);
    (false, found_any_in_any_subtree)
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
