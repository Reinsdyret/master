use heuristic_ttc::{
    TTCState, local_search::run_local_search, operators::{RandomRemoveAndAddCycle, RandomRemoveOneAndRepair, RemoveAndAddCycle, RemoveAndAddCyclePLUSRandomRemoveOneAndRepair}, parse_data_file, simulated_annealing::run_simulated_annealing, solution::{ScoringStrategy, Solution}
};


fn main() {
    let files = vec![
        "data/test_150000_patient_2000_doctors_5_districts_0.3_prob.txt"
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

        let best_solution = run_simulated_annealing(&init_solution, RemoveAndAddCyclePLUSRandomRemoveOneAndRepair, &state, ScoringStrategy::ByCardinality);

        println!(
            "{} -> score: {}, valid: {}",
            file,
            best_solution.score(&ScoringStrategy::ByCardinality),
            best_solution.verify(&state),
        );

        println!(
            "{} Total cycles\n
            Average length: {}",
            best_solution.cycles.len(),
            best_solution.cycles.iter().map(|c| c.len()).sum::<usize>() as f64 / best_solution.cycles.len() as f64
        )
    }
}
