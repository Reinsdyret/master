use ttc::benchmarking::{AlgorithmConfig, Benchmarker};
use ttc::{ttc_algorithm_with_pruning, ttc_algorithm_without_prioritization};
use ttc::ttc_scc::scc_algorithm;

fn main() {
    let data_files = vec![
        // "data/test_133000_patient_4000_doctors_15_districts_0.01_prob.txt".to_string(),
        // "data/test_133000_patient_4000_doctors_25_districts_0.01_prob.txt".to_string(),
        // "data/test_133000_patient_4000_doctors_35_districts_0.01_prob.txt".to_string(),
        // "data/test_150000_patient_5000_doctors_50_districts_0.05_prob.txt".to_string(),
        // "data/test_200000_patient_10000_doctors_100_districts_0.1_prob.txt".to_string(),
        // "data/test_1500_patient_2_doctors_2_districts_0.1_prob.txt".to_string()
        // "data/test_200_patient_15_doctors_3_districts_0.1_prob.txt".to_string()
        // "data/test_150_patient_10_doctors_2_districts_0.0_prob.txt".to_string()
        // "data/test_1000_patient_20_doctors_1_districts_0.0_prob.txt".to_string(),
        // "data/test_100000_patient_2000_doctors_2_districts_0.2_prob.txt".to_string(),
        "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_5000_unassigned.txt".to_string(),
        "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_25000_unassigned.txt".to_string(),
        "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_50000_unassigned.txt".to_string(),
        // "data/test_1000_patient_100_doctors_10_districts_0.1_prob.txt".to_string()
        // "data/test_100000_patient_2000_doctors_3_districts_0.15_prob.txt".to_string()
        // "data/test_500000_patient_25000_doctors_1000_districts_0.05_prob.txt".to_string(),
        // "data/test_150000_patient_2000_doctors_8_districts_0.05_prob.txt".to_string(),
        // "data/test_150000_patient_2000_doctors_16_districts_0.05_prob.txt".to_string(),
        // "data/test_150000_patient_2000_doctors_5_districts_0.3_prob.txt".to_string(), // 0 Patients no doctor
        // "data/test_150000_patient_2000_doctors_32_districts_0.05_prob.txt".to_string(),
        // "data/test_150000_patient_2000_doctors_64_districts_0.05_prob.txt".to_string(),
        // "data/test_150000_patient_2000_doctors_128_districts_0.05_prob.txt".to_string(),
        // "data/test_150000_patient_4000_doctors_128_districts_0.05_prob.txt".to_string(),
        // "data/test_150000_patient_6000_doctors_128_districts_0.05_prob.txt".to_string(),
        // "data/test_100000_patient_2000_doctors_10_districts_chain.txt".to_string()
        // "data/test_1000_patient_100_doctors_10_districts_chain.txt".to_string(),
        // "data/test_1000_patient_100_doctors_10_districts_0.1_prob.txt".to_string()
    ];

    const NUM_RUNS: usize = 1;

    // Configure algorithms to benchmark
    let algorithms = vec![
        AlgorithmConfig::new("DFS Pruning", ttc_algorithm_with_pruning),
        AlgorithmConfig::new("DFS (No Prioritization)", ttc_algorithm_without_prioritization),
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
