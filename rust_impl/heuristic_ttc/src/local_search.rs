use std::fs::File;
use std::io::{BufWriter, Write};

use crate::{TTCState, operators::Operator, solution::{ScoringStrategy, Solution}};


pub fn run_local_search<T: Operator>(init_solution: &Solution, operator: T, state: &TTCState, strategy: ScoringStrategy) -> Solution {    
    let mut best_solution = init_solution.clone();
    let mut best_score = best_solution.score(&strategy);

    let output_path = "local_search_scores.csv";
    let file = File::create(output_path).expect("failed to create local_search_scores.csv");
    let mut writer = BufWriter::new(file);
    writeln!(writer, "iter,best_score").expect("failed to write header");

    for i in 0..10_000 {
        if i % 1000 == 0 {println!("{}", i)}
        let new_solution = operator.apply(&best_solution, &state);

        if !new_solution.verify(state) {continue;}

        let score = new_solution.score(&strategy);

        if score > best_score {
            best_solution = new_solution;
            best_score = score;
        }

        writeln!(writer, "{},{}", i, best_score).expect("failed to write score row");
    }

    best_solution
}