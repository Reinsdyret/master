use ttc::excact::{CyclePacker, PwCyclePacker};
use ttc::{ttc_algorithm, verify_ttc_result, PriorityStrategy, TTCState, parse_data_file};
use std::fs::File;
use std::io::Write;
use std::collections::HashSet;

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
        "data/test_100000_patient_1500_doctors_0_unassigned.txt",
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
        // print!("  CyclePacker... ");
        // let _ = std::io::stdout().flush();
        // let t0 = std::time::Instant::now();
        // let mut packer = CyclePacker::new(&patients, &doctors);
        // packer.pack_cycles();
        // let exact_ms = t0.elapsed().as_millis();
        // let exact_satisfied = packer.count_satisfied_real_patients(&patients);
        // println!("{} satisfied in {}ms", exact_satisfied, exact_ms);
        // results.push(RunResult {
        //     dataset: dataset.clone(),
        //     num_patients,
        //     num_doctors,
        //     algorithm: "CyclePacker".to_string(),
        //     patients_satisfied: exact_satisfied,
        //     patients_wanting_switch,
        //     time_ms: exact_ms,
        // });

        // --- CyclePacker (priority-weighted) ---
        print!("  CyclePacker (priority-weighted)... ");
        let _ = std::io::stdout().flush();
        let t0 = std::time::Instant::now();
        let mut pw_packer = PwCyclePacker::new(&patients, &doctors);
        pw_packer.pack_cycles();
        let pw_ms = t0.elapsed().as_millis();
        let pw_satisfied_patients = pw_packer.satisfied_patients(&patients);
        let pw_satisfied_priorities: HashSet<usize> =
            pw_satisfied_patients.iter().map(|p| p.priority).collect();
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

        // --- TTC heuristic (StrictPriority) ---
        print!("  TTC (StrictPriority)... ");
        let _ = std::io::stdout().flush();
        let original_patients = patients.clone();
        let t1 = std::time::Instant::now();
        let mut ttc_state = TTCState::new(patients.clone(), doctors.clone());
        let ttc_result = ttc_algorithm(&mut ttc_state, PriorityStrategy::StrictPriority);
        let ttc_ms = t1.elapsed().as_millis();
        // ttc_result.solution stores the priority values of satisfied patients
        let ttc_satisfied_priorities: &HashSet<usize> = &ttc_result.solution;
        println!("{} satisfied in {}ms", ttc_result.patients_reassigned, ttc_ms);
        verify_ttc_result(&original_patients, &ttc_state);

        results.push(RunResult {
            dataset: dataset.clone(),
            num_patients,
            num_doctors,
            algorithm: "TTC_StrictPriority".to_string(),
            patients_satisfied: ttc_result.patients_reassigned,
            patients_wanting_switch,
            time_ms: ttc_ms,
        });

        // --- Patient-by-patient comparison (PW vs TTC, sorted by priority descending) ---

        // Get all switching real patients, sort by priority descending (highest = most important)
        
        let mut switching: Vec<_> = patients.iter()
            .filter(|p| !p.is_dummy && p.wants_to_switch)
            .collect();
        switching.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut pw_leads = 0usize;
        let mut ttc_leads = 0usize;
        let mut both = 0usize;
        let mut neither = 0usize;

        let cmp_path = format!("priority_comparison_{}.txt", dataset);
        let mut cmp_file = File::create(&cmp_path).expect("failed to create comparison file");
        writeln!(cmp_file, "Priority comparison: PW vs TTC ({})", dataset).unwrap();
        writeln!(cmp_file, "Sorted by priority descending (highest priority number = most important)").unwrap();
        writeln!(cmp_file, "{:>10}  {:>3}  {:>5}  {:>10}  {:>5}",
            "Patient ID", "PW", "TTC", "Priority", "Verdict").unwrap();
        writeln!(cmp_file, "{}", "-".repeat(42)).unwrap();

        for p in &switching {
            let in_pw  = pw_satisfied_priorities.contains(&p.priority);
            let in_ttc = ttc_satisfied_priorities.contains(&p.priority);

            let verdict = match (in_pw, in_ttc) {
                (true,  true)  => { both      += 1; "both" }
                (true,  false) => { pw_leads  += 1; "PW only" }
                (false, true)  => { ttc_leads += 1; "TTC only" }
                (false, false) => { neither   += 1; "neither" }
            };

            writeln!(cmp_file, "{:>10}  {:>3}  {:>5}  {:>10}  {:>5}",
                p.id,
                if in_pw  { "Y" } else { "N" },
                if in_ttc { "Y" } else { "N" },
                p.priority,
                verdict).unwrap();
        }

        writeln!(cmp_file, "{}", "-".repeat(42)).unwrap();
        writeln!(cmp_file, "Both satisfied : {}", both).unwrap();
        writeln!(cmp_file, "PW only        : {}", pw_leads).unwrap();
        writeln!(cmp_file, "TTC only       : {}", ttc_leads).unwrap();
        writeln!(cmp_file, "Neither        : {}", neither).unwrap();
        println!("  Per-patient comparison written to {}", cmp_path);

        // Console summary: find the highest-priority patient where the algorithms first diverge
        let first_pw_only  = switching.iter().find(|p| pw_satisfied_priorities.contains(&p.priority) && !ttc_satisfied_priorities.contains(&p.priority));
        let first_ttc_only = switching.iter().find(|p| !pw_satisfied_priorities.contains(&p.priority) && ttc_satisfied_priorities.contains(&p.priority));
        match (first_pw_only, first_ttc_only) {
            (Some(pw_p), Some(ttc_p)) if pw_p.priority > ttc_p.priority =>
                println!("  [OK] PW leads at priority {} (patient {}), TTC's first exclusive is priority {} (patient {}) — PW wins the lex comparison",
                    pw_p.priority, pw_p.id, ttc_p.priority, ttc_p.id),
            (Some(pw_p), Some(ttc_p)) =>
                println!("  [WARN] TTC leads at priority {} (patient {}) before PW leads at priority {} (patient {}) — TTC wins the lex comparison; priority weighting is NOT dominating",
                    ttc_p.priority, ttc_p.id, pw_p.priority, pw_p.id),
            (Some(pw_p), None) =>
                println!("  [OK] PW strictly dominates: first exclusive gain at priority {} (patient {}), TTC never satisfies a patient that PW misses",
                    pw_p.priority, pw_p.id),
            (None, Some(ttc_p)) =>
                println!("  [WARN] TTC dominates: TTC satisfies priority {} (patient {}) that PW misses, and PW never satisfies anyone TTC misses — priority weighting is WORSE than TTC",
                    ttc_p.priority, ttc_p.id),
            (None, None) =>
                println!("  [OK] Both algorithms satisfy exactly the same patients"),
        }
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
