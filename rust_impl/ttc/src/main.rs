use ttc::benchmarking::Benchmarker;

fn main() {
    let data_files = vec![
        // "data/test_133000_patient_4000_doctors_15_districts_0.01_prob.txt".to_string(),
        // "data/test_133000_patient_4000_doctors_25_districts_0.01_prob.txt".to_string(),
        // "data/test_133000_patient_4000_doctors_35_districts_0.01_prob.txt".to_string(),
        // "data/test_150000_patient_5000_doctors_50_districts_0.05_prob.txt".to_string(),
        // "data/test_200000_patient_10000_doctors_100_districts_0.1_prob.txt".to_string(),
        "data/test_500000_patient_25000_doctors_1000_districts_0.05_prob.txt".to_string(),
        "data/test_150000_patient_2000_doctors_8_districts_0.05_prob.txt".to_string(),
        "data/test_150000_patient_2000_doctors_16_districts_0.05_prob.txt".to_string(),
        "data/test_150000_patient_2000_doctors_32_districts_0.05_prob.txt".to_string(),
        "data/test_150000_patient_2000_doctors_64_districts_0.05_prob.txt".to_string(),
        "data/test_150000_patient_2000_doctors_128_districts_0.05_prob.txt".to_string(),
        "data/test_150000_patient_4000_doctors_128_districts_0.05_prob.txt".to_string(),
        "data/test_150000_patient_6000_doctors_128_districts_0.05_prob.txt".to_string(),
        // "data/test_100000_patient_2000_doctors_10_districts_chain.txt".to_string()
        // "data/test_1000_patient_100_doctors_10_districts_chain.txt".to_string(),
        // "data/test_1000_patient_100_doctors_10_districts_0.1_prob.txt".to_string()
    ];

    const NUM_RUNS: usize = 10;

    let mut benchmarker = Benchmarker::new(data_files, NUM_RUNS);

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
