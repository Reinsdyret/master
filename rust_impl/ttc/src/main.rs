use ttc::excact::{CyclePacker, PwCyclePacker};
use ttc::{greedy_dfs, verify_result, PriorityStrategy, AssignmentState, parse_data_file, Patient};
use std::fs::File;
use std::io::Write;
use std::collections::HashSet;

fn lex_compare(
    a_set: &HashSet<usize>,
    b_set: &HashSet<usize>,
    switching: &[&Patient],
    label_a: &str,
    label_b: &str,
    out_path: &str,
) {
    let mut a_leads = 0usize;
    let mut b_leads = 0usize;
    let mut both = 0usize;
    let mut neither = 0usize;

    let mut f = File::create(out_path).expect("failed to create comparison file");
    writeln!(f, "Lex comparison: {} vs {} — sorted by priority descending", label_a, label_b).unwrap();
    writeln!(f, "{:>10}  {:>5}  {:>5}  {:>10}  {:>10}",
        "PatientID", label_a, label_b, "Priority", "Verdict").unwrap();
    writeln!(f, "{}", "-".repeat(50)).unwrap();

    for p in switching {
        let in_a = a_set.contains(&p.priority);
        let in_b = b_set.contains(&p.priority);
        let verdict = match (in_a, in_b) {
            (true,  true)  => { both    += 1; "both" }
            (true,  false) => { a_leads += 1; "A only" }
            (false, true)  => { b_leads += 1; "B only" }
            (false, false) => { neither += 1; "neither" }
        };
        writeln!(f, "{:>10}  {:>5}  {:>5}  {:>10}  {:>10}",
            p.id,
            if in_a { "Y" } else { "N" },
            if in_b { "Y" } else { "N" },
            p.priority,
            verdict).unwrap();
    }

    writeln!(f, "{}", "-".repeat(50)).unwrap();
    writeln!(f, "Both      : {}", both).unwrap();
    writeln!(f, "A only    : {}", a_leads).unwrap();
    writeln!(f, "B only    : {}", b_leads).unwrap();
    writeln!(f, "Neither   : {}", neither).unwrap();
    println!("  Lex comparison written to {}", out_path);

    let first_a_only = switching.iter().find(|p| a_set.contains(&p.priority) && !b_set.contains(&p.priority));
    let first_b_only = switching.iter().find(|p| !a_set.contains(&p.priority) && b_set.contains(&p.priority));
    match (first_a_only, first_b_only) {
        (Some(ap), Some(bp)) if ap.priority > bp.priority =>
            println!("  [{}  wins lex] first divergence: {} satisfies priority {} (patient {}), {} satisfies priority {} (patient {})",
                label_a, label_a, ap.priority, ap.id, label_b, bp.priority, bp.id),
        (Some(ap), Some(bp)) =>
            println!("  [{} wins lex] first divergence: {} satisfies priority {} (patient {}), {} satisfies priority {} (patient {})",
                label_b, label_b, bp.priority, bp.id, label_a, ap.priority, ap.id),
        (Some(ap), None) =>
            println!("  [{} strictly dominates] first exclusive at priority {} (patient {})", label_a, ap.priority, ap.id),
        (None, Some(bp)) =>
            println!("  [{} strictly dominates] first exclusive at priority {} (patient {})", label_b, bp.priority, bp.id),
        (None, None) =>
            println!("  [Equal] both algorithms satisfy exactly the same patients"),
    }
}

struct RunResult {
    dataset: String,
    num_patients: usize,
    num_doctors: usize,
    algorithm: String,
    patients_satisfied: usize,
    patients_wanting_switch: usize,
    time_ms: u128,
}

