use crate::excact::{CardCyclePacker, PwCyclePacker, UtilCyclePacker, util_exp_weight};
use crate::{CycleStats, Doctor, Patient, ResultWithStats, AssignmentState};
use indicatif::{ProgressBar, ProgressStyle};
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::{HashMap, HashSet};

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
    pub avg_wait_days: f64,
    pub max_wait_days: usize,
}

/// Wait-time threshold (days) above which a wait is counted as "starved".
pub const STARVATION_THRESHOLD_DAYS: usize = 90;

/// Summary of a set of wait times, derived from a histogram so any percentile
/// is exact and cheap to compute.
#[derive(Clone, Debug, Default)]
pub struct WaitSummary {
    pub count: usize,
    pub avg: f64,
    pub std: f64,
    pub max: usize,
    pub p50: usize,
    pub p90: usize,
    pub p95: usize,
    pub p99: usize,
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

    // --- Waitlist health over the run ---
    pub min_waitlist_size: usize,
    pub max_waitlist_size: usize,
    pub final_waitlist_size: usize,

    // --- Wait-time stats ---
    /// Completed waits: one entry per resolved request, wait = days from request to resolution.
    pub resolved_wait: WaitSummary,
    /// Censored waits: patients still on the waitlist at run end (never resolved get the full run length).
    pub outstanding_wait: WaitSummary,
    /// Combined max over resolved + outstanding (the headline "longest anyone ever waited").
    pub overall_max_wait: usize,
    /// Combined mean over resolved + outstanding waits.
    pub overall_avg_wait: f64,
    pub starvation_threshold_days: usize,
    pub starved_resolved: usize,
    pub starved_outstanding: usize,

    /// Raw histograms (index = wait days, value = count). Persisted so any
    /// further statistic can be recomputed from a finished run without re-running.
    pub wait_hist_resolved: Vec<u64>,
    pub wait_hist_outstanding: Vec<u64>,
}

fn hist_add(hist: &mut Vec<u64>, wait: usize) {
    if wait >= hist.len() {
        hist.resize(wait + 1, 0);
    }
    hist[wait] += 1;
}

fn hist_total(h: &[u64]) -> u64 {
    h.iter().sum()
}

fn hist_sum(h: &[u64]) -> u128 {
    h.iter().enumerate().map(|(w, &c)| w as u128 * c as u128).sum()
}

fn hist_sumsq(h: &[u64]) -> u128 {
    h.iter().enumerate().map(|(w, &c)| (w as u128) * (w as u128) * c as u128).sum()
}

fn hist_max(h: &[u64]) -> usize {
    for w in (0..h.len()).rev() {
        if h[w] > 0 {
            return w;
        }
    }
    0
}

/// Smallest wait `w` such that at least `pct`% of the mass is <= `w`.
fn hist_percentile(h: &[u64], pct: f64) -> usize {
    let total = hist_total(h);
    if total == 0 {
        return 0;
    }
    let target = (pct / 100.0 * total as f64).ceil().max(1.0) as u64;
    let mut cum = 0u64;
    for (w, &c) in h.iter().enumerate() {
        cum += c;
        if cum >= target {
            return w;
        }
    }
    hist_max(h)
}

fn hist_count_above(h: &[u64], threshold: usize) -> u64 {
    h.iter().enumerate().filter(|(w, _)| *w > threshold).map(|(_, &c)| c).sum()
}

fn wait_summary_from_hist(h: &[u64]) -> WaitSummary {
    let total = hist_total(h);
    if total == 0 {
        return WaitSummary::default();
    }
    let n = total as f64;
    let avg = hist_sum(h) as f64 / n;
    let var = (hist_sumsq(h) as f64 / n) - avg * avg;
    WaitSummary {
        count: total as usize,
        avg,
        std: if var > 0.0 { var.sqrt() } else { 0.0 },
        max: hist_max(h),
        p50: hist_percentile(h, 50.0),
        p90: hist_percentile(h, 90.0),
        p95: hist_percentile(h, 95.0),
        p99: hist_percentile(h, 99.0),
    }
}

