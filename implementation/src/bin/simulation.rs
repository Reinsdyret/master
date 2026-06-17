use chrono::Local;
use std::fs::File;
use std::io::{BufWriter, Write};
use implementation::simulation::{NewRequestMode, SimulationConfig, SimulationResult, run_exact_cardinality, run_exact_priority, run_simulation, run_util_exp_1_01, run_util_exp_1_1, run_util_exp_1_05, run_util_exp_1_5, run_util_exp_1_9, run_util_linear};
use implementation::{AssignmentState, ResultWithStats, run_greedy_dfs_strict_prio};
use implementation::huitfeldt::huitfeldt_ttc;

const DAY_HEADER: &str = "algorithm,day,waitlist_before,patients_resolved,new_requests,waitlist_after,satisfaction_rate,cycles_found,avg_cycle_length,max_cycle_length,avg_wait_days,max_wait_days,cross_requests_added,cross_resolved,solve_ms";

const SUMMARY_HEADER: &str = "algorithm,num_patients,num_doctors,num_days,total_resolved,avg_daily_satisfaction_rate,\
avg_waitlist_size,min_waitlist_size,max_waitlist_size,final_waitlist_size,\
avg_cycles_per_day,avg_cycle_length_overall,\
resolved_count,resolved_avg_wait,resolved_std_wait,resolved_p50,resolved_p90,resolved_p95,resolved_p99,resolved_max,\
outstanding_count,outstanding_avg_wait,outstanding_p50,outstanding_p90,outstanding_p95,outstanding_p99,outstanding_max,\
overall_avg_wait,overall_max_wait,starvation_threshold_days,starved_resolved,starved_outstanding,\
num_districts,cross_district_prob,cross_added,within_added,\
cross_resolved_count,cross_resolved_avg,cross_resolved_p90,cross_resolved_p99,cross_resolved_max,\
within_resolved_count,within_resolved_avg,within_resolved_p90,within_resolved_p99,within_resolved_max,\
cross_outstanding_count,cross_outstanding_avg,within_outstanding_count,within_outstanding_avg,\
total_solve_ms,avg_solve_ms,max_solve_ms";

const HIST_HEADER: &str = "algorithm,kind,wait_days,count";

const DISTRICTS_HEADER: &str = "district_id,doctor_count,patient_count";

fn main() {
    let algorithms: Vec<(&str, fn(&mut AssignmentState) -> ResultWithStats)> = vec![
        ("Greedy DFS",        run_greedy_dfs_strict_prio),
        ("Huitfeldt TTC",     huitfeldt_ttc),
        ("Exact Cardinality", run_exact_cardinality),
        ("Exact Priority",    run_exact_priority),
        ("Util Linear",       run_util_linear),
        ("Util Exp 1.01",     run_util_exp_1_01),
        ("Util Exp 1.05",     run_util_exp_1_05),
        ("Util Exp 1.1",      run_util_exp_1_1),
        ("Util Exp 1.5",      run_util_exp_1_5),
        ("Util Exp 1.9",      run_util_exp_1_9),
    ];

    // Open all three output files up front and write headers, so each algorithm's
    // results can be flushed to disk the moment it finishes. A crash partway
    // through the run then keeps every algorithm that already completed.
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let out_dir = "simulation_results";
    std::fs::create_dir_all(out_dir).expect("failed to create output directory");
    let day_path = format!("{}/simulation_{}.csv", out_dir, timestamp);
    let summary_path = format!("{}/simulation_summary_{}.csv", out_dir, timestamp);
    let hist_path = format!("{}/simulation_wait_hist_{}.csv", out_dir, timestamp);
    let districts_path = format!("{}/simulation_districts_{}.csv", out_dir, timestamp);

    let mut day_w = BufWriter::new(File::create(&day_path).expect("failed to create day CSV"));
    let mut summary_w = BufWriter::new(File::create(&summary_path).expect("failed to create summary CSV"));
    let mut hist_w = BufWriter::new(File::create(&hist_path).expect("failed to create histogram CSV"));
    let mut districts_w = BufWriter::new(File::create(&districts_path).expect("failed to create districts CSV"));
    writeln!(day_w, "{}", DAY_HEADER).unwrap();
    writeln!(summary_w, "{}", SUMMARY_HEADER).unwrap();
    writeln!(hist_w, "{}", HIST_HEADER).unwrap();
    writeln!(districts_w, "{}", DISTRICTS_HEADER).unwrap();
    day_w.flush().unwrap();
    summary_w.flush().unwrap();
    hist_w.flush().unwrap();
    districts_w.flush().unwrap();

    println!("Writing incrementally to:\n  {}\n  {}\n  {}\n  {}\n", day_path, summary_path, hist_path, districts_path);

    let mut results: Vec<SimulationResult> = Vec::new();

    for (name, alg) in &algorithms {
        let config = SimulationConfig {
            num_patients: 100_000,
            num_doctors: 102,
            waitlist_fraction: 0.05,
            num_days: 365 * 2,
            new_requests_per_day: NewRequestMode::Fixed(18),
            min_new_requests_fraction: 0.0,
            algorithm: *alg,
            algorithm_name: name.to_string(),
            seed: 42,
            num_districts: 6,
            cross_district_prob: 0.11,
        };

        let result = run_simulation(config);
        result.print_table();

        // District structure is identical across algorithms (same seed), so write
        // it once from the first run.
        if results.is_empty() {
            write_district_rows(&mut districts_w, &result);
            districts_w.flush().unwrap();
        }

        // Flush this algorithm's rows to all three files immediately.
        write_day_rows(&mut day_w, &result);
        write_summary_row(&mut summary_w, &result);
        write_hist_rows(&mut hist_w, &result);
        day_w.flush().unwrap();
        summary_w.flush().unwrap();
        hist_w.flush().unwrap();

        results.push(result);
    }

    print_comparison(&results);

    println!("\nPer-day data : {}", day_path);
    println!("Summary      : {}", summary_path);
    println!("Wait hist    : {}", hist_path);
    println!("Districts    : {}", districts_path);
}

