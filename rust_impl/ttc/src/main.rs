use ttc::benchmarking::{AlgorithmConfig, Benchmarker};
use ttc::excact::CyclePacker;
use ttc::{ttc_algorithm, restricted_ttc_algorithm, verify_ttc_result, PriorityStrategy, TTCResultWithStats, TTCState, parse_data_file};
use std::fs::File;
use std::io::Write;
// use ttc::ttc_scc::scc_algorithm;

// Wrapper functions for each strategy (required for benchmarker's function pointer interface)
fn strategy_strict_priority(state: &mut TTCState) -> TTCResultWithStats {
    ttc_algorithm(state, PriorityStrategy::StrictPriority)
}

fn strategy_unassigned_first(state: &mut TTCState) -> TTCResultWithStats {
    ttc_algorithm(state, PriorityStrategy::UnassignedFirst)
}

fn strategy_random(state: &mut TTCState) -> TTCResultWithStats {
    ttc_algorithm(state, PriorityStrategy::Random)
}

fn strategy_high_demand_first(state: &mut TTCState) -> TTCResultWithStats {
    ttc_algorithm(state, PriorityStrategy::HighDemandFirst)
}

fn strategy_low_demand_first(state: &mut TTCState) -> TTCResultWithStats {
    ttc_algorithm(state, PriorityStrategy::LowDemandFirst)
}

fn restricted_ttc(state: &mut TTCState) ->  TTCResultWithStats {
    restricted_ttc_algorithm(state)
}

fn main() {
    let data_files = vec![
        // Small test files
        // "data/test_4_patient_3_doctors_mini.txt".to_string()
        // "data/test_200_patient_15_doctors_3_districts_0.1_prob.txt".to_string(),
        "data/test_1000_patient_30_doctors_0_unassigned.txt".to_string(),
        // "data/test_1000_patient_100_doctors_10_districts_0.1_prob.txt".to_string(),
        
        // Medium test files
        // "data/test_100000_patient_2000_doctors_2_districts_0.2_prob.txt".to_string(),
        // "data/test_150000_patient_2000_doctors_5_districts_0.3_prob.txt".to_string(),
        
        // Large test files with unassigned patients
        //"data/test_250000_patient_5000_doctors_10_districts_0.25_prob_5000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_25000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_50000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.001_prob_50000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.05_prob_50000_unassigned.txt".to_string(),
    ];

    
    let test_file = "data/test_100000_patient_1500_doctors_0_unassigned.txt";
    let (patients, doctors) = parse_data_file(test_file).unwrap();

    // ---- CyclePacker (exact) ----
    println!("\n=== CyclePacker (exact) ===");
    let t0 = std::time::Instant::now();
    let mut packer = CyclePacker::new(&patients, &doctors);
    packer.pack_cycles();
    let exact_ms = t0.elapsed().as_millis();
    packer.verify_solution(&patients, &doctors);
    packer.verify_patient_edges(&patients);
    let exact_satisfied = packer.count_satisfied_real_patients(&patients);
    println!("CyclePacker satisfied real patients: {} (took {}ms)", exact_satisfied, exact_ms);

    // ---- TTC heuristic ----
    println!("\n=== TTC (StrictPriority) ===");
    let original_patients = patients.clone();
    let t1 = std::time::Instant::now();
    let mut ttc_state = TTCState::new(patients.clone(), doctors.clone());
    let ttc_result = ttc_algorithm(&mut ttc_state, PriorityStrategy::StrictPriority);
    let ttc_ms = t1.elapsed().as_millis();
    println!("TTC satisfied real patients: {} (took {}ms)", ttc_result.patients_reassigned, ttc_ms);
    verify_ttc_result(&original_patients, &ttc_state);

    println!("\n=== Summary ===");
    println!("CyclePacker: {} patients satisfied in {}ms", exact_satisfied, exact_ms);
    println!("TTC heuristic: {} patients satisfied in {}ms", ttc_result.patients_reassigned, ttc_ms);

    /*
    const NUM_RUNS: usize = 1;

    // Configure algorithms to benchmark - try different priority strategies!
    let algorithms = vec![
        AlgorithmConfig::new("Strict Priority", strategy_strict_priority),
        // AlgorithmConfig::new("Restricted TTC", restricted_ttc),
        // AlgorithmConfig::new("Unassigned First", strategy_unassigned_first),
        // AlgorithmConfig::new("Random Order", strategy_random),
        // AlgorithmConfig::new("High Demand First", strategy_high_demand_first),  // Slow - 2x runtime
        // AlgorithmConfig::new("Low Demand First", strategy_low_demand_first),
        // AlgorithmConfig::new("SCC", scc_algorithm),
    ];

    let mut benchmarker = Benchmarker::new(data_files, NUM_RUNS, algorithms);

    match benchmarker.run_benchmarks() {
        Ok(_) => {
            println!("\nAll benchmarks completed successfully!");

            benchmarker.print_summary();

            match benchmarker.save_results("benchmark_results_comprehensive.txt") {
                Ok(_) => {
                    println!("\nResults saved to benchmark_results_comprehensive.txt");
                }
                Err(e) => eprintln!("Failed to save results: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Benchmark failed: {}", e);
        }
    }

    // Log solutions to files for comparison
    /*let test_file = "data/test_150000_patient_2000_doctors_5_districts_0.3_prob.txt";
    if let Ok((patients, doctors)) = parse_data_file(test_file) {
        // Run Strict Priority
        let mut state1 = TTCState::new(patients.clone(), doctors.clone());
        let result1 = ttc_algorithm(&mut state1, PriorityStrategy::StrictPriority);
        log_solution_to_file(&result1.solution, "solution_strict_priority.txt");

        // Run Restricted TTC
        let mut state2 = TTCState::new(patients, doctors);
        let result2 = restricted_ttc_algorithm(&mut state2);
        log_solution_to_file(&result2.solution, "solution_restricted_ttc.txt");

        println!("\nSolutions logged to solution_strict_priority.txt and solution_restricted_ttc.txt");
    }*/
    */
}

/// Logs patient priorities to a file, sorted in decreasing order
fn log_solution_to_file(solution: &std::collections::HashSet<usize>, filename: &str) {
    let mut priorities: Vec<usize> = solution.iter().copied().collect();
    priorities.sort();
    priorities.reverse(); // Decreasing order (highest priority first)

    if let Ok(mut file) = File::create(filename) {
        let content = priorities
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let _ = file.write_all(content.as_bytes());
    }
}