fn main() {
    let data_files = vec![
        "data/test_100_patient_15_doctors_0_unassigned.txt",
        "data/test_1000_patient_30_doctors_0_unassigned.txt",
        "data/test_10000_patient_150_doctors_0_unassigned.txt",
        // "data/test_100000_patient_1500_doctors_0_unassigned.txt",
    ];

    let mut results: Vec<RunResult> = Vec::new();

    for file in &data_files {
        println!("\n=== Dataset: {} ===", file);
        let (patients, doctors) = match parse_data_file(file) {
            Ok(d) => d,
            Err(e) => { eprintln!("Failed to parse {}: {}", file, e); continue; }
        };

        let num_patients = patients.iter().filter(|p| !p.is_dummy).count();
        let num_doctors = doctors.iter().filter(|d| !d.is_dummy).count();
        let patients_wanting_switch = patients.iter().filter(|p| !p.is_dummy && p.wants_to_switch).count();

        // Extract dataset label (e.g. "100p_15d")
        let dataset = format!("{}p_{}d", num_patients, num_doctors);

        // --- CyclePacker (exact, cardinality) ---
        print!("  CyclePacker... ");
        let _ = std::io::stdout().flush();
        let packer_state = AssignmentState::new(patients.clone(), doctors.clone());
        let t0 = std::time::Instant::now();
        let mut packer = CyclePacker::new(&packer_state);
        packer.pack_cycles();
        let exact_ms = t0.elapsed().as_millis();
        let exact_satisfied = packer.count_satisfied_real_patients(&patients);
        println!("{} satisfied in {}ms", exact_satisfied, exact_ms);
        results.push(RunResult {
            dataset: dataset.clone(),
            num_patients,
            num_doctors,
            algorithm: "CyclePacker".to_string(),
            patients_satisfied: exact_satisfied,
            patients_wanting_switch,
            time_ms: exact_ms,
        });

        // --- CyclePacker (priority-weighted) ---
        print!("  CyclePacker (priority-weighted)... ");
        let _ = std::io::stdout().flush();
        let pw_state = AssignmentState::new(patients.clone(), doctors.clone());
        let t0 = std::time::Instant::now();
        let mut pw_packer = PwCyclePacker::new(&pw_state);
        pw_packer.pack_cycles();
        let pw_ms = t0.elapsed().as_millis();
        let pw_satisfied_patients = pw_packer.satisfied_patients(&pw_state.patients);
        let pw_count = pw_satisfied_patients.len();
        println!("{} satisfied in {}ms", pw_count, pw_ms);
        
        results.push(RunResult {
            dataset: dataset.clone(),
            num_patients,
            num_doctors,
            algorithm: "CyclePacker_PriorityWeighted".to_string(),
            patients_satisfied: pw_count,
            patients_wanting_switch,
            time_ms: pw_ms,
        });

        // --- Greedy DFS ---
        print!("  Greedy DFS... ");
        let _ = std::io::stdout().flush();
        let mut greedy_state = AssignmentState::new(patients.clone(), doctors.clone());
        let t1 = std::time::Instant::now();
        let greedy_result = greedy_dfs(&mut greedy_state, PriorityStrategy::StrictPriority);
        let greedy_ms = t1.elapsed().as_millis();
        println!("{} satisfied in {}ms", greedy_result.patients_reassigned, greedy_ms);
        verify_result(&patients, &greedy_state);

        results.push(RunResult {
            dataset: dataset.clone(),
            num_patients,
            num_doctors,
            algorithm: "Greedy_DFS".to_string(),
            patients_satisfied: greedy_result.patients_reassigned,
            patients_wanting_switch,
            time_ms: greedy_ms,
        });
    }
    

    // Write CSV
    let csv_path = "benchmark_scaling.csv";
    match File::create(csv_path) {
        Ok(mut f) => {
            writeln!(f, "dataset,num_patients,num_doctors,algorithm,patients_satisfied,patients_wanting_switch,satisfaction_rate,time_ms").unwrap();
            for r in &results {
                let rate = if r.patients_wanting_switch > 0 {
                    r.patients_satisfied as f64 / r.patients_wanting_switch as f64
                } else {
                    0.0
                };
                writeln!(
                    f,
                    "{},{},{},{},{},{},{:.4},{}",
                    r.dataset, r.num_patients, r.num_doctors, r.algorithm,
                    r.patients_satisfied, r.patients_wanting_switch, rate, r.time_ms
                ).unwrap();
            }
            println!("\nResults saved to {}", csv_path);
        }
        Err(e) => eprintln!("Failed to write CSV: {}", e),
    }

    // Print summary table
    println!("\n{:<20} {:<30} {:>12} {:>12}", "Dataset", "Algorithm", "Satisfied", "Time (ms)");
    println!("{}", "-".repeat(78));
    for r in &results {
        println!("{:<20} {:<30} {:>12} {:>12}", r.dataset, r.algorithm, r.patients_satisfied, r.time_ms);
    }
}
