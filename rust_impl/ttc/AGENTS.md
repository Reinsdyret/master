# Repository Guidelines

## Project Structure & Module Organization
The Rust crate lives under `src/`, with `lib.rs` re-exporting the reusable modules and `main.rs` orchestrating the end-to-end benchmark run. Core graph primitives sit in `graph.rs`, while cycle detection lives in `scc.rs` and the optimized solver in `ttc_scc.rs`. The benchmarking binary is split into `src/bin/scc_benchmark.rs` for standalone timing runs. Input fixtures belong in `data/`, and helper analysis scripts such as `analyze_benchmark.py`, `verify_cycles.py`, and `compare_cycles.py` sit at the repository root; keep generated artefacts (e.g., `benchmark_results.txt`) out of version control unless they document a regression.

## Build, Test, and Development Commands
- `cargo check` — fast type-check to validate incremental changes.
- `cargo fmt` — format Rust sources; run before every commit.
- `cargo clippy -- -D warnings` — catch lint regressions in graph and SCC logic.
- `cargo run --release` — execute the default benchmark driver against `data/test_133000_patient_4000_doctors_28_districts.txt`.
- `cargo run --release --bin scc_benchmark` — profile the Rust benchmarking entry point.
- `make scc_benchmark && ./scc_benchmark` — build and run the C++ comparison harness when cross-validating results.

## Coding Style & Naming Conventions
Adopt standard `rustfmt` output (four-space indentation, trailing commas on multi-line data, `snake_case` functions, `UpperCamelCase` types). Keep module names aligned with filenames (`ttc_scc`, `graph`) and prefer concise helper functions over long procedural blocks in `main.rs`. Document assumptions with `///` doc comments when APIs are exposed via `lib.rs`, and include quick inline comments only where algorithmic steps are non-obvious.

## Testing Guidelines
Unit tests belong alongside modules using `#[cfg(test)]` blocks; focus coverage on graph construction and SCC invariants. Run `cargo test` before opening a pull request, adding `-- --nocapture` when debugging failing cases. For integration-style validation, reuse the Python scripts in the repository (`python3 verify_cycles.py`) to compare cycle outputs across algorithms and time series files under `data/`.

## Commit & Pull Request Guidelines
Use short, present-tense commit summaries (`module: add V2 SCC solver`) followed by optional body lines describing rationale and datasets touched. Group related Rust, C++, and Python updates in a single commit only when they compile and run together. Pull requests should outline the change, list any new data inputs, attach relevant benchmark numbers, and reference GitHub issues or TODOs when applicable. Screenshots are unnecessary; prefer attaching command output snippets for performance comparisons.

## Benchmarking & Data Notes
When producing new timing results, capture raw output in a scratch file under `benchmark_results.txt` or a similarly named artefact ignored by Git, then feed it into `analyze_benchmark.py` for summary tables. Large synthetic datasets should be generated with `python3 generate_test_data.py` and stored outside the repository, documenting their paths in the PR description so collaborators can reproduce the run.
