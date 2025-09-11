use ttc::{parse_data_file, TTCState, ttc_algorithm_with_pruning, TTCResult, TTCResultWithStats, TTCSCCSolver};

fn main() {
    match parse_data_file("data/test_1000000_patient_1000000_doctors.txt") {
        Ok((patients, doctors)) => {
            println!("Parsed {} patients and {} doctors", patients.len(), doctors.len());
            
            println!("\n=== BENCHMARK: Pruning vs SCC Algorithms ===");
            
            // Test pruning algorithm
            println!("\n🔄 Running PRUNING algorithm...");
            let (result_pruning, time_pruning) = run_pruning_experiment(patients.clone(), doctors.clone());
            
            // Test SCC algorithm  
            println!("\n🚀 Running SCC algorithm...");
            let (result_scc, time_scc) = run_scc_experiment(patients, doctors);
            
            // Compare results
            print_comparison(&result_pruning, time_pruning, &result_scc, time_scc);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn run_pruning_experiment(patients: Vec<ttc::Patient>, doctors: Vec<ttc::Doctor>) -> (TTCResultWithStats, std::time::Duration) {
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

fn run_scc_experiment(patients: Vec<ttc::Patient>, doctors: Vec<ttc::Doctor>) -> (TTCResultWithStats, std::time::Duration) {
    let mut state = TTCState::new(patients, doctors);
    
    println!("📊 Initial state:");
    print_statistics(&state);
    
    let start_time = std::time::Instant::now();
    let mut scc_solver = TTCSCCSolver::new();
    let result = scc_solver.solve(&mut state);
    let execution_time = start_time.elapsed();
    
    println!("\n📊 SCC Results:");
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
}

fn print_comparison(
    result_pruning: &TTCResultWithStats, 
    time_pruning: std::time::Duration,
    result_scc: &TTCResultWithStats, 
    time_scc: std::time::Duration
) {
    println!("\n⚡ PERFORMANCE COMPARISON:");
    println!("  Pruning time: {:.3}ms", time_pruning.as_secs_f64() * 1000.0);
    println!("  SCC time: {:.3}ms", time_scc.as_secs_f64() * 1000.0);
    
    if time_pruning.as_nanos() > 0 {
        let speedup = time_pruning.as_secs_f64() / time_scc.as_secs_f64();
        println!("  SCC Speedup: {:.2}x", speedup);
    }
    
    println!("\n📈 RESULT COMPARISON:");
    println!("  Cycles found: {} (Pruning) vs {} (SCC)", result_pruning.cycles_found, result_scc.cycles_found);
    println!("  Patients reassigned: {} (Pruning) vs {} (SCC)", result_pruning.patients_reassigned, result_scc.patients_reassigned);
    println!("  Patients pruned: {} (Pruning) vs {} (SCC)", result_pruning.patients_pruned, result_scc.patients_pruned);
    
    // Verify results match
    let results_match = result_pruning.cycles_found == result_scc.cycles_found &&
                       result_pruning.patients_reassigned == result_scc.patients_reassigned;
    
    if results_match {
        println!("  ✅ Results match - algorithms are equivalent!");
    } else {
        println!("  ⚠️  Results differ - need investigation!");
    }
}


fn print_algorithm_results_with_stats(result: &TTCResultWithStats, execution_time: std::time::Duration) {
    println!("🎯 Algorithm Results (with pruning):");
    println!("  ⏱️  Execution time: {:.3}ms", execution_time.as_secs_f64() * 1000.0);
    
    if result.cycles_found == 0 {
        println!("  No cycles found - no reassignments made");
    } else {
        println!("  Cycles found: {}", result.cycles_found);
        println!("  Patients reassigned: {}", result.patients_reassigned);
        println!("  Average cycle size: {:.1}", 
            result.patients_reassigned as f64 / result.cycles_found as f64);
        
        // Performance metrics
        if execution_time.as_nanos() > 0 {
            let patients_per_ms = result.patients_reassigned as f64 / (execution_time.as_secs_f64() * 1000.0);
            println!("  Performance: {:.1} patients/ms", patients_per_ms);
        }
    }
    
    println!("  🔧 Patients pruned: {}", result.patients_pruned);
    if result.patients_pruned > 0 {
        println!("  🔧 Pruning efficiency: {:.1}% of search space eliminated", 
            result.patients_pruned as f64 / (result.patients_reassigned + result.patients_pruned) as f64 * 100.0);
    }
}

fn print_statistics(state: &TTCState) {
    let total_patients = state.patients.len();
    let truly_happy = state.patients.iter()
        .filter(|p| !p.wants_to_switch && !p.is_stuck)
        .count();
    let stuck_patients = state.patients.iter()
        .filter(|p| p.is_stuck)
        .count();
    let unhappy_patients = state.patients.iter()
        .filter(|p| p.wants_to_switch && !p.is_stuck)
        .count();
    
    let total_switching_requests = state.doctors.iter()
        .map(|d| d.switching_patients.len())
        .sum::<usize>();
    
    println!("📊 Statistics:");
    println!("  Total patients: {}", total_patients);
    println!("  🎉 Truly happy: {} ({:.1}%)", 
        truly_happy, 
        (truly_happy as f64 / total_patients as f64) * 100.0
    );
    println!("  😞 Still unhappy: {} ({:.1}%)", 
        unhappy_patients,
        (unhappy_patients as f64 / total_patients as f64) * 100.0
    );
    println!("  🚫 Stuck (cannot be satisfied): {} ({:.1}%)", 
        stuck_patients,
        (stuck_patients as f64 / total_patients as f64) * 100.0
    );
    println!("  Total switching requests: {}", total_switching_requests);
    
    // Doctor utilization
    let doctors_with_switching = state.doctors.iter()
        .filter(|d| !d.switching_patients.is_empty())
        .count();
    println!("  Doctors with switching patients: {} / {}", 
        doctors_with_switching, state.doctors.len());
    
    // Priority distribution of still-unhappy patients
    if unhappy_patients > 0 {
        let switching_priorities: Vec<usize> = state.patients.iter()
            .filter(|p| p.wants_to_switch && !p.is_stuck)
            .map(|p| p.priority)
            .collect();
        let min_priority = switching_priorities.iter().min().unwrap_or(&0);
        let max_priority = switching_priorities.iter().max().unwrap_or(&0);
        let avg_priority = switching_priorities.iter().sum::<usize>() as f64 / switching_priorities.len() as f64;
        
        println!("  Priority range of still unhappy: {} - {} (avg: {:.1})", 
            min_priority, max_priority, avg_priority);
    }
}

