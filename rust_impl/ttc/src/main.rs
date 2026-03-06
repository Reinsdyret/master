use ttc::excact::CyclePacker;
use ttc::{ttc_algorithm, verify_ttc_result, PriorityStrategy, TTCState, parse_data_file};
use std::fs::File;
use std::io::Write;

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

        // --- CyclePacker (exact) ---
        print!("  CyclePacker... ");
        let _ = std::io::stdout().flush();
        let t0 = std::time::Instant::now();
        let mut packer = CyclePacker::new(&patients, &doctors);
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

        // --- TTC heuristic (StrictPriority) ---
        print!("  TTC (StrictPriority)... ");
        let _ = std::io::stdout().flush();
        let original_patients = patients.clone();
        let t1 = std::time::Instant::now();
        let mut ttc_state = TTCState::new(patients.clone(), doctors.clone());
        let ttc_result = ttc_algorithm(&mut ttc_state, PriorityStrategy::StrictPriority);
        let ttc_ms = t1.elapsed().as_millis();
        println!("{} satisfied in {}ms", ttc_result.patients_reassigned, ttc_ms);
        verify_ttc_result(&original_patients, &ttc_state);

        results.push(RunResult {
            dataset,
            num_patients,
            num_doctors,
            algorithm: "TTC_StrictPriority".to_string(),
            patients_satisfied: ttc_result.patients_reassigned,
            patients_wanting_switch,
            time_ms: ttc_ms,
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
    println!("\n{:<20} {:<22} {:>12} {:>12}", "Dataset", "Algorithm", "Satisfied", "Time (ms)");
    println!("{}", "-".repeat(70));
    for r in &results {
        println!("{:<20} {:<22} {:>12} {:>12}", r.dataset, r.algorithm, r.patients_satisfied, r.time_ms);
    }
}
