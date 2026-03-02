# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of the Top Trading Cycles (TTC) algorithm for patient-doctor reassignment optimization. The project includes multiple algorithmic approaches:

1. **Heuristic TTC (Primary)**: Sequential cycle-finding algorithm with multiple priority strategies
2. **Exact Algorithm (CyclePacker)**: Optimal solution finder using residual graph and positive cycle detection
3. **SCC-based (Historical)**: Uses Tarjan's algorithm to find strongly connected components (currently commented out)

The system models a matching market where patients want to switch to different doctors, and the algorithm finds cycles of exchanges that satisfy all participants.

## Build and Run Commands

```bash
# Standard development
cargo check              # Fast type-check for incremental changes
cargo fmt                # Format code (run before commits)
cargo clippy -- -D warnings  # Lint checking

# Build and run
cargo build              # Debug build
cargo build --release    # Optimized build
cargo run --bin ttc      # Run main benchmark in debug mode
cargo run --release --bin ttc  # Run with optimizations (recommended)

# Testing
cargo test              # Run all tests
cargo test -- --nocapture  # Run tests with output
```

## Data Format

Input files in `data/` follow this format:
```
num_patients,num_doctors
preferred_doctor_1,preferred_doctor_2,...
current_doctor_1,current_doctor_2,...
priority_1,priority_2,...
district_1,district_2,...  (optional)
capacity_1,capacity_2,...  (optional, line 6)
```

Key points:
- Patient/doctor IDs start from 1
- Doctor ID 0 is reserved for unassigned patients (dummy doctor)
- `current_doctor = 0` or `None` means patient is unassigned
- Capacities define maximum patients per doctor

## Core Architecture

### Module Structure

- **src/lib.rs**: Core data structures (`Patient`, `Doctor`, `TTCState`) and main TTC algorithm with priority strategies
- **src/excact.rs**: `CyclePacker` - optimal cycle finding using residual graphs
- **src/benchmarking.rs**: Framework for comparing algorithms with timing and metrics
- **src/scc.rs**: `TarjanSCC` implementation for strongly connected components
- **src/ttc_scc.rs**: Historical SCC-based TTC solver (currently commented out)
- **src/main.rs**: Benchmark orchestration - runs CyclePacker and various TTC strategies

### Key Data Structures

**Patient**:
- `id`: Unique identifier (1-indexed)
- `is_dummy`: Whether this is a dummy patient filling unused capacity
- `priority`: Lower number = higher priority
- `preferred_doctor`: Desired doctor ID
- `current_doctor`: `Option<usize>` - current assignment or None
- `wants_to_switch`: Computed flag
- `is_stuck`: Marked when patient cannot be satisfied (pruning optimization)

**Doctor**:
- `id`: Unique identifier (1-indexed, 0 = dummy)
- `is_dummy`: Whether this is the dummy doctor for unassigned patients
- `capacity`: Maximum patients this doctor can serve
- `switching_patients`: Priority-sorted list of patients wanting to switch
- `assigned_patients`: All currently assigned patient IDs

**TTCState**:
- Container managing all patients and doctors
- Maintains patients sorted by priority
- Tracks total availability and assignments

### Priority Strategies

The `PriorityStrategy` enum controls patient processing order:
- `StrictPriority`: Process by priority number (default behavior)
- `UnassignedFirst`: Prioritize unassigned patients, then by priority
- `Random`: Shuffle randomly for baseline comparison
- `HighDemandFirst`: Process patients wanting popular doctors first
- `LowDemandFirst`: Process patients wanting unpopular doctors first

Different strategies can produce different outcomes. The benchmarking framework compares them.

### CyclePacker (Exact Algorithm)

The exact algorithm in `src/excact.rs` finds the optimal solution:

1. **Graph Construction**: Builds doctor-to-doctor graph where edges represent patient flow
   - Edge (u, v) with capacity k means k patients at doctor u want doctor v
   - Edges have cost +1 (original) and -1 (residual)

2. **Positive Cycle Detection**: Iteratively finds cycles with positive total cost
   - Uses DFS to detect cycles in the residual graph
   - A positive cycle means net gain in satisfied patients

3. **Cycle Application**: Applies cycles to residual graph until no positive cycles remain
   - Updates edge capacities and reverse edges
   - Terminates when optimal solution is found

4. **Verification**: `verify_solution()` checks optimality constraints

