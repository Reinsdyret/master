use std::fs::File;
use std::io::{BufWriter, Write};
use heuristic_ttc::operators::{RandomRemoveAndAddCycle, RandomRemoveOneAndRepair};
use heuristic_ttc::{
    TTCState,
    operators::{InsertOneBetween, Operator, RemoveOneIfEdge},
    cycle_ilp::select_cycles_via_ilp,
    parse_data_file,
    simulated_annealing::run_simulated_annealing_multi,
    solution::{ScoringStrategy, Solution},
};


fn main() {
    let files = vec![
        "data/test_100000_patient_1500_doctors_0_unassigned.txt"
        // "data/test_250000_patient_5000_doctors_10_districts_0.25_prob_5000_unassigned.txt",
    ]; 

    for file in files {
        let (patients, doctors) = match parse_data_file(file) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to parse {}: {}", file, err);
                continue;
            }
        };

        let state = TTCState::new(patients, doctors);
        let init_solution = Solution::new(vec![], &state);
        // let best_solution = run_local_search(
        //     &init_solution,
        //     RandomRemoveAndAddCycle,
        //     &state,
        //     ScoringStrategy::ByCardinality,
        // );

        // TODO: teste med flyttall på pariorites scoring
        // TODO: Teste med bare sammenligne strenger i stedet for å gjøre om til bigint
        // TODO: Kanskje bare sortere 10 og 10 om gangen og sammenligne, kanskje vi ikke trenger å sortere hele priority listen
        // Halveis quicksort
        
        // TODO: 

        let op1 = InsertOneBetween;
        let op2 = RemoveOneIfEdge;
        let op3 = RandomRemoveAndAddCycle;
        let op4 = RandomRemoveOneAndRepair;
        let operators: [&dyn Operator; 4] = [&op1, &op2, &op3, &op4];

        let best_solution = run_simulated_annealing_multi(
            &init_solution,
            &state,
            &operators,
            ScoringStrategy::ByCardinality,
            0.9,
            1e-3,
            10_000,
        );

        println!(
            "{} -> SA score: {}, valid: {}",
            file,
            best_solution.score(&ScoringStrategy::ByCardinality),
            best_solution.verify(&state),
        );

        println!(
            "{} SA cycles\n
            Average length: {}",
            best_solution.cycles.len(),
            best_solution.cycles.iter().map(|c| c.len()).sum::<usize>() as f64 / best_solution.cycles.len() as f64
        );

        // let output_path = format!("{}_sa_switched_priorities.txt", file.replace("/", "_"));
        // let file = File::create(&output_path).expect("failed to create priorities file");
        // let mut writer = BufWriter::new(file);
        // for priority in &best_solution.repr {
        //     writeln!(writer, "{}", priority).expect("failed to write priority");
        // }
        // println!("Wrote SA priorities to {}", output_path);

        // let output_path = format!("{}_ilp_switched_priorities.txt", file.replace("/", "_"));
        // let file = File::create(&output_path).expect("failed to create priorities file");
        // let mut writer = BufWriter::new(file);
        // for priority in &ilp_solution.repr {
        //     writeln!(writer, "{}", priority).expect("failed to write priority");
        // }
        // println!("Wrote ILP priorities to {}", output_path);
    }
}
