use chrono::Local;
use std::fs::File;
use std::io::{BufWriter, Write};
use ttc::simulation::{NewRequestMode, SimulationConfig, SimulationResult, run_exact_cardinality, run_exact_priority, run_simulation};
use ttc::{AssignmentState, ResultWithStats, ttc_algorithm_strict_priority};
use ttc::huitfeldt::huitfeldt_ttc;

fn main() {
    let algorithms: Vec<(&str, fn(&mut AssignmentState) -> ResultWithStats)> = vec![
        ("Greedy DFS",        ttc_algorithm_strict_priority),
        ("Huitfeldt TTC",     huitfeldt_ttc),
        ("Exact Cardinality", run_exact_cardinality),
        ("Exact Priority",    run_exact_priority),
    ];

    let mut results: Vec<SimulationResult> = Vec::new();

    for (name, alg) in &algorithms {
        let config = SimulationConfig {
            num_patients: 5_631_845,
            num_doctors: 5767,
            waitlist_fraction: 0.05,
            num_days: 365 * 3,
            new_requests_per_day: NewRequestMode::Fixed(13000),
            min_new_requests_fraction: 0.001,
            algorithm: *alg,
            algorithm_name: name.to_string(),
            seed: 42,
        };

        let result = run_simulation(config);
        result.print_table();
        results.push(result);
    }

    let csv_path = save_csv(&results);
    println!("\nData saved to: {}", csv_path);
}

fn save_csv(results: &[SimulationResult]) -> String {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("simulation_{}.csv", timestamp);

    let file = File::create(&filename).expect("failed to create CSV");
    let mut w = BufWriter::new(file);

    writeln!(
        w,
        "algorithm,day,waitlist_before,patients_resolved,new_requests,waitlist_after,satisfaction_rate,cycles_found,avg_cycle_length,max_cycle_length,avg_wait_days,max_wait_days"
    )
    .unwrap();

    for r in results {
        for s in &r.day_stats {
            writeln!(
                w,
                "{},{},{},{},{},{},{:.6},{},{:.4},{},{:.4},{}",
                r.algorithm_name,
                s.day + 1,
                s.waitlist_size_before,
                s.patients_resolved,
                s.new_requests_added,
                s.waitlist_size_after,
                s.satisfaction_rate,
                s.cycles_found,
                s.avg_cycle_length,
                s.max_cycle_length,
                s.avg_wait_days,
                s.max_wait_days,
            )
            .unwrap();
        }
    }

    filename
}
