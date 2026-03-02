use std::{collections::HashMap, fs};

pub mod solution;
pub mod operators;
pub mod local_search;
pub mod simulated_annealing;
pub mod cycle_ilp;

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
        self.switching_patients.push(patient);
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
