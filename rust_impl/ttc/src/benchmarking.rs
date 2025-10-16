use crate::ttc_scc::TTCSCCSolverV2;
use crate::{
    Doctor, Patient, SCCStats, TTCResultWithStats, TTCSCCSolver, TTCState, parse_data_file,
    ttc_algorithm_with_pruning,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmTiming {
    pub total_time_ms: f64,
    pub graph_building_ms: f64,
    pub scc_finding_ms: f64,
    pub cycle_finding_ms: f64,
    pub cycle_execution_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub file_name: String,
    pub num_patients: usize,
    pub num_doctors: usize,
    pub run_number: usize,
    pub dfs_timing: AlgorithmTiming,
    pub scc_v1_timing: AlgorithmTiming,
    pub scc_v2_timing: AlgorithmTiming,
    pub cycles_found: usize,
    pub patients_reassigned: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub file_name: String,
    pub num_patients: usize,
    pub num_doctors: usize,
    pub num_runs: usize,
    pub dfs_avg: AlgorithmTiming,
    pub scc_v1_avg: AlgorithmTiming,
    pub scc_v2_avg: AlgorithmTiming,
    pub avg_cycles_found: f64,
    pub avg_patients_reassigned: f64,
}

pub struct Benchmarker {
    data_files: Vec<String>,
    num_runs: usize,
    results: Vec<BenchmarkRun>,
}

impl Benchmarker {
    pub fn new(data_files: Vec<String>, num_runs: usize) -> Self {
        Self {
            data_files,
            num_runs,
            results: Vec::new(),
        }
    }

    pub fn run_benchmarks(&mut self) -> Result<(), String> {
        println!("\n{}", "=".repeat(80));
        println!("🚀 STARTING COMPREHENSIVE BENCHMARK");
        println!("{}", "=".repeat(80));
        println!("Files to benchmark: {}", self.data_files.len());
        println!("Runs per file: {}", self.num_runs);
        println!(
            "Total executions: {}",
            self.data_files.len() * self.num_runs * 3
        );
        println!("{}", "=".repeat(80));

        for (file_idx, file_path) in self.data_files.iter().enumerate() {
            println!(
                "\n📁 [{}/{}] Processing file: {}",
                file_idx + 1,
                self.data_files.len(),
                file_path
            );

            if !Path::new(file_path).exists() {
                eprintln!("⚠️  File not found: {}, skipping...", file_path);
                continue;
            }

            // Parse the data once
            let (patients, doctors) = match parse_data_file(file_path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("❌ Error parsing {}: {}", file_path, e);
                    continue;
                }
            };

            let num_patients = patients.len();
            let num_doctors = doctors.len();
            let file_name = Path::new(file_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            println!(
                "   Loaded {} patients, {} doctors",
                num_patients, num_doctors
            );

            // Run multiple iterations
            for run in 1..=self.num_runs {
                println!("\n   🔄 Run {}/{} for {}", run, self.num_runs, file_name);

                // DFS Algorithm
                println!("      [1/3] Running DFS Pruning...");
                let (dfs_result, dfs_timing) =
                    self.run_dfs_algorithm(patients.clone(), doctors.clone());

                // SCC V1 Algorithm
                println!("      [2/3] Running SCC V1...");
                let (scc_v1_result, scc_v1_timing) =
                    self.run_scc_v1_algorithm(patients.clone(), doctors.clone());

                // SCC V2 Algorithm
                println!("      [3/3] Running SCC V2...");
                let (scc_v2_result, scc_v2_timing) =
                    self.run_scc_v2_algorithm(patients.clone(), doctors.clone());

                // Verify results match
                if dfs_result.cycles_found != scc_v1_result.cycles_found
                    || dfs_result.cycles_found != scc_v2_result.cycles_found
                {
                    eprintln!(
                        "      ⚠️  WARNING: Result mismatch! DFS: {}, SCC V1: {}, SCC V2: {}",
                        dfs_result.cycles_found,
                        scc_v1_result.cycles_found,
                        scc_v2_result.cycles_found
                    );
                }

                // Print results before moving
                println!(
                    "      ✅ Completed: {} cycles, {} patients reassigned",
                    dfs_result.cycles_found, dfs_result.patients_reassigned
                );
                println!(
                    "         DFS: {:.2}ms | SCC V1: {:.2}ms | SCC V2: {:.2}ms",
                    dfs_timing.total_time_ms,
                    scc_v1_timing.total_time_ms,
                    scc_v2_timing.total_time_ms
                );

                // Store results
                let benchmark_run = BenchmarkRun {
                    file_name: file_name.clone(),
                    num_patients,
                    num_doctors,
                    run_number: run,
                    dfs_timing,
                    scc_v1_timing,
                    scc_v2_timing,
                    cycles_found: dfs_result.cycles_found,
                    patients_reassigned: dfs_result.patients_reassigned,
                };

                self.results.push(benchmark_run);
            }
        }

        Ok(())
    }

    fn run_dfs_algorithm(
        &self,
        patients: Vec<Patient>,
        doctors: Vec<Doctor>,
    ) -> (TTCResultWithStats, AlgorithmTiming) {
        let mut state = TTCState::new(patients, doctors);

        let start = Instant::now();
        let result = ttc_algorithm_with_pruning(&mut state);
        let total_time = start.elapsed();

        let timing = AlgorithmTiming {
            total_time_ms: total_time.as_secs_f64() * 1000.0,
            graph_building_ms: 0.0,
            scc_finding_ms: 0.0,
            cycle_finding_ms: 0.0,
            cycle_execution_ms: 0.0,
        };

        (result, timing)
    }

    fn run_scc_v1_algorithm(
        &self,
        patients: Vec<Patient>,
        doctors: Vec<Doctor>,
    ) -> (TTCResultWithStats, AlgorithmTiming) {
        let mut state = TTCState::new(patients, doctors);
        let mut solver = TTCSCCSolver::new();

        let start = Instant::now();
        let result = solver.solve(&mut state);
        let total_time = start.elapsed();

        let stats = solver.get_stats();
        let timing = self.convert_scc_stats_to_timing(stats, total_time);

        (result, timing)
    }

    fn run_scc_v2_algorithm(
        &self,
        patients: Vec<Patient>,
        doctors: Vec<Doctor>,
    ) -> (TTCResultWithStats, AlgorithmTiming) {
        let mut state = TTCState::new(patients, doctors);
        let mut solver = TTCSCCSolverV2::new();

        let start = Instant::now();
        let result = solver.solve(&mut state);
        let total_time = start.elapsed();

        let stats = solver.get_stats();
        let timing = self.convert_scc_stats_to_timing(stats, total_time);

        (result, timing)
    }

    fn convert_scc_stats_to_timing(
        &self,
        stats: &SCCStats,
        total_time: Duration,
    ) -> AlgorithmTiming {
        AlgorithmTiming {
            total_time_ms: total_time.as_secs_f64() * 1000.0,
            graph_building_ms: stats.time_graph_building.as_secs_f64() * 1000.0,
            scc_finding_ms: stats.time_scc_finding.as_secs_f64() * 1000.0,
            cycle_finding_ms: stats.time_cycle_finding.as_secs_f64() * 1000.0,
            cycle_execution_ms: stats.time_cycle_execution.as_secs_f64() * 1000.0,
        }
    }

    pub fn compute_summaries(&self) -> Vec<BenchmarkSummary> {
        let mut summaries = Vec::new();
        let mut file_groups: std::collections::HashMap<String, Vec<&BenchmarkRun>> =
            std::collections::HashMap::new();

        // Group results by file
        for result in &self.results {
            file_groups
                .entry(result.file_name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        // Compute averages for each file
        for (file_name, runs) in file_groups {
            if runs.is_empty() {
                continue;
            }

            let num_runs = runs.len();
            let first_run = runs[0];

            let dfs_avg = Self::average_timing(runs.iter().map(|r| &r.dfs_timing));
            let scc_v1_avg = Self::average_timing(runs.iter().map(|r| &r.scc_v1_timing));
            let scc_v2_avg = Self::average_timing(runs.iter().map(|r| &r.scc_v2_timing));

            let avg_cycles_found =
                runs.iter().map(|r| r.cycles_found as f64).sum::<f64>() / num_runs as f64;
            let avg_patients_reassigned = runs
                .iter()
                .map(|r| r.patients_reassigned as f64)
                .sum::<f64>()
                / num_runs as f64;

            summaries.push(BenchmarkSummary {
                file_name,
                num_patients: first_run.num_patients,
                num_doctors: first_run.num_doctors,
                num_runs,
                dfs_avg,
                scc_v1_avg,
                scc_v2_avg,
                avg_cycles_found,
                avg_patients_reassigned,
            });
        }

        // Sort by number of patients
        summaries.sort_by_key(|s| s.num_patients);
        summaries
    }

    fn average_timing<'a, I>(timings: I) -> AlgorithmTiming
    where
        I: Iterator<Item = &'a AlgorithmTiming>,
    {
        let timings: Vec<&AlgorithmTiming> = timings.collect();
        let count = timings.len() as f64;

        AlgorithmTiming {
            total_time_ms: timings.iter().map(|t| t.total_time_ms).sum::<f64>() / count,
            graph_building_ms: timings.iter().map(|t| t.graph_building_ms).sum::<f64>() / count,
            scc_finding_ms: timings.iter().map(|t| t.scc_finding_ms).sum::<f64>() / count,
            cycle_finding_ms: timings.iter().map(|t| t.cycle_finding_ms).sum::<f64>() / count,
            cycle_execution_ms: timings.iter().map(|t| t.cycle_execution_ms).sum::<f64>() / count,
        }
    }

    pub fn save_results(&self, output_path: &str) -> Result<(), String> {
        let summaries = self.compute_summaries();

        let mut file =
            File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;

        writeln!(file, "# Benchmark Results").map_err(|e| format!("Failed to write: {}", e))?;
        writeln!(file, "# Generated: {}", chrono::Local::now())
            .map_err(|e| format!("Failed to write: {}", e))?;
        writeln!(file, "# Runs per file: {}", self.num_runs)
            .map_err(|e| format!("Failed to write: {}", e))?;
        writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;

        writeln!(file, "[summary]").map_err(|e| format!("Failed to write: {}", e))?;
        for summary in &summaries {
            writeln!(file, "file={}", summary.file_name)
                .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file, "num_patients={}", summary.num_patients)
                .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file, "num_doctors={}", summary.num_doctors)
                .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file, "avg_cycles_found={:.2}", summary.avg_cycles_found)
                .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "avg_patients_reassigned={:.2}",
                summary.avg_patients_reassigned
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file, "dfs_total_ms={:.2}", summary.dfs_avg.total_time_ms)
                .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v1_total_ms={:.2}",
                summary.scc_v1_avg.total_time_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v1_graph_building_ms={:.2}",
                summary.scc_v1_avg.graph_building_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v1_scc_finding_ms={:.2}",
                summary.scc_v1_avg.scc_finding_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v1_cycle_finding_ms={:.2}",
                summary.scc_v1_avg.cycle_finding_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v1_cycle_execution_ms={:.2}",
                summary.scc_v1_avg.cycle_execution_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v2_total_ms={:.2}",
                summary.scc_v2_avg.total_time_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v2_graph_building_ms={:.2}",
                summary.scc_v2_avg.graph_building_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v2_scc_finding_ms={:.2}",
                summary.scc_v2_avg.scc_finding_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v2_cycle_finding_ms={:.2}",
                summary.scc_v2_avg.cycle_finding_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "scc_v2_cycle_execution_ms={:.2}",
                summary.scc_v2_avg.cycle_execution_ms
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
        }

        // Write detailed data for plotting
        writeln!(file, "[detailed_data]").map_err(|e| format!("Failed to write: {}", e))?;
        writeln!(
            file,
            "file_name,num_patients,num_doctors,run,dfs_total_ms,scc_v1_total_ms,scc_v2_total_ms,\
             scc_v1_graph_ms,scc_v1_scc_ms,scc_v1_cycle_ms,scc_v1_exec_ms,\
             scc_v2_graph_ms,scc_v2_scc_ms,scc_v2_cycle_ms,scc_v2_exec_ms,\
             cycles_found,patients_reassigned"
        )
        .map_err(|e| format!("Failed to write: {}", e))?;

        for result in &self.results {
            writeln!(
                file,
                "{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{}",
                result.file_name,
                result.num_patients,
                result.num_doctors,
                result.run_number,
                result.dfs_timing.total_time_ms,
                result.scc_v1_timing.total_time_ms,
                result.scc_v2_timing.total_time_ms,
                result.scc_v1_timing.graph_building_ms,
                result.scc_v1_timing.scc_finding_ms,
                result.scc_v1_timing.cycle_finding_ms,
                result.scc_v1_timing.cycle_execution_ms,
                result.scc_v2_timing.graph_building_ms,
                result.scc_v2_timing.scc_finding_ms,
                result.scc_v2_timing.cycle_finding_ms,
                result.scc_v2_timing.cycle_execution_ms,
                result.cycles_found,
                result.patients_reassigned
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
        }

        Ok(())
    }

    pub fn print_summary(&self) {
        let summaries = self.compute_summaries();

        println!("\n{}", "=".repeat(100));
        println!("📊 BENCHMARK SUMMARY");
        println!("{}", "=".repeat(100));

        for summary in &summaries {
            println!("\n📁 File: {}", summary.file_name);
            println!(
                "   {} patients, {} doctors ({} runs)",
                summary.num_patients, summary.num_doctors, summary.num_runs
            );
            println!(
                "   Results: {:.1} cycles, {:.1} patients reassigned",
                summary.avg_cycles_found, summary.avg_patients_reassigned
            );
            println!("\n   Algorithm Performance:");
            println!("      DFS:    {:>10.2}ms", summary.dfs_avg.total_time_ms);
            println!(
                "      SCC V1: {:>10.2}ms (graph: {:.2}ms, scc: {:.2}ms, cycle: {:.2}ms, exec: {:.2}ms)",
                summary.scc_v1_avg.total_time_ms,
                summary.scc_v1_avg.graph_building_ms,
                summary.scc_v1_avg.scc_finding_ms,
                summary.scc_v1_avg.cycle_finding_ms,
                summary.scc_v1_avg.cycle_execution_ms
            );
            println!(
                "      SCC V2: {:>10.2}ms (graph: {:.2}ms, scc: {:.2}ms, cycle: {:.2}ms, exec: {:.2}ms)",
                summary.scc_v2_avg.total_time_ms,
                summary.scc_v2_avg.graph_building_ms,
                summary.scc_v2_avg.scc_finding_ms,
                summary.scc_v2_avg.cycle_finding_ms,
                summary.scc_v2_avg.cycle_execution_ms
            );

            let dfs_vs_v1 = summary.dfs_avg.total_time_ms / summary.scc_v1_avg.total_time_ms;
            let dfs_vs_v2 = summary.dfs_avg.total_time_ms / summary.scc_v2_avg.total_time_ms;
            println!("\n   Speedup vs DFS:");
            println!("      SCC V1: {:.2}x", dfs_vs_v1);
            println!("      SCC V2: {:.2}x", dfs_vs_v2);
        }

        println!("\n{}", "=".repeat(100));
    }
}
