/// Run true_ttc_algorithm on a single file and print reassigned patient IDs (one per line).
/// Usage: cargo run --release --bin run_true_ttc -- <data_file>

use ttc::{true_ttc_algorithm, AssignmentState, parse_data_file};

fn main() {
    let path = std::env::args().nth(1).expect("usage: run_true_ttc <data_file>");

    let (patients, doctors) = parse_data_file(&path)
        .unwrap_or_else(|e| { eprintln!("parse error: {e}"); std::process::exit(1); });

    // Map priority -> patient id for lookup after algorithm runs
    let priority_to_id: std::collections::HashMap<usize, usize> = patients
        .iter()
        .filter(|p| !p.is_dummy)
        .map(|p| (p.priority, p.id))
        .collect();

    let mut state = AssignmentState::new(patients, doctors);
    let result = true_ttc_algorithm(&mut state);

    // Print summary to stderr so stdout stays clean for piping
    eprintln!(
        "reassigned={} wants_switch={}",
        result.patients_reassigned,
        priority_to_id.len()
    );

    // Print reassigned patient IDs to stdout (one per line)
    let mut ids: Vec<usize> = result
        .solution
        .iter()
        .filter_map(|&prio| priority_to_id.get(&prio).copied())
        .collect();
    ids.sort_unstable();
    for id in ids {
        println!("{id}");
    }
}
