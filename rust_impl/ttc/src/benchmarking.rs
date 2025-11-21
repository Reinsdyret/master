use crate::{
    Doctor, Patient, TTCResultWithStats, TTCState, parse_data_file,
    ttc_algorithm_with_pruning,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::os::linux::raw::stat;
use std::path::Path;
use std::time::{Duration, Instant};

/// Type alias for TTC algorithm functions
/// Takes a mutable reference to TTCState and returns results with stats
pub type AlgorithmFn = fn(&mut TTCState) -> TTCResultWithStats;

/// Configuration for a TTC algorithm to benchmark
pub struct AlgorithmConfig {
    pub name: String,
    pub run_fn: AlgorithmFn,
}

impl AlgorithmConfig {
    pub fn new(name: impl Into<String>, run_fn: AlgorithmFn) -> Self {
        Self {
            name: name.into(),
            run_fn,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmTiming {
    pub total_time_ms: f64,
    pub graph_building_ms: f64,
    pub scc_finding_ms: f64,
    pub cycle_finding_ms: f64,
    pub cycle_execution_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmResult {
    pub timing: AlgorithmTiming,
    pub cycles_found: usize,
    pub patients_reassigned: usize,
    pub remaining_capacity: usize,
    pub unassigned_matched: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub file_name: String,
    pub num_patients: usize,
    pub num_doctors: usize,
    pub run_number: usize,
    /// Results keyed by algorithm name
    pub algorithm_results: HashMap<String, AlgorithmResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmSummary {
    pub avg_timing: AlgorithmTiming,
    pub avg_cycles_found: f64,
    pub avg_patients_reassigned: f64,
    pub avg_remaining_capacity: f64,
    pub avg_unassigned_matched: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub file_name: String,
    pub num_patients: usize,
    pub num_doctors: usize,
    pub num_runs: usize,
    /// Algorithm summaries keyed by algorithm name
    pub algorithm_summaries: HashMap<String, AlgorithmSummary>,
}

pub struct Benchmarker {
    data_files: Vec<String>,
    num_runs: usize,
    algorithms: Vec<AlgorithmConfig>,
    results: Vec<BenchmarkRun>,
}

impl Benchmarker {
    pub fn new(data_files: Vec<String>, num_runs: usize, algorithms: Vec<AlgorithmConfig>) -> Self {
        Self {
            data_files,
            num_runs,
            algorithms,
            results: Vec::new(),
        }
    }

    pub fn run_benchmarks(&mut self) -> Result<(), String> {
        println!("\n{}", "=".repeat(80));
        println!("Files to benchmark: {}", self.data_files.len());
        println!("Algorithms to test: {}", self.algorithms.len());
        for algo in &self.algorithms {
            println!("   - {}", algo.name);
        }
        println!("Runs per file: {}", self.num_runs);
        println!(
            "Total executions: {}",
            self.data_files.len() * self.num_runs * self.algorithms.len()
        );
        println!("{}", "=".repeat(80));

        for (file_idx, file_path) in self.data_files.iter().enumerate() {
            println!(
                "\n[{}/{}] Processing file: {}",
                file_idx + 1,
                self.data_files.len(),
                file_path
            );

            if !Path::new(file_path).exists() {
                eprintln!("File not found: {}, skipping...", file_path);
                continue;
            }

            // Parse the data once
            let (patients, doctors) = match parse_data_file(file_path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error parsing {}: {}", file_path, e);
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
                println!("\n   Run {}/{} for {}", run, self.num_runs, file_name);

                let mut algorithm_results = HashMap::new();

                // Run each algorithm
                for algo in &self.algorithms {
                    println!("      Running {}...", algo.name);
                    let (result, timing, unassigned_matched) =
                        self.run_algorithm(algo, patients.clone(), doctors.clone());

                    println!(
                        "         Completed: {} cycles, {} patients reassigned, {} unassigned matched",
                        result.cycles_found, result.patients_reassigned, unassigned_matched
                    );
                    println!("         Remaining capacity: {}", result.remaining_capacity);
                    println!("         Time: {:.2}ms", timing.total_time_ms);

                    algorithm_results.insert(
                        algo.name.clone(),
                        AlgorithmResult {
                            timing,
                            cycles_found: result.cycles_found,
                            patients_reassigned: result.patients_reassigned,
                            remaining_capacity: result.remaining_capacity,
                            unassigned_matched,
                        },
                    );
                }

                // Store results
                let benchmark_run = BenchmarkRun {
                    file_name: file_name.clone(),
                    num_patients,
                    num_doctors,
                    run_number: run,
                    algorithm_results,
                };

                self.results.push(benchmark_run);
            }
        }

        Ok(())
    }

    /// Generic algorithm runner that handles timing and state management
    fn run_algorithm(
        &self,
        algo: &AlgorithmConfig,
        patients: Vec<Patient>,
        doctors: Vec<Doctor>,
    ) -> (TTCResultWithStats, AlgorithmTiming, usize) {
        let unassigned_patients = patients
            .iter()
            .filter(|p| p.current_doctor == Some(0))
            .count();

        let mut state = TTCState::new(patients, doctors);

        let start = Instant::now();
        let result = (algo.run_fn)(&mut state);
        let total_time = start.elapsed();

        let unassigned_patients_now = state
            .patients
            .iter()
            .filter(|p| p.current_doctor == Some(0) && p.priority != usize::MAX)
            .count();
        let unassigned_matched = unassigned_patients - unassigned_patients_now;

        let timing = AlgorithmTiming {
            total_time_ms: total_time.as_secs_f64() * 1000.0,
            graph_building_ms: 0.0,
            scc_finding_ms: 0.0,
            cycle_finding_ms: 0.0,
            cycle_execution_ms: 0.0,
        };

        (result, timing, unassigned_matched)
    }

    pub fn compute_summaries(&self) -> Vec<BenchmarkSummary> {
        let mut summaries = Vec::new();
        let mut file_groups: HashMap<String, Vec<&BenchmarkRun>> = HashMap::new();

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

            // Compute summaries for each algorithm
            let mut algorithm_summaries = HashMap::new();

            // Get all algorithm names from the first run
            for algo_name in first_run.algorithm_results.keys() {
                let algo_results: Vec<&AlgorithmResult> = runs
                    .iter()
                    .filter_map(|r| r.algorithm_results.get(algo_name))
                    .collect();

                if !algo_results.is_empty() {
                    let count = algo_results.len() as f64;

                    algorithm_summaries.insert(
                        algo_name.clone(),
                        AlgorithmSummary {
                            avg_timing: Self::average_timing(algo_results.iter().map(|r| &r.timing)),
                            avg_cycles_found: algo_results.iter().map(|r| r.cycles_found as f64).sum::<f64>() / count,
                            avg_patients_reassigned: algo_results.iter().map(|r| r.patients_reassigned as f64).sum::<f64>() / count,
                            avg_remaining_capacity: algo_results.iter().map(|r| r.remaining_capacity as f64).sum::<f64>() / count,
                            avg_unassigned_matched: algo_results.iter().map(|r| r.unassigned_matched as f64).sum::<f64>() / count,
                        },
                    );
                }
            }

            summaries.push(BenchmarkSummary {
                file_name,
                num_patients: first_run.num_patients,
                num_doctors: first_run.num_doctors,
                num_runs,
                algorithm_summaries,
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
        writeln!(file, "# Algorithms: {}", self.algorithms.iter().map(|a| &a.name).cloned().collect::<Vec<_>>().join(", "))
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

            // Get sorted algorithm names for consistent output
            let mut algo_names: Vec<&String> = summary.algorithm_summaries.keys().collect();
            algo_names.sort();

            for algo_name in algo_names {
                if let Some(algo_summary) = summary.algorithm_summaries.get(algo_name) {
                    writeln!(file, "{}_total_ms={:.2}", algo_name.to_lowercase().replace(" ", "_"), algo_summary.avg_timing.total_time_ms)
                        .map_err(|e| format!("Failed to write: {}", e))?;
                    writeln!(file, "{}_cycles={:.2}", algo_name.to_lowercase().replace(" ", "_"), algo_summary.avg_cycles_found)
                        .map_err(|e| format!("Failed to write: {}", e))?;
                    writeln!(file, "{}_patients_reassigned={:.2}", algo_name.to_lowercase().replace(" ", "_"), algo_summary.avg_patients_reassigned)
                        .map_err(|e| format!("Failed to write: {}", e))?;
                    writeln!(file, "{}_unassigned_matched={:.2}", algo_name.to_lowercase().replace(" ", "_"), algo_summary.avg_unassigned_matched)
                        .map_err(|e| format!("Failed to write: {}", e))?;
                    writeln!(file, "{}_remaining_capacity={:.2}", algo_name.to_lowercase().replace(" ", "_"), algo_summary.avg_remaining_capacity)
                        .map_err(|e| format!("Failed to write: {}", e))?;
                }
            }
            writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
        }

        // Write detailed data for plotting
        writeln!(file, "[detailed_data]").map_err(|e| format!("Failed to write: {}", e))?;

        // Build dynamic CSV header
        let mut header = "file_name,num_patients,num_doctors,run".to_string();
        if !self.results.is_empty() && !self.results[0].algorithm_results.is_empty() {
            let mut algo_names: Vec<&String> = self.results[0].algorithm_results.keys().collect();
            algo_names.sort();
            for algo_name in &algo_names {
                let safe_name = algo_name.to_lowercase().replace(" ", "_");
                header.push_str(&format!(",{}_total_ms,{}_cycles,{}_patients_reassigned,{}_unassigned_matched,{}_remaining_capacity",
                    safe_name, safe_name, safe_name, safe_name, safe_name));
            }
        }
        writeln!(file, "{}", header).map_err(|e| format!("Failed to write: {}", e))?;

        // Write detailed data rows
        for result in &self.results {
            let mut row = format!(
                "{},{},{},{}",
                result.file_name,
                result.num_patients,
                result.num_doctors,
                result.run_number
            );

            let mut algo_names: Vec<&String> = result.algorithm_results.keys().collect();
            algo_names.sort();

            for algo_name in &algo_names {
                if let Some(algo_result) = result.algorithm_results.get(*algo_name) {
                    row.push_str(&format!(
                        ",{:.2},{},{},{},{}",
                        algo_result.timing.total_time_ms,
                        algo_result.cycles_found,
                        algo_result.patients_reassigned,
                        algo_result.unassigned_matched,
                        algo_result.remaining_capacity
                    ));
                }
            }
            writeln!(file, "{}", row).map_err(|e| format!("Failed to write: {}", e))?;
        }

        Ok(())
    }

    pub fn print_summary(&self) {
        let summaries = self.compute_summaries();

        println!("\n{}", "=".repeat(100));
        println!("BENCHMARK SUMMARY");
        println!("{}", "=".repeat(100));

        for summary in &summaries {
            println!("\nFile: {}", summary.file_name);
            println!(
                "   {} patients, {} doctors ({} runs)",
                summary.num_patients, summary.num_doctors, summary.num_runs
            );

            // Get algorithm names and sort them for consistent output
            let mut algo_names: Vec<&String> = summary.algorithm_summaries.keys().collect();
            algo_names.sort();

            println!("\n   Algorithm Performance:");
            for algo_name in algo_names {
                if let Some(algo_summary) = summary.algorithm_summaries.get(algo_name) {
                    println!("      {}:", algo_name);
                    println!("         Time:                   {:>10.2}ms", algo_summary.avg_timing.total_time_ms);
                    println!("         Cycles:                 {:>10.1}", algo_summary.avg_cycles_found);
                    println!("         Patients reassigned:    {:>10.1}", algo_summary.avg_patients_reassigned);
                    println!("         Unassigned matched:     {:>10.1}", algo_summary.avg_unassigned_matched);
                    println!("         Remaining capacity:     {:>10.1}", algo_summary.avg_remaining_capacity);
                }
            }
        }

        println!("\n{}", "=".repeat(100));
    }
}
