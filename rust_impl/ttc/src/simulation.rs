use crate::excact::{CyclePacker, PwCyclePacker};
use crate::{CycleStats, Doctor, Patient, ResultWithStats, AssignmentState};
use indicatif::{ProgressBar, ProgressStyle};
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::HashSet;

pub enum NewRequestMode {
    SameAsResolved,
    Fixed(usize),
    Fraction(f64),
}

pub struct SimulationConfig {
    pub num_patients: usize,
    pub num_doctors: usize,
    pub waitlist_fraction: f64,
    pub num_days: usize,
    pub new_requests_per_day: NewRequestMode,
    /// Minimum new requests per day as a fraction of num_patients (floor).
    /// Guards against days where 0 get resolved so the simulation doesn't stall.
    /// E.g. 0.05 means at least 5% of num_patients new requests per day.
    pub min_new_requests_fraction: f64,
    pub algorithm: fn(&mut AssignmentState) -> ResultWithStats,
    pub algorithm_name: String,
    pub seed: u64,
}

pub struct DayStats {
    pub day: usize,
    pub waitlist_size_before: usize,
    pub patients_resolved: usize,
    pub new_requests_added: usize,
    pub waitlist_size_after: usize,
    pub satisfaction_rate: f64,
    pub cycles_found: usize,
    pub avg_cycle_length: f64,
    pub max_cycle_length: usize,
}

pub struct SimulationResult {
    pub algorithm_name: String,
    pub num_patients: usize,
    pub num_doctors: usize,
    pub num_days: usize,
    pub day_stats: Vec<DayStats>,
    pub total_resolved: usize,
    pub avg_daily_satisfaction_rate: f64,
    pub avg_waitlist_size: f64,
    pub avg_cycles_per_day: f64,
    pub avg_cycle_length_overall: f64,
}

impl SimulationResult {
    pub fn print_table(&self) {
        println!("\n=== {} ===", self.algorithm_name);
        println!(
            "Patients: {}, Doctors: {}, Days: {}",
            self.num_patients, self.num_doctors, self.num_days
        );
        println!(
            "{:>4}  {:>10}  {:>9}  {:>8}  {:>10}  {:>7}  {:>7}  {:>8}",
            "Day", "WaitBefore", "Resolved", "NewReq", "WaitAfter", "Sat%", "Cycles", "AvgCyc"
        );
        println!("{}", "-".repeat(75));
        for s in &self.day_stats {
            println!(
                "{:>4}  {:>10}  {:>9}  {:>8}  {:>10}  {:>6.1}%  {:>7}  {:>8.2}",
                s.day + 1,
                s.waitlist_size_before,
                s.patients_resolved,
                s.new_requests_added,
                s.waitlist_size_after,
                s.satisfaction_rate * 100.0,
                s.cycles_found,
                s.avg_cycle_length,
            );
        }
        println!("{}", "-".repeat(75));
        println!(
            "Total resolved: {}  Avg sat: {:.1}%  Avg waitlist: {:.1}  Avg cycles/day: {:.2}  Avg cycle len: {:.2}",
            self.total_resolved,
            self.avg_daily_satisfaction_rate * 100.0,
            self.avg_waitlist_size,
            self.avg_cycles_per_day,
            self.avg_cycle_length_overall,
        );
    }
}

/// Generate non-uniform capacities for num_doctors that sum exactly to num_patients.
fn generate_doctor_capacities(
    num_doctors: usize,
    num_patients: usize,
    rng: &mut impl Rng,
) -> Vec<usize> {
    let mut weights = vec![0.0_f64; num_doctors];
    for w in &mut weights {
        *w = rng.gen_range(0.5..1.5);
    }
    let total_weight: f64 = weights.iter().sum();

    let mut capacities = vec![0_usize; num_doctors];
    for i in 0..num_doctors {
        let raw = ((weights[i] / total_weight) * num_patients as f64).round() as usize;
        capacities[i] = raw.max(1);
    }

    // Adjust sum to be exactly num_patients
    let current_sum: usize = capacities.iter().sum();
    if current_sum < num_patients {
        let mut diff = num_patients - current_sum;
        while diff > 0 {
            let idx = rng.gen_range(0..num_doctors);
            capacities[idx] += 1;
            diff -= 1;
        }
    } else if current_sum > num_patients {
        let mut excess = current_sum - num_patients;
        while excess > 0 {
            let idx = rng.gen_range(0..num_doctors);
            if capacities[idx] > 1 {
                capacities[idx] -= 1;
                excess -= 1;
            }
        }
    }

    capacities
}

/// Rebuild doctor.assigned_patients and doctor.switching_patients from patient.current_doctor.
/// Call this after any batch modification to patient state.
fn rebuild_all_doctor_state(state: &mut AssignmentState) {
    for doc in &mut state.doctors {
        doc.switching_patients.clear();
        doc.assigned_patients.clear();
    }

    for i in 0..state.patients.len() {
        let p = state.patients[i];
        if let Some(doc_id) = p.current_doctor {
            if let Some(doc) = state.doctors.get_mut(doc_id) {
                doc.assigned_patients.push(p.id);
                if p.wants_to_switch && !p.is_dummy {
                    // add_switching_patient maintains priority order via binary search
                    doc.add_switching_patient(p);
                }
            }
        }
    }
}

