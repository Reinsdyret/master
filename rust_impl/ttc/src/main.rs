use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use ttc::{
    TTCResultWithStats, TTCSCCSolver, TTCState, parse_data_file, ttc_algorithm_with_pruning,
    ttc_scc::TTCSCCSolverV2,
};

const NUM_ITERATIONS: usize = 1;

fn main() {
    match parse_data_file("data/test_150000_patient_2000_doctors_8_districts.txt") {
        Ok((patients, doctors)) => {
            println!(
                "Parsed {} patients and {} doctors",
                patients.len(),
                doctors.len()
            );

            println!(
                "\n=== BENCHMARK: Running {} iterations of each algorithm ===",
                NUM_ITERATIONS
            );

            let mut pruning_times = Vec::with_capacity(NUM_ITERATIONS);
            let mut scc_v1_times = Vec::with_capacity(NUM_ITERATIONS);
            let mut scc_v2_times = Vec::with_capacity(NUM_ITERATIONS);

            for i in 1..=NUM_ITERATIONS {
                println!("\n--- Iteration {}/{} ---", i, NUM_ITERATIONS);

                // Test pruning algorithm
                println!("🔄 Running DFS PRUNING algorithm...");
                let (result_pruning, time_pruning) =
                    run_pruning_experiment(patients.clone(), doctors.clone());
                pruning_times.push(time_pruning.as_secs_f64() * 1000.0);

                // Test SCC algorithm V1 (Tarjan with doctor nodes)
                println!("⚡ Running SCC V1 (with doctor nodes)...");
                let (result_scc_v1, time_scc_v1) =
                    run_scc_experiment(patients.clone(), doctors.clone());
                scc_v1_times.push(time_scc_v1.as_secs_f64() * 1000.0);

                // Test SCC algorithm V2 (Optimized subgraph)
                println!("🚀 Running SCC V2 (optimized subgraph)...");
                let (result_scc_v2, time_scc_v2) =
                    run_scc_tarjan_experiment(patients.clone(), doctors.clone());
                scc_v2_times.push(time_scc_v2.as_secs_f64() * 1000.0);

                // Verify results are consistent
                if i == 1 {
                    verify_results_three(&result_pruning, &result_scc_v1, &result_scc_v2);
                }
            }

            // Write results to file
            write_benchmark_results(&pruning_times, &scc_v1_times, &scc_v2_times)
                .expect("Failed to write benchmark results");

            // Print summary
            print_summary(HashMap::from([
                ("Pruning".to_owned(), &pruning_times),
                ("SCC V1".to_owned(), &scc_v1_times),
                ("SCC V2".to_owned(), &scc_v2_times),
            ]));
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn write_benchmark_results(pruning: &[f64], scc_v1: &[f64], scc_v2: &[f64]) -> std::io::Result<()> {
    let mut file = File::create("benchmark_results.txt")?;

    writeln!(file, "# Benchmark Results ({} iterations)", NUM_ITERATIONS)?;
    writeln!(file, "# Times in milliseconds")?;
    writeln!(file)?;

    writeln!(file, "dfs_pruning = {:?}", pruning)?;
    writeln!(file, "scc_v1 = {:?}", scc_v1)?;
    writeln!(file, "scc_v2_optimized = {:?}", scc_v2)?;

    println!("\n✅ Results written to benchmark_results.txt");
    Ok(())
}

fn verify_results_three(
    result_pruning: &TTCResultWithStats,
    result_scc_v1: &TTCResultWithStats,
    result_scc_v2: &TTCResultWithStats,
) {
    let all_match = result_pruning.cycles_found == result_scc_v1.cycles_found
        && result_pruning.cycles_found == result_scc_v2.cycles_found
        && result_pruning.patients_reassigned == result_scc_v1.patients_reassigned
        && result_pruning.patients_reassigned == result_scc_v2.patients_reassigned;

    if all_match {
        println!("✅ All algorithms produce identical results!");
    } else {
        println!("⚠️  WARNING: Results differ between algorithms!");
        println!(
            "  DFS Pruning:  {} cycles, {} patients",
            result_pruning.cycles_found, result_pruning.patients_reassigned
        );
        println!(
            "  SCC V1:       {} cycles, {} patients",
            result_scc_v1.cycles_found, result_scc_v1.patients_reassigned
        );
        println!(
            "  SCC V2:       {} cycles, {} patients",
            result_scc_v2.cycles_found, result_scc_v2.patients_reassigned
        );
    }
}

fn print_summary(stats: HashMap<String, &Vec<f64>>) {
    println!("\n{}", "=".repeat(60));
    println!("BENCHMARK SUMMARY ({} iterations)", NUM_ITERATIONS);
    println!("{}", "=".repeat(60));

    let calc_stats = |data: &[f64], name: &str| {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        println!("\n{}:", name);
        println!("  Mean: {:.2}ms", mean);
        println!("  Min:  {:.2}ms", min);
        println!("  Max:  {:.2}ms", max);
        println!("  Range: {:.2}ms", max - min);
        mean
    };

    println!("\n{}", "=".repeat(60));
    println!("RELATIVE PERFORMANCE:");
    println!("{}", "=".repeat(60));

    let mut means: HashMap<String, f64> = stats
        .iter()
        .map(|(key, value)| {
            let mean = calc_stats(value, key);
            (key.clone(), mean)
        })
        .collect();

    for (key, mean) in &means {
        for (key2, mean2) in &means {
            if key.eq(key2) {
                continue;
            }
            println!("{} vs {}: {:.2}x", key, key2, mean2 / mean)
        }
    }
}

fn run_pruning_experiment(
    patients: Vec<ttc::Patient>,
    doctors: Vec<ttc::Doctor>,
) -> (TTCResultWithStats, std::time::Duration) {
    let mut state = TTCState::new(patients, doctors);

    println!("📊 Initial state:");
    print_statistics(&state);

    let start_time = std::time::Instant::now();
    let result = ttc_algorithm_with_pruning(&mut state);
    let execution_time = start_time.elapsed();

    println!("\n📊 PRUNING Results:");
    print_algorithm_results_with_stats(&result, execution_time);
    print_statistics(&state);

    (result, execution_time)
}

fn run_scc_experiment(
    patients: Vec<ttc::Patient>,
    doctors: Vec<ttc::Doctor>,
) -> (TTCResultWithStats, std::time::Duration) {
    let mut state = TTCState::new(patients, doctors);

    println!("📊 Initial state:");
    print_statistics(&state);

    let start_time = std::time::Instant::now();
    let mut scc_solver = TTCSCCSolver::new();
    let result = scc_solver.solve(&mut state);
    let execution_time = start_time.elapsed();

    println!("\n📊 SCC V1 (Tarjan with doctor nodes) Results:");
    print_algorithm_results_with_stats(&result, execution_time);
    print_scc_stats(scc_solver.get_stats());
    print_statistics(&state);

    (result, execution_time)
}

fn run_scc_tarjan_experiment(
    patients: Vec<ttc::Patient>,
    doctors: Vec<ttc::Doctor>,
) -> (TTCResultWithStats, std::time::Duration) {
    let mut state = TTCState::new(patients, doctors);

    println!("📊 Initial state:");
    print_statistics(&state);

    let start_time = std::time::Instant::now();
    let mut scc_solver = TTCSCCSolverV2::new();
    let result = scc_solver.solve(&mut state);
    let execution_time = start_time.elapsed();

    println!("\n📊 SCC V2 (Tarjan patient-only) Results:");
    print_algorithm_results_with_stats(&result, execution_time);
    print_scc_stats(scc_solver.get_stats());
    print_statistics(&state);

    (result, execution_time)
}

fn print_scc_stats(stats: &ttc::SCCStats) {
    println!("🔍 SCC Algorithm Stats:");
    println!("  SCCs found: {}", stats.sccs_found);
    println!("  Largest SCC size: {}", stats.largest_scc_size);
    println!("  Cycles processed: {}", stats.cycles_processed);
    println!("  Graph nodes: {}", stats.total_graph_nodes);
    println!("  Graph edges: {}", stats.total_graph_edges);

    // Timing breakdown
    let total_time = stats.time_graph_building
        + stats.time_scc_finding
        + stats.time_cycle_finding
        + stats.time_cycle_execution;
    if total_time.as_nanos() > 0 {
        println!("\n⏱️  Timing Breakdown:");
        println!(
            "  Graph building:  {:.2}ms ({:.1}%)",
            stats.time_graph_building.as_secs_f64() * 1000.0,
            stats.time_graph_building.as_secs_f64() / total_time.as_secs_f64() * 100.0
        );
        println!(
            "  SCC finding:     {:.2}ms ({:.1}%)",
            stats.time_scc_finding.as_secs_f64() * 1000.0,
            stats.time_scc_finding.as_secs_f64() / total_time.as_secs_f64() * 100.0
        );
        println!(
            "  Cycle finding:   {:.2}ms ({:.1}%)",
            stats.time_cycle_finding.as_secs_f64() * 1000.0,
            stats.time_cycle_finding.as_secs_f64() / total_time.as_secs_f64() * 100.0
        );
        println!(
            "  Cycle execution: {:.2}ms ({:.1}%)",
            stats.time_cycle_execution.as_secs_f64() * 1000.0,
            stats.time_cycle_execution.as_secs_f64() / total_time.as_secs_f64() * 100.0
        );
        println!(
            "  Total measured:  {:.2}ms",
            total_time.as_secs_f64() * 1000.0
        );
    }
}

fn print_comparison_three(
    result_pruning: &TTCResultWithStats,
    time_pruning: std::time::Duration,
    result_scc_kosaraju: &TTCResultWithStats,
    time_scc_kosaraju: std::time::Duration,
    result_scc_tarjan: &TTCResultWithStats,
    time_scc_tarjan: std::time::Duration,
) {
    println!("\n⚡ PERFORMANCE COMPARISON:");
    println!(
        "  Pruning time:       {:.3}ms",
        time_pruning.as_secs_f64() * 1000.0
    );
    println!(
        "  SCC (Kosaraju) time: {:.3}ms",
        time_scc_kosaraju.as_secs_f64() * 1000.0
    );
    println!(
        "  SCC (Tarjan) time:   {:.3}ms",
        time_scc_tarjan.as_secs_f64() * 1000.0
    );

    if time_pruning.as_nanos() > 0 {
        let speedup_kosaraju = time_pruning.as_secs_f64() / time_scc_kosaraju.as_secs_f64();
        let speedup_tarjan = time_pruning.as_secs_f64() / time_scc_tarjan.as_secs_f64();
        println!("  Kosaraju Speedup: {:.2}x", speedup_kosaraju);
        println!("  Tarjan Speedup:   {:.2}x", speedup_tarjan);
    }

    println!("\n📈 RESULT COMPARISON:");
    println!("  Cycles found:");
    println!("    Pruning:       {}", result_pruning.cycles_found);
    println!("    SCC (Kosaraju): {}", result_scc_kosaraju.cycles_found);
    println!("    SCC (Tarjan):   {}", result_scc_tarjan.cycles_found);

    println!("  Patients reassigned:");
    println!("    Pruning:       {}", result_pruning.patients_reassigned);
    println!(
        "    SCC (Kosaraju): {}",
        result_scc_kosaraju.patients_reassigned
    );
    println!(
        "    SCC (Tarjan):   {}",
        result_scc_tarjan.patients_reassigned
    );

    println!("  Patients pruned:");
    println!("    Pruning:       {}", result_pruning.patients_pruned);
    println!(
        "    SCC (Kosaraju): {}",
        result_scc_kosaraju.patients_pruned
    );
    println!("    SCC (Tarjan):   {}", result_scc_tarjan.patients_pruned);

    // Verify results match
    let all_match = result_pruning.cycles_found == result_scc_kosaraju.cycles_found
        && result_pruning.cycles_found == result_scc_tarjan.cycles_found
        && result_pruning.patients_reassigned == result_scc_kosaraju.patients_reassigned
        && result_pruning.patients_reassigned == result_scc_tarjan.patients_reassigned;

    if all_match {
        println!("\n  ✅ All results match - algorithms are equivalent!");
    } else {
        println!("\n  ⚠️  Results differ - need investigation!");
    }
}

fn print_algorithm_results_with_stats(
    result: &TTCResultWithStats,
    execution_time: std::time::Duration,
) {
    println!("🎯 Algorithm Results (with pruning):");
    println!(
        "  ⏱️  Execution time: {:.3}ms",
        execution_time.as_secs_f64() * 1000.0
    );

    if result.cycles_found == 0 {
        println!("  No cycles found - no reassignments made");
    } else {
        println!("  Cycles found: {}", result.cycles_found);
        println!("  Patients reassigned: {}", result.patients_reassigned);
        println!(
            "  Average cycle size: {:.1}",
            result.patients_reassigned as f64 / result.cycles_found as f64
        );

        // Performance metrics
        if execution_time.as_nanos() > 0 {
            let patients_per_ms =
                result.patients_reassigned as f64 / (execution_time.as_secs_f64() * 1000.0);
            println!("  Performance: {:.1} patients/ms", patients_per_ms);
        }
    }

    println!("  🔧 Patients pruned: {}", result.patients_pruned);
    if result.patients_pruned > 0 {
        println!(
            "  🔧 Pruning efficiency: {:.1}% of search space eliminated",
            result.patients_pruned as f64
                / (result.patients_reassigned + result.patients_pruned) as f64
                * 100.0
        );
    }
}

fn print_statistics(state: &TTCState) {
    let total_patients = state.patients.len();
    let truly_happy = state
        .patients
        .iter()
        .filter(|p| !p.wants_to_switch && !p.is_stuck)
        .count();
    let stuck_patients = state.patients.iter().filter(|p| p.is_stuck).count();
    let unhappy_patients = state
        .patients
        .iter()
        .filter(|p| p.wants_to_switch && !p.is_stuck)
        .count();

    let total_switching_requests = state
        .doctors
        .iter()
        .map(|d| d.switching_patients.len())
        .sum::<usize>();

    println!("📊 Statistics:");
    println!("  Total patients: {}", total_patients);
    println!(
        "  🎉 Truly happy: {} ({:.1}%)",
        truly_happy,
        (truly_happy as f64 / total_patients as f64) * 100.0
    );
    println!(
        "  😞 Still unhappy: {} ({:.1}%)",
        unhappy_patients,
        (unhappy_patients as f64 / total_patients as f64) * 100.0
    );
    println!(
        "  🚫 Stuck (cannot be satisfied): {} ({:.1}%)",
        stuck_patients,
        (stuck_patients as f64 / total_patients as f64) * 100.0
    );
    println!("  Total switching requests: {}", total_switching_requests);

    // Doctor utilization
    let doctors_with_switching = state
        .doctors
        .iter()
        .filter(|d| !d.switching_patients.is_empty())
        .count();
    println!(
        "  Doctors with switching patients: {} / {}",
        doctors_with_switching,
        state.doctors.len()
    );

    // Priority distribution of still-unhappy patients
    if unhappy_patients > 0 {
        let switching_priorities: Vec<usize> = state
            .patients
            .iter()
            .filter(|p| p.wants_to_switch && !p.is_stuck)
            .map(|p| p.priority)
            .collect();
        let min_priority = switching_priorities.iter().min().unwrap_or(&0);
        let max_priority = switching_priorities.iter().max().unwrap_or(&0);
        let avg_priority =
            switching_priorities.iter().sum::<usize>() as f64 / switching_priorities.len() as f64;

        println!(
            "  Priority range of still unhappy: {} - {} (avg: {:.1})",
            min_priority, max_priority, avg_priority
        );
    }
}
