use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;
use heuristic_ttc::{
    TTCState,
    operators::{InsertOneBetween, Operator, RemoveOneIfEdge},
    operators::{RandomRemoveAndAddCycle, RandomRemoveOneAndRepair},
    simulated_annealing::run_simulated_annealing_timed,
    solution::{ScoringStrategy, Solution},
    parse_data_file,
};

fn main() {
    let data_files: Vec<(&str, &str)> = vec![
        ("100p_15d",       "../ttc/data/test_100_patient_15_doctors_0_unassigned.txt"),
        ("1000p_30d",      "../ttc/data/test_1000_patient_30_doctors_0_unassigned.txt"),
        ("10000p_150d",    "../ttc/data/test_10000_patient_150_doctors_0_unassigned.txt"),
        ("100000p_1500d",  "../ttc/data/test_100000_patient_1500_doctors_0_unassigned.txt"),
    ];

    let exact_times = read_exact_times("../ttc/benchmark_scaling.csv");

    let op1 = InsertOneBetween;
    let op2 = RemoveOneIfEdge;
    let op3 = RandomRemoveAndAddCycle;
    let op4 = RandomRemoveOneAndRepair;
    let operators: [&dyn Operator; 4] = [&op1, &op2, &op3, &op4];

    struct Row {
        dataset: String,
        num_patients: usize,
        num_doctors: usize,
        satisfied: usize,
        wanting: usize,
        actual_ms: u128,
    }
    let mut rows: Vec<Row> = Vec::new();

    for (dataset, file) in &data_files {
        println!("\n=== {} ({}) ===", dataset, file);

        let (patients, doctors) = match parse_data_file(file) {
            Ok(d) => d,
            Err(e) => { eprintln!("  Failed to parse: {}", e); continue; }
        };

        let num_patients = patients.iter().filter(|p| !p.is_dummy).count();
        let num_doctors  = doctors.iter().filter(|d| !d.is_dummy).count();
        let wanting      = patients.iter().filter(|p| !p.is_dummy && p.wants_to_switch).count();

        let exact_ms  = exact_times.get(*dataset).copied().unwrap_or(0);
        let budget_ms = exact_ms.max(10_000);
        println!("  Budget: {}ms  (exact was {}ms)", budget_ms, exact_ms);

        let state    = TTCState::new(patients, doctors);
        let init_sol = Solution::new(vec![], &state);

        let t0 = std::time::Instant::now();
        let best = run_simulated_annealing_timed(
            &init_sol,
            &state,
            &operators,
            Duration::from_millis(budget_ms as u64),
            ScoringStrategy::ByCardinality,
            0.9,
            1e-3,
        );
        let actual_ms = t0.elapsed().as_millis();

        let (valid, true_satisfied) = verify_solution(&best, &state);
        let reported_score: usize = best
            .score(&ScoringStrategy::ByCardinality)
            .to_str_radix(10)
            .parse()
            .unwrap_or(0);

        println!("  Cycles:         {}", best.cycles.len());
        println!("  Reported score: {}", reported_score);
        println!("  True satisfied: {}", true_satisfied);
        println!("  Valid:          {}", valid);
        println!("  Time:           {}ms", actual_ms);

        if reported_score != true_satisfied {
            eprintln!("  [BUG] score() disagrees with true count: {} vs {}", reported_score, true_satisfied);
        }

        rows.push(Row { dataset: dataset.to_string(), num_patients, num_doctors,
                        satisfied: true_satisfied, wanting, actual_ms });
    }

    let out = "../ttc/benchmark_scaling_sa.csv";
    match File::create(out) {
        Ok(mut f) => {
            writeln!(f, "dataset,num_patients,num_doctors,algorithm,patients_satisfied,patients_wanting_switch,satisfaction_rate,time_ms").unwrap();
            for r in &rows {
                let rate = if r.wanting > 0 { r.satisfied as f64 / r.wanting as f64 } else { 0.0 };
                writeln!(f, "{},{},{},SA_Heuristic,{},{},{:.4},{}",
                    r.dataset, r.num_patients, r.num_doctors,
                    r.satisfied, r.wanting, rate, r.actual_ms).unwrap();
            }
            println!("\nResults saved to {}", out);
        }
        Err(e) => eprintln!("Failed to write CSV: {}", e),
    }
}

/// Verify solution integrity and return (is_valid, true_real_patients_satisfied).
///
/// Cycles are stored as [a, b, c, a] — the first node is repeated at the end.
/// This function strips that duplicate before counting and checking edges.
fn verify_solution(solution: &Solution, state: &TTCState) -> (bool, usize) {
    let mut all_ids: Vec<usize> = Vec::new();
    let mut ok = true;

    for (ci, cycle) in solution.cycles.iter().enumerate() {
        if cycle.len() < 2 {
            eprintln!("  [WARN] Cycle {} too short (len={})", ci, cycle.len());
            ok = false;
            continue;
        }

        if cycle[0] != *cycle.last().unwrap() {
            eprintln!("  [WARN] Cycle {} not closed (first={}, last={})",
                ci, cycle[0], cycle.last().unwrap());
            ok = false;
        }

        let nodes = &cycle[..cycle.len() - 1]; // strip repeated last node

        for &id in nodes {
            all_ids.push(id);
        }

        for i in 0..nodes.len() {
            let a_id = nodes[i];
            let b_id = cycle[i + 1];
            let Some(a) = state.get_patient(a_id) else {
                eprintln!("  [WARN] Cycle {} unknown patient {}", ci, a_id);
                ok = false;
                continue;
            };
            let Some(b) = state.get_patient(b_id) else {
                eprintln!("  [WARN] Cycle {} unknown patient {}", ci, b_id);
                ok = false;
                continue;
            };
            if a.preferred_doctor != b.current_doctor.unwrap_or(usize::MAX) {
                eprintln!("  [WARN] Cycle {} invalid edge: patient {} prefers doc {} but patient {} is at {:?}",
                    ci, a_id, a.preferred_doctor, b_id, b.current_doctor);
                ok = false;
            }
        }
    }

    let unique: HashSet<usize> = all_ids.iter().cloned().collect();
    if unique.len() != all_ids.len() {
        eprintln!("  [WARN] {} patient IDs appear in multiple cycles",
            all_ids.len() - unique.len());
        ok = false;
    }

    let true_satisfied = all_ids.iter()
        .filter(|&&id| state.get_patient(id).map(|p| !p.is_dummy).unwrap_or(false))
        .count();

    (ok, true_satisfied)
}

/// Read CyclePacker runtimes keyed by dataset label from benchmark_scaling.csv.
fn read_exact_times(csv_path: &str) -> std::collections::HashMap<String, u128> {
    let mut map = std::collections::HashMap::new();
    let Ok(file) = File::open(csv_path) else { return map; };
    for line in BufReader::new(file).lines().skip(1) {
        let Ok(line) = line else { continue; };
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 8 && cols[3] == "CyclePacker" {
            if let Ok(ms) = cols[7].parse::<u128>() {
                map.insert(cols[0].to_string(), ms);
            }
        }
    }
    map
}