/// Pick count satisfied patients and make them submit new switch requests.
fn add_new_requests(state: &mut AssignmentState, count: usize, rng: &mut impl Rng) {
    let num_doctors = state.doctors.len() - 1; // doctors[0] is dummy, real are 1..=num_doctors

    let mut candidates: Vec<usize> = Vec::new();
    for p in &state.patients {
        if !p.is_dummy && !p.wants_to_switch {
            candidates.push(p.id);
        }
    }

    candidates.shuffle(rng);
    candidates.truncate(count);

    for &patient_id in &candidates {
        let current_doctor = state
            .get_patient(patient_id)
            .and_then(|p| p.current_doctor)
            .unwrap_or(1);

        let new_pref = loop {
            let candidate = rng.gen_range(1..=num_doctors);
            if candidate != current_doctor {
                break candidate;
            }
        };

        if let Some(p) = state.get_patient_mut(patient_id) {
            p.preferred_doctor = new_pref;
            p.wants_to_switch = true;
            p.priority = 1;
            p.is_stuck = false;
        }
    }

    rebuild_all_doctor_state(state);
}

pub fn init_state(config: &SimulationConfig, rng: &mut impl Rng) -> AssignmentState {
    let n = config.num_patients;
    let num_docs = config.num_doctors;

    let capacities = generate_doctor_capacities(num_docs, n, rng);

    // Build a flat slot list: each entry is the doctor_id for that slot
    let mut slots: Vec<usize> = Vec::with_capacity(n);
    for i in 0..num_docs {
        let doctor_id = i + 1;
        for _ in 0..capacities[i] {
            slots.push(doctor_id);
        }
    }
    slots.shuffle(rng);

    // Create patients: priority = id, initially assigned to their slot
    let mut patients: Vec<Patient> = Vec::with_capacity(n);
    for id in 1..=n {
        let current_doctor = slots[id - 1];
        patients.push(Patient::new(id, false, id, current_doctor, Some(current_doctor)));
    }

    // Create doctors: index 0 is dummy placeholder, real doctors at 1..=num_docs
    let mut doctors: Vec<Doctor> = Vec::with_capacity(num_docs + 1);
    doctors.push(Doctor::new_with_capacity(0, true, 0)); // dummy at index 0
    for i in 0..num_docs {
        doctors.push(Doctor::new_with_capacity(i + 1, false, capacities[i]));
    }

    // Select which patients go on the initial waitlist
    let waitlist_count = (n as f64 * config.waitlist_fraction).round() as usize;
    let mut order: Vec<usize> = (0..n).collect();
    order.shuffle(rng);
    order.truncate(waitlist_count);

    for &idx in &order {
        let current_doctor = patients[idx].current_doctor.unwrap();
        let new_pref = loop {
            let candidate = rng.gen_range(1..=num_docs);
            if candidate != current_doctor {
                break candidate;
            }
        };
        patients[idx].preferred_doctor = new_pref;
        patients[idx].wants_to_switch = true;
    }

    // Populate doctor state from patient data
    for p in &patients {
        if let Some(doc_id) = p.current_doctor {
            doctors[doc_id].assigned_patients.push(p.id);
            if p.wants_to_switch {
                doctors[doc_id].add_switching_patient(*p);
            }
        }
    }

    AssignmentState::new(patients, doctors)
}

