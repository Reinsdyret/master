# Patient Reallocation and Waitlist Reduction in the Norwegian GP System

Master's thesis by Lars Møen Haukland, University of Bergen: Department of Informatics.

## Abstract

Many assignment problems share a common structure: each agent already holds one item, such as a house, a school place, or a doctor, and some agents would prefer to hold an item currently held by someone else.
When no spare items are available, the only way to satisfy these agents is to find cycles of agents who can exchange their items simultaneously.
This thesis studies how to find such exchanges, motivated by the allocation of general practitioners (GPs) in Norway, where people wanting to switch GP must wait for a slot to open because almost all GP lists are full.
We formalise the problem as one of finding cycles in a directed graph of patients and GPs, and define three notions of a good set of exchanges, one that resolves as many patients as possible, one that gives strict priority to those who have waited the longest, and one that balances the two.
We develop exact algorithms based on cycle cancelling for each of these, together with a fast heuristic, and compare them against an existing mechanism in a simulation of the Norwegian GP system.
We find that the choice of objective has little effect on how many patients are resolved, but a large effect on which patients are resolved and therefore on how long they wait, and that the algorithms differ widely in running time.

## Repository layout

- `implementation/`: Rust crate with all algorithms (exact cardinality, exact priority, utility variants, greedy heuristic, Huitfeldt TTC) and the GP-system simulation, plus the Python plotting scripts.
- `thesis/`: the thesis source (Typst).

## Running the simulation

Requires a [Rust toolchain](https://rustup.rs/).

```sh
git clone <repo-url>
cd implementation
cargo run --release --bin simulation
```

The run compares every algorithm and writes timestamped CSVs into `implementation/simulation_results/`:

- `simulation_<ts>.csv`: per-day metrics
- `simulation_summary_<ts>.csv`: one summary row per algorithm
- `simulation_wait_hist_<ts>.csv`: waiting-time histograms
- `simulation_districts_<ts>.csv`: district structure

### Changing the configuration

Edit the `SimulationConfig` block in `implementation/src/bin/simulation.rs` (number of patients, doctors, days, new requests per day, districts, seed, etc.), then rerun the command above.

## Plotting the results

Plotting needs Python with matplotlib:

```sh
pip install matplotlib
```

Run from inside `implementation/`, passing the CSVs the simulation produced (replace `<ts>` with the timestamp of your run):

```sh
# Per-day time series (e.g. waitlist size, satisfaction rate)
python plot_simulation.py simulation_results/simulation_<ts>.csv --metric avg_wait_days

# Summary comparison across algorithms
python plot_summary.py simulation_results/simulation_summary_<ts>.csv

# Waiting-time histograms / CDFs
python plot_wait_hist.py simulation_results/simulation_wait_hist_<ts>.csv --kind both

# District structure
python plot_districts.py simulation_results/simulation_districts_<ts>.csv \
    --summary simulation_results/simulation_summary_<ts>.csv
```

Pass `-h` to any script for its full set of options. Generated figures are written to the `plots/` directory.