impl SimulationResult {
    /// Per-algorithm summary. Per-day rows are not printed here (they flood the
    /// console for long runs); they are written to the day CSV instead.
    pub fn print_table(&self) {
        println!("\n=== {} ===", self.algorithm_name);
        println!(
            "Patients: {}, Doctors: {}, Days: {}",
            self.num_patients, self.num_doctors, self.num_days
        );
        println!(
            "Throughput : total resolved {}  avg sat {:.1}%  avg cycles/day {:.2}  avg cycle len {:.2}",
            self.total_resolved,
            self.avg_daily_satisfaction_rate * 100.0,
            self.avg_cycles_per_day,
            self.avg_cycle_length_overall,
        );
        println!(
            "Waitlist   : avg {:.1}  min {}  max {}  final {}",
            self.avg_waitlist_size,
            self.min_waitlist_size,
            self.max_waitlist_size,
            self.final_waitlist_size,
        );
        let r = &self.resolved_wait;
        println!(
            "Wait (resolved, n={}): avg {:.1}d  std {:.1}  p50 {}  p90 {}  p95 {}  p99 {}  max {}d",
            r.count, r.avg, r.std, r.p50, r.p90, r.p95, r.p99, r.max,
        );
        let o = &self.outstanding_wait;
        println!(
            "Wait (outstanding, n={}): avg {:.1}d  p50 {}  p90 {}  p95 {}  p99 {}  max {}d",
            o.count, o.avg, o.p50, o.p90, o.p95, o.p99, o.max,
        );
        println!(
            "Wait (overall)        : avg {:.1}d  max {}d",
            self.overall_avg_wait, self.overall_max_wait,
        );
        let res_pct = if r.count > 0 {
            self.starved_resolved as f64 / r.count as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "Starved (>{}d): {} resolved ({:.1}%)  {} outstanding",
            self.starvation_threshold_days, self.starved_resolved, res_pct, self.starved_outstanding,
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
            p.priority = 0;
            p.is_stuck = false;
            p.wait_days = 0;
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

    // Create patients: priority starts at 0, incremented each day they wait
    let mut patients: Vec<Patient> = Vec::with_capacity(n);
    for id in 1..=n {
        let current_doctor = slots[id - 1];
        patients.push(Patient::new(id, false, 0, current_doctor, Some(current_doctor)));
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

    // Running histogram of completed (resolved) wait times across the whole run,
    // indexed by wait days. Built incrementally as patients resolve each day.
    let mut wait_hist_resolved: Vec<u64> = Vec::new();

    let pb = ProgressBar::new(num_days as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40}] Day {pos}/{len} ({elapsed} elapsed, eta {eta})")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message(format!("{}", config.algorithm_name));

    for day in 0..num_days {
        // Age waiting patients (priority += 1, wait_days += 1 per day on waitlist) and reset stuck flags
        for p in &mut state.patients {
            if p.wants_to_switch && !p.is_dummy {
                p.priority += 1;
                p.wait_days += 1;
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

        // Snapshot wait_days for all currently waiting patients before algorithm runs
        let wait_snapshot: HashMap<usize, usize> = state.patients.iter()
            .filter(|p| !p.is_dummy && p.wants_to_switch)
            .map(|p| (p.id, p.wait_days))
            .collect();

        let result = (config.algorithm)(&mut state);

        // Compute wait time stats for patients resolved this day
        let resolved_waits: Vec<usize> = state.patients.iter()
            .filter(|p| !p.is_dummy && !p.wants_to_switch)
            .filter_map(|p| wait_snapshot.get(&p.id).copied())
            .collect();
        let avg_wait = if resolved_waits.is_empty() {
            0.0
        } else {
            resolved_waits.iter().sum::<usize>() as f64 / resolved_waits.len() as f64
        };
        let max_wait = resolved_waits.iter().copied().max().unwrap_or(0);

        // Fold this day's completed waits into the run-wide histogram. This is
        // the "running max/avg updated every time someone is resolved" — every
        // resolved patient contributes its wait at the moment of resolution.
        for &w in &resolved_waits {
            hist_add(&mut wait_hist_resolved, w);
        }

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
            avg_wait_days: avg_wait,
            max_wait_days: max_wait,
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

    // Censored waits: patients still on the waitlist after the final day. A
    // patient who was never resolved has had wait_days incremented every day it
    // waited, so its value reflects the full outstanding wait (up to num_days).
    let mut wait_hist_outstanding: Vec<u64> = Vec::new();
    for p in &state.patients {
        if !p.is_dummy && p.wants_to_switch {
            hist_add(&mut wait_hist_outstanding, p.wait_days);
        }
    }

    let resolved_wait = wait_summary_from_hist(&wait_hist_resolved);
    let outstanding_wait = wait_summary_from_hist(&wait_hist_outstanding);

    // Combined (resolved + outstanding) headline figures.
    let overall_count = hist_total(&wait_hist_resolved) + hist_total(&wait_hist_outstanding);
    let overall_avg_wait = if overall_count == 0 {
        0.0
    } else {
        (hist_sum(&wait_hist_resolved) + hist_sum(&wait_hist_outstanding)) as f64 / overall_count as f64
    };
    let overall_max_wait = resolved_wait.max.max(outstanding_wait.max);

    let starved_resolved = hist_count_above(&wait_hist_resolved, STARVATION_THRESHOLD_DAYS) as usize;
    let starved_outstanding = hist_count_above(&wait_hist_outstanding, STARVATION_THRESHOLD_DAYS) as usize;

    let min_waitlist_size = day_stats.iter().map(|s| s.waitlist_size_before).min().unwrap_or(0);
    let max_waitlist_size = day_stats.iter().map(|s| s.waitlist_size_before).max().unwrap_or(0);
    let final_waitlist_size = day_stats.last().map(|s| s.waitlist_size_after).unwrap_or(0);

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
        min_waitlist_size,
        max_waitlist_size,
        final_waitlist_size,
        resolved_wait,
        outstanding_wait,
        overall_max_wait,
        overall_avg_wait,
        starvation_threshold_days: STARVATION_THRESHOLD_DAYS,
        starved_resolved,
        starved_outstanding,
        wait_hist_resolved,
        wait_hist_outstanding,
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

    let mut packer = CardCyclePacker::new(state);

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

fn run_util_with_prio(
    state: &mut AssignmentState,
    prio: impl Fn(&Patient) -> i128,
) -> ResultWithStats {
    let initial_unsatisfied = state.count_unsatisfied_patients();
    let initial_unassigned = state.count_unassigned_patients();
    let total_capacity = state.get_total_capacity();
    let initial_capacity_used = state.get_capacity_used();

    let mut packer = UtilCyclePacker::new(state, prio);
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

pub fn run_util_linear(state: &mut AssignmentState) -> ResultWithStats {
    run_util_with_prio(state, |p| p.priority as i128)
}

pub fn run_util_exp_1_1(state: &mut AssignmentState) -> ResultWithStats {
    run_util_with_prio(state, |p| util_exp_weight(1.1, p.priority))
}

pub fn run_util_exp_1_5(state: &mut AssignmentState) -> ResultWithStats {
    run_util_with_prio(state, |p| util_exp_weight(1.5, p.priority))
}

pub fn run_util_exp_1_9(state: &mut AssignmentState) -> ResultWithStats {
    run_util_with_prio(state, |p| util_exp_weight(1.9, p.priority))
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