pub fn run_simulation(config: SimulationConfig) -> SimulationResult {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut state = init_state(&config, &mut rng);

    let num_patients = config.num_patients;
    let num_doctors = config.num_doctors;
    let num_days = config.num_days;

    let mut day_stats: Vec<DayStats> = Vec::with_capacity(num_days);

    let pb = ProgressBar::new(num_days as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40}] Day {pos}/{len} ({elapsed} elapsed, eta {eta})")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message(format!("{}", config.algorithm_name));

    for day in 0..num_days {
        // Age waiting patients (priority += 1 per day on waitlist) and reset stuck flags
        for p in &mut state.patients {
            if p.wants_to_switch && !p.is_dummy {
                p.priority += 1;
            }
            p.is_stuck = false;
        }

        // Rebuild doctor state and re-sort patients_by_priority
        rebuild_all_doctor_state(&mut state);
        state = AssignmentState::new(
            std::mem::take(&mut state.patients),
            std::mem::take(&mut state.doctors),
        );

        let waitlist_before = state.count_unsatisfied_patients();

        let result = (config.algorithm)(&mut state);

        let new_count = match &config.new_requests_per_day {
            NewRequestMode::SameAsResolved => result.patients_reassigned,
            NewRequestMode::Fixed(n) => *n,
            NewRequestMode::Fraction(f) => (waitlist_before as f64 * f).round() as usize,
        };
        let min_new = (config.num_patients as f64 * config.min_new_requests_fraction).ceil() as usize;
        let new_count = new_count.max(min_new);

        add_new_requests(&mut state, new_count, &mut rng);

        let waitlist_after = state.count_unsatisfied_patients();
        let sat_rate = if waitlist_before == 0 {
            1.0
        } else {
            result.patients_reassigned as f64 / waitlist_before as f64
        };

        day_stats.push(DayStats {
            day,
            waitlist_size_before: waitlist_before,
            patients_resolved: result.patients_reassigned,
            new_requests_added: new_count,
            waitlist_size_after: waitlist_after,
            satisfaction_rate: sat_rate,
            cycles_found: result.cycles_found,
            avg_cycle_length: result.cycle_stats.avg_cycle_length(),
            max_cycle_length: result.cycle_stats.max_cycle_length(),
        });

        pb.inc(1);
    }

    pb.finish_with_message(format!("{} done", config.algorithm_name));

    // Aggregate stats
    let total_resolved: usize = day_stats.iter().map(|s| s.patients_resolved).sum();
    let n_days = day_stats.len() as f64;

    let avg_daily_satisfaction_rate =
        day_stats.iter().map(|s| s.satisfaction_rate).sum::<f64>() / n_days;
    let avg_waitlist_size =
        day_stats.iter().map(|s| s.waitlist_size_before as f64).sum::<f64>() / n_days;
    let avg_cycles_per_day =
        day_stats.iter().map(|s| s.cycles_found as f64).sum::<f64>() / n_days;

    let mut cycle_len_sum = 0.0_f64;
    let mut cycle_len_count = 0_usize;
    for s in &day_stats {
        if s.cycles_found > 0 {
            cycle_len_sum += s.avg_cycle_length;
            cycle_len_count += 1;
        }
    }
    let avg_cycle_length_overall = if cycle_len_count == 0 {
        0.0
    } else {
        cycle_len_sum / cycle_len_count as f64
    };

    SimulationResult {
        algorithm_name: config.algorithm_name,
        num_patients,
        num_doctors,
        num_days,
        day_stats,
        total_resolved,
        avg_daily_satisfaction_rate,
        avg_waitlist_size,
        avg_cycles_per_day,
        avg_cycle_length_overall,
    }
}

fn build_result_from_satisfied(
    state: &mut AssignmentState,
    satisfied_ids: Vec<usize>,
    initial_unsatisfied: usize,
    initial_unassigned: usize,
    total_capacity: usize,
    initial_capacity_used: usize,
) -> ResultWithStats {
    let patients_reassigned = satisfied_ids.len();
    let mut solution: HashSet<usize> = HashSet::new();

    for id in &satisfied_ids {
        if let Some(p) = state.get_patient(*id) {
            solution.insert(p.priority);
        }
    }
    for id in satisfied_ids {
        state.resolve_patient(id);
    }

    ResultWithStats {
        solution,
        cycles_found: 0,
        patients_reassigned,
        patients_pruned: 0,
        remaining_capacity: state.get_total_availability(),
        cycle_stats: CycleStats::new(),
        initial_unsatisfied,
        final_unsatisfied: state.count_unsatisfied_patients(),
        initial_unassigned,
        final_unassigned: state.count_unassigned_patients(),
        total_capacity,
        initial_capacity_used,
    }
}

/// Exact cardinality matching via CyclePacker. Maximises number of satisfied patients.
pub fn run_exact_cardinality(state: &mut AssignmentState) -> ResultWithStats {
    let initial_unsatisfied = state.count_unsatisfied_patients();
    let initial_unassigned = state.count_unassigned_patients();
    let total_capacity = state.get_total_capacity();
    let initial_capacity_used = state.get_capacity_used();

    let mut packer = CyclePacker::new(state);

    let cycle_stats = packer.pack_cycles();

    let satisfied_ids: Vec<usize> = packer
        .satisfied_patients(&state.patients)
        .iter()
        .map(|p| p.id)
        .collect();

    let mut result = build_result_from_satisfied(
        state, satisfied_ids,
        initial_unsatisfied, initial_unassigned,
        total_capacity, initial_capacity_used,
    );
    result.cycles_found = cycle_stats.total_cycles();
    result.cycle_stats = cycle_stats;
    result
}

/// Exact priority-weighted matching via PwCyclePacker. Lexicographically optimal by priority.
pub fn run_exact_priority(state: &mut AssignmentState) -> ResultWithStats {
    let initial_unsatisfied = state.count_unsatisfied_patients();
    let initial_unassigned = state.count_unassigned_patients();
    let total_capacity = state.get_total_capacity();
    let initial_capacity_used = state.get_capacity_used();

    let mut packer = PwCyclePacker::new(state);

    let cycle_stats = packer.pack_cycles();

    let satisfied_ids: Vec<usize> = packer
        .satisfied_patients(&state.patients)
        .iter()
        .map(|p| p.id)
        .collect();

    let mut result = build_result_from_satisfied(
        state, satisfied_ids,
        initial_unsatisfied, initial_unassigned,
        total_capacity, initial_capacity_used,
    );
    result.cycles_found = cycle_stats.total_cycles();
    result.cycle_stats = cycle_stats;
    result
}
