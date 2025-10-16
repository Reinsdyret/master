use std::env;
use ttc::{TTCState, parse_data_file};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <data_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    match parse_data_file(filename) {
        Ok((patients, doctors)) => {
            println!(
                "Loading data: {} patients, {} doctors",
                patients.len(),
                doctors.len()
            );

            let state = TTCState::new(patients, doctors);

            // Build patient-to-patient graph
            println!("Building patient-to-patient graph...");
            let (graph, active_patients, edge_count) = build_patient_graph(&state);

            println!(
                "Graph built: {} active patients, {} edges",
                active_patients, edge_count
            );

            // Find SCCs using Tarjan's algorithm
            println!("Finding SCCs...");
            let start = std::time::Instant::now();

            let sccs = find_sccs(&graph);

            let duration = start.elapsed();

            // Count SCCs with size > 1
            let large_sccs = sccs.iter().filter(|scc| scc.len() > 1).count();
            let largest_scc = sccs.iter().map(|scc| scc.len()).max().unwrap_or(0);

            println!("\n=== Results ===");
            println!("Total SCCs: {}", sccs.len());
            println!("SCCs with size > 1: {}", large_sccs);
            println!("Largest SCC size: {}", largest_scc);
            println!("Time: {} ms", duration.as_millis());

            // println!("SCCS: ");
            // for scc in sccs {
            //     println!("{:?}", scc);
            // }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn build_patient_graph(state: &TTCState) -> (Vec<Vec<usize>>, usize, usize) {
    // Map from doctor to patients who currently have that doctor
    let mut doctor_to_patients: Vec<Vec<usize>> = vec![Vec::new(); state.doctors.len() + 1];

    let mut active_patients = 0;
    let mut max_patient_id = 0;

    for patient in &state.patients {
        if patient.wants_to_switch && !patient.is_stuck {
            active_patients += 1;
            doctor_to_patients[patient.current_doctor].push(patient.id);
            if patient.id > max_patient_id {
                max_patient_id = patient.id;
            }
        }
    }

    // Build adjacency list graph
    let mut graph = vec![Vec::new(); max_patient_id + 1];
    let mut edge_count = 0;

    // Add edges: patient i -> patient j if i wants j's doctor
    for patient in &state.patients {
        if !patient.wants_to_switch || patient.is_stuck {
            continue;
        }

        let wanted_doctor = patient.preferred_doctor;

        for &other_patient_id in &doctor_to_patients[wanted_doctor] {
            if patient.id != other_patient_id {
                if let Some(other_patient) = state.get_patient(other_patient_id) {
                    if other_patient.wants_to_switch && !other_patient.is_stuck {
                        graph[patient.id].push(other_patient_id);
                        edge_count += 1;
                    }
                }
            }
        }
    }

    (graph, active_patients, edge_count)
}

// Tarjan's SCC algorithm - same implementation as C++
fn find_sccs(graph: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let n = graph.len();
    let mut disc = vec![-1; n];
    let mut low = vec![0; n];
    let mut onstack = vec![false; n];
    let mut scc = Vec::new();
    let mut foundat = 1;
    let mut stack = Vec::new();

    for v in 0..n {
        if disc[v] == -1 {
            tarjan(
                v,
                graph,
                &mut disc,
                &mut low,
                &mut onstack,
                &mut scc,
                &mut foundat,
                &mut stack,
            );
        }
    }

    scc
}

fn tarjan(
    u: usize,
    graph: &Vec<Vec<usize>>,
    disc: &mut Vec<i32>,
    low: &mut Vec<i32>,
    onstack: &mut Vec<bool>,
    scc: &mut Vec<Vec<usize>>,
    foundat: &mut i32,
    stack: &mut Vec<usize>,
) {
    disc[u] = *foundat;
    low[u] = *foundat;
    *foundat += 1;
    stack.push(u);
    onstack[u] = true;

    for &i in &graph[u] {
        if disc[i] == -1 {
            tarjan(i, graph, disc, low, onstack, scc, foundat, stack);
            low[u] = low[u].min(low[i]);
        } else if onstack[i] {
            low[u] = low[u].min(disc[i]);
        }
    }

    if disc[u] == low[u] {
        let mut scctem = Vec::new();
        loop {
            let v = stack.pop().unwrap();
            onstack[v] = false;
            scctem.push(v);
            if u == v {
                break;
            }
        }
        scc.push(scctem);
    }
}