fn write_district_rows<W: Write>(w: &mut W, r: &SimulationResult) {
    for d in &r.district_stats {
        writeln!(w, "{},{},{}", d.district_id, d.doctor_count, d.patient_count).unwrap();
    }
}

/// Side-by-side comparison of the key metrics across all algorithms, so the
/// trade-offs are visible without scrolling through per-algorithm summaries.
fn print_comparison(results: &[SimulationResult]) {
    println!("\n=== Algorithm comparison ===");
    println!(
        "{:<18}  {:>9}  {:>6}  {:>9}  {:>9}  {:>9}  {:>9}  {:>11}  {:>12}",
        "Algorithm", "Resolved", "Sat%", "AvgWait", "P90Wait", "P99Wait", "MaxWait", "Outstanding", "AvgSolve/day"
    );
    println!("{}", "-".repeat(109));
    for r in results {
        println!(
            "{:<18}  {:>9}  {:>5.1}%  {:>8.1}d  {:>8}d  {:>8}d  {:>8}d  {:>11}  {:>10.3}ms",
            r.algorithm_name,
            r.total_resolved,
            r.avg_daily_satisfaction_rate * 100.0,
            r.resolved_wait.avg,
            r.resolved_wait.p90,
            r.resolved_wait.p99,
            r.overall_max_wait,
            r.outstanding_wait.count,
            r.avg_solve_ms,
        );
    }
    println!("{}", "-".repeat(109));
}

fn write_day_rows<W: Write>(w: &mut W, r: &SimulationResult) {
    for s in &r.day_stats {
        writeln!(
            w,
            "{},{},{},{},{},{},{:.6},{},{:.4},{},{:.4},{},{},{},{:.4}",
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
            s.cross_requests_added,
            s.cross_resolved,
            s.solve_ms,
        )
        .unwrap();
    }
}

fn write_summary_row<W: Write>(w: &mut W, r: &SimulationResult) {
    let rw = &r.resolved_wait;
    let ow = &r.outstanding_wait;
    let cd = &r.cross_district;
    writeln!(
        w,
        "{},{},{},{},{},{:.6},{:.4},{},{},{},{:.4},{:.4},\
{},{:.4},{:.4},{},{},{},{},{},\
{},{:.4},{},{},{},{},{},\
{:.4},{},{},{},{},\
{},{:.4},{},{},\
{},{:.4},{},{},{},\
{},{:.4},{},{},{},\
{},{:.4},{},{:.4},\
{:.4},{:.4},{:.4}",
        r.algorithm_name,
        r.num_patients,
        r.num_doctors,
        r.num_days,
        r.total_resolved,
        r.avg_daily_satisfaction_rate,
        r.avg_waitlist_size,
        r.min_waitlist_size,
        r.max_waitlist_size,
        r.final_waitlist_size,
        r.avg_cycles_per_day,
        r.avg_cycle_length_overall,
        rw.count, rw.avg, rw.std, rw.p50, rw.p90, rw.p95, rw.p99, rw.max,
        ow.count, ow.avg, ow.p50, ow.p90, ow.p95, ow.p99, ow.max,
        r.overall_avg_wait,
        r.overall_max_wait,
        r.starvation_threshold_days,
        r.starved_resolved,
        r.starved_outstanding,
        r.num_districts, r.cross_district_prob, cd.cross_added, cd.within_added,
        cd.resolved_cross.count, cd.resolved_cross.avg, cd.resolved_cross.p90, cd.resolved_cross.p99, cd.resolved_cross.max,
        cd.resolved_within.count, cd.resolved_within.avg, cd.resolved_within.p90, cd.resolved_within.p99, cd.resolved_within.max,
        cd.outstanding_cross.count, cd.outstanding_cross.avg, cd.outstanding_within.count, cd.outstanding_within.avg,
        r.total_solve_ms, r.avg_solve_ms, r.max_solve_ms,
    )
    .unwrap();
}

/// Full wait-time distributions in long format (one row per wait-day bucket).
/// Persisting the raw histograms means any further statistic — arbitrary
/// percentiles, alternative starvation thresholds, full CDFs — can be
/// recomputed from a finished run without re-running the simulation.
fn write_hist_rows<W: Write>(w: &mut W, r: &SimulationResult) {
    for (wait, &count) in r.wait_hist_resolved.iter().enumerate() {
        if count > 0 {
            writeln!(w, "{},resolved,{},{}", r.algorithm_name, wait, count).unwrap();
        }
    }
    for (wait, &count) in r.wait_hist_outstanding.iter().enumerate() {
        if count > 0 {
            writeln!(w, "{},outstanding,{},{}", r.algorithm_name, wait, count).unwrap();
        }
    }
}