Key methods:
- `new(patients, doctors)`: Builds initial graph
- `pack_cycles()`: Main optimization loop
- `find_positive_cycle()`: DFS-based cycle detection
- `get_solution_edges()`: Returns final matching edges

### Heuristic TTC Algorithm

The main TTC implementation (`ttc_algorithm` in `src/lib.rs`):

1. Orders patients by chosen priority strategy
2. For each patient who wants to switch:
   - Attempts to find a cycle starting from that patient using DFS
   - If found, executes the cycle (all patients in cycle switch)
   - If not found and all paths exhausted, marks patient as "stuck"
3. Returns statistics on cycles found, patients reassigned, etc.

Graph structure is patient-to-patient:
- Edge from Patient A to Patient B exists if A wants the doctor that B currently has
- Only includes patients where `wants_to_switch && !is_stuck`
- Self-loops are excluded

### Benchmarking Framework

The `benchmarking` module provides:
- `AlgorithmConfig`: Wrapper for algorithm name and function pointer
- `Benchmarker`: Runs multiple algorithms across datasets with timing
- `AlgorithmResult`: Captures metrics like satisfaction rate, capacity utilization, execution time
- Comparison utilities for lexicographic priority (which solution is "better")

## Key Implementation Details

### Dummy Nodes

The system uses dummy nodes to model capacity:
- **Dummy Doctor (ID 0)**: Represents the "unassigned" state for patients
- **Dummy Patients**: Created for each unit of unused capacity at real doctors
  - Priority = `usize::MAX` (processed last)
  - Current doctor = the real doctor with capacity
  - Preferred doctor = 0 (dummy doctor)
  - Allows capacity to participate in cycles

### Cycle Detection

Cycles represent valid exchanges where all participants improve:
- Example: Patient A (at Doctor 1, wants Doctor 2) → Patient B (at Doctor 2, wants Doctor 3) → Patient C (at Doctor 3, wants Doctor 1)
- All three patients can simultaneously switch to their preferred doctor

### Solution Quality Metrics

The benchmarking framework tracks:
- **Patients reassigned**: Count of patients who got preferred doctor
- **Satisfaction rate**: Percentage of switching patients satisfied
- **Unassigned resolution rate**: Percentage of unassigned patients matched
- **Capacity utilization**: Percentage of doctor capacity used
- **Cycles found**: Number of cycles executed
- **Timing breakdowns**: Graph building, cycle finding, cycle execution

## Python Analysis Scripts

Several Python scripts assist with debugging and validation:

- `generate_test_data.py`: Creates synthetic test datasets
- `verify_cycles.py`: Validates that found cycles are valid exchanges
- `compare_cycles.py`: Compares cycle outputs between algorithms
- `analyze_benchmark.py`: Parses and summarizes benchmark results
- `plot_benchmarks.py`: Generates visualizations of benchmark data
- `check_patient_differences.py`: Identifies which patients differ between algorithm outputs
- `check_dfs_in_sccs.py`: Verifies DFS cycles appear within SCCs

## Development Notes

- The crate edition is set to "2024" in Cargo.toml (non-standard; Rust 2021 is latest stable edition)
- Progress bars use the `indicatif` crate
- Hash maps use `rustc-hash` (`FxHashMap`) for performance
- Graph operations use `petgraph` crate
- The `serde` crate enables result serialization
- Main program currently focuses on exact algorithm (`CyclePacker`)
- Test data files use descriptive names encoding problem parameters (e.g., `test_1000_patient_30_doctors_0_unassigned.txt`)

## Common Workflows

**Adding a new priority strategy**:
1. Add variant to `PriorityStrategy` enum in `src/lib.rs`
2. Implement ordering logic in `TTCState::sort_patients_by_strategy()`
3. Create wrapper function in `src/main.rs`
4. Add to `algorithms` vec in `main()`

**Benchmarking a dataset**:
1. Add file path to `data_files` vec in `src/main.rs`
2. Run `cargo run --release --bin ttc`
3. Results saved to `benchmark_results_comprehensive.txt`
4. Analyze with `python3 analyze_benchmark.py`

**Validating algorithm correctness**:
1. Enable cycle logging in algorithm implementations
2. Run comparison: `cargo run --release --bin ttc`
3. Use Python scripts to validate: `python3 verify_cycles.py`
