use ttc::benchmarking::{AlgorithmConfig, Benchmarker};
use ttc::{ttc_algorithm, restricted_ttc_algorithm, PriorityStrategy, TTCResultWithStats, TTCState};
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
        // "data/test_200_patient_15_doctors_3_districts_0.1_prob.txt".to_string(),
        // "data/test_1000_patient_100_doctors_10_districts_0.1_prob.txt".to_string(),
        
        // Medium test files
        // "data/test_100000_patient_2000_doctors_2_districts_0.2_prob.txt".to_string(),
        "data/test_150000_patient_2000_doctors_5_districts_0.3_prob.txt".to_string(),
        
        // Large test files with unassigned patients
        //"data/test_250000_patient_5000_doctors_10_districts_0.25_prob_5000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_25000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_50000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.001_prob_50000_unassigned.txt".to_string(),
        // "data/test_250000_patient_5000_doctors_10_districts_0.05_prob_50000_unassigned.txt".to_string(),
    ];

    const NUM_RUNS: usize = 1;

    // Configure algorithms to benchmark - try different priority strategies!
    let algorithms = vec![
        AlgorithmConfig::new("Strict Priority", strategy_strict_priority),
        AlgorithmConfig::new("Restricted TTC", restricted_ttc),
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
}
