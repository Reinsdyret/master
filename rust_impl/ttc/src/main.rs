use ttc::benchmarking::Benchmarker;

fn main() {
    // Benchmark configuration:
    // 1. Scaling analysis: Different sizes
    // 2. District analysis: Same size (150k patients, 2k doctors) with varying districts

    let data_files = vec![
        // === Scaling Analysis: Variable sizes ===
        /*
        "data/test_100_patient_10_doctors.txt".to_string(),
        "data/test_1000_patient_100_doctors_3_districts.txt".to_string(),
        "data/test_10000_patient_1000_doctors_5_districts.txt".to_string(),
        "data/test_50000_patient_1000_doctors_20_districts.txt".to_string(),
        "data/test_100000_patient_4000_doctors_10_districts.txt".to_string(),
        // === District Impact Analysis: 150k patients, 2k doctors ===
        "data/test_150000_patient_2000_doctors_1_districts.txt".to_string(),
        "data/test_150000_patient_2000_doctors_2_districts.txt".to_string(),
        "data/test_150000_patient_2000_doctors_4_districts.txt".to_string(),
        "data/test_150000_patient_2000_doctors_8_districts.txt".to_string(),
        "data/test_150000_patient_2000_doctors_16_districts.txt".to_string(),
        // === District Impact Analysis: 133k patients, 4k doctors ===
        "data/test_133000_patient_4000_doctors.txt".to_string(),
        "data/test_133000_patient_4000_doctors_15_districts.txt".to_string(),
        "data/test_133000_patient_4000_doctors_25_districts.txt".to_string(),
        "data/test_133000_patient_4000_doctors_28_districts.txt".to_string(),
        "data/test_133000_patient_4000_doctors_30_districts.txt".to_string(),
         */
        // "data/test_300000_patient_5000_doctors_1000_districts_0.0_prob.txt".to_string(),    
        // "data/test_100000_patient_2000_doctors_10_districts_chain.txt".to_string()
        "data/test_1000_patient_100_doctors_10_districts_chain.txt".to_string(),
        "data/test_1000_patient_100_doctors_10_districts_0.1_prob.txt".to_string()
    ];

    const NUM_RUNS: usize = 10;

    println!("\n🎯 Benchmark Configuration:");
    println!("   Files to benchmark: {}", data_files.len());
    println!("   Runs per file: {}", NUM_RUNS);
    println!(
        "   Total algorithm executions: {}",
        data_files.len() * NUM_RUNS * 3
    );
    println!("\n📊 Analysis Types:");
    println!("   1. Scaling Analysis: How algorithms perform with different problem sizes");
    println!("   2. District Impact: How district count affects performance (2 dataset groups)");

    let mut benchmarker = Benchmarker::new(data_files, NUM_RUNS);

    match benchmarker.run_benchmarks() {
        Ok(_) => {
            println!("\n✅ All benchmarks completed successfully!");

            benchmarker.print_summary();

            match benchmarker.save_results("benchmark_results_comprehensive.txt") {
                Ok(_) => {
                    println!("\n💾 Results saved to benchmark_results_comprehensive.txt");
                    println!("\n📈 Next steps:");
                    println!("   Run: python3 plot_benchmarks.py");
                    println!("   This will generate plots showing:");
                    println!("     - Overall performance comparison");
                    println!("     - Scaling behavior");
                    println!("     - District impact on performance");
                    println!("     - Timing breakdowns for SCC algorithms");
                }
                Err(e) => eprintln!("❌ Failed to save results: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Benchmark failed: {}", e);
        }
    }
}
