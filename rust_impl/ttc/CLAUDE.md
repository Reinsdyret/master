# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of the Top Trading Cycles (TTC) algorithm for patient-doctor reassignment optimization. The project compares two algorithmic approaches:

1. **DFS-based (Pruning)**: Sequential search through patients by priority, finding cycles via depth-first search with pruning optimization
2. **SCC-based**: Uses Kosaraju's algorithm to find strongly connected components, then extracts cycles from each SCC

Both algorithms process the same patient-doctor reassignment problem but use different graph traversal strategies.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run with optimizations (recommended for benchmarking)
cargo build --release --bin ttc
cargo run --release --bin ttc

# Run in debug mode
cargo run --bin ttc

# Run tests
cargo test
```

## Data Format

Input files (in `data/`) follow this format:
```
num_patients,num_doctors
preferred_doctor_1,preferred_doctor_2,...
current_doctor_1,current_doctor_2,...
priority_1,priority_2,...
```



The main program currently uses `data/test_10000_patient_1000_doctors_5_districts.txt`.

## Core Architecture

### Data Model ([src/lib.rs](src/lib.rs))

- **Patient**: Has an ID, priority (lower = higher priority), preferred doctor, current doctor, and flags (`wants_to_switch`, `is_stuck`)
- **Doctor**: Has an ID and maintains a priority-sorted list of `switching_patients`
- **TTCState**: Container managing all patients and doctors, with patients indexed by ID and sorted by priority

### Algorithm Implementations

1. **DFS/Pruning** ([src/lib.rs](src/lib.rs)):
   - `ttc_algorithm_with_pruning()`: Main entry point
   - Iterates through patients in strict priority order
   - Uses `find_cycle_from_patient_with_direct_pruning()` to find cycles via DFS
   - Marks patients as "stuck" if they cannot form any cycle (pruning optimization)
   - Writes found cycles to `dfs_cycles.txt`

2. **SCC-based** ([src/ttc_scc.rs](src/ttc_scc.rs)):
   - `TTCSCCSolver`: Original version (finds all SCCs, processes by lowest priority in each)
   - `TTCSCCSolverV2`: Sequential version (processes patients in priority order like DFS)
   - Uses `KosajaruSCC` ([src/scc.rs](src/scc.rs)) to find strongly connected components
   - Builds patient-to-patient adjacency list (no doctor nodes)
   - Writes found cycles to `scc_cycles.txt`

### Graph Utilities ([src/graph.rs](src/graph.rs))

- `TTCGraph`: Provides graph representation with adjacency lists
- Currently used for testing/analysis, not in main algorithm flow

### Main Program ([src/main.rs](src/main.rs))

Benchmarks both algorithms side-by-side:
- Parses input data
- Runs DFS pruning algorithm
- Runs SCC algorithm
- Compares execution times and result metrics
- Prints statistics about cycles found, patients reassigned, and algorithm performance

## Key Implementation Details

### Patient Priority

Patients are processed in ascending priority order (lower number = higher priority). The DFS version processes strictly sequentially by priority. The SCC version (V2) also processes sequentially to match DFS behavior.

### Cycle Detection

Both algorithms find cycles in the preference graph:
- A cycle means patients can swap doctors in a way that satisfies all participants
- Example: Patient A wants Doctor 2 → Patient B (at Doctor 2) wants Doctor 3 → Patient C (at Doctor 3) wants Doctor 1 → Patient A (at Doctor 1)

### Graph Structure

The graph is patient-to-patient:
- Edge from Patient A to Patient B exists if A wants the doctor that B currently has
- Self-loops are excluded
- Only patients who `wants_to_switch && !is_stuck` are included

### Known Issues

As indicated by recent commits, there's a discrepancy between the DFS and SCC methods that needs investigation. The algorithms should produce equivalent results but currently show differences.

## Python Verification Scripts

Several Python scripts exist for debugging:
- `generate_test_data.py`: Creates test data files
- `compare_cycles.py`: Compares cycles from DFS vs SCC output
- `verify_cycles.py`: Validates that found cycles are valid
- `check_patient_differences.py`: Checks which patients differ between algorithms
- `check_dfs_in_sccs.py`: Verifies DFS cycles appear within SCCs

## Development Notes

- The edition in Cargo.toml is set to "2024" (likely should be "2021")
- Progress bars use `indicatif` crate
- Both algorithms write debug output to text files for comparison
- Tests are partially disabled in `scc.rs` and `graph.rs` due to needing full TTCState
