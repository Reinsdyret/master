use std::fs::File;
use std::io::{BufWriter, Write};

use num_bigint::BigUint;
use num_traits::Zero;

use crate::{TTCState, operators::Operator, solution::{ScoringStrategy, Solution}};

pub fn run_simulated_annealing<T: Operator>(
    init_solution: &Solution,
    operator: T,
    state: &TTCState,
    strategy: ScoringStrategy,
) -> Solution {
    run_simulated_annealing_with_params(init_solution, operator, state, strategy, 0.9, 1e-3, 100_000)
}

pub fn run_simulated_annealing_with_params<T: Operator>(
    init_solution: &Solution,
    operator: T,
    state: &TTCState,
    strategy: ScoringStrategy,
    prob: f64,
    t_final: f64,
    iterations: usize,
) -> Solution {
    let (delta_avg, mut incumbent, mut best_solution, mut incumbent_score, mut best_score) =
        estimate_avg_delta(init_solution, &operator, state, &strategy, prob);

    let t_zero = if delta_avg <= 0.0 {
        1.0
    } else {
        (-1.0 * delta_avg) / (0.8f64).ln()
    };
    let alpha = if iterations <= 1 {
        1.0
    } else {
        f64::powf(t_final / t_zero, 1.0 / (iterations as f64 - 1.0))
    };
    let mut temp = t_zero;

    let output_path = "simulated_annealing_scores.csv";
    let file = File::create(output_path).expect("failed to create simulated_annealing_scores.csv");
    let mut writer = BufWriter::new(file);
    writeln!(writer, "iter,best_score,incumbent_score").expect("failed to write header");

    for i in 0..iterations {
        if i % 1000 == 0 {println!("{}", i)}
        let new_solution = operator.apply(&incumbent, state);
        if !new_solution.verify(state) {
            temp *= alpha;
            writeln!(
                writer,
                "{},{},{}",
                i,
                best_score.to_str_radix(10),
                incumbent_score.to_str_radix(10)
            )
                .expect("failed to write score row");
            continue;
        }

        let new_score = new_solution.score(&strategy);
        let delta = score_delta_log10(&new_score, &incumbent_score);

        if delta > 0.0 {
            incumbent = new_solution;
            incumbent_score = new_score.clone();
            if new_score > best_score {
                best_solution = incumbent.clone();
                best_score = new_score;
            }
        } else {
            let p: f64 = libm::exp(delta / temp);
            if rand::random::<f64>() < p {
                incumbent = new_solution;
                incumbent_score = new_score;
            }
        }

        writeln!(
            writer,
            "{},{},{}",
            i,
            best_score.to_str_radix(10),
            incumbent_score.to_str_radix(10)
        )
            .expect("failed to write score row");
        temp *= alpha;
    }

    best_solution
}

fn estimate_avg_delta<T: Operator>(
    init_solution: &Solution,
    operator: &T,
    state: &TTCState,
    strategy: &ScoringStrategy,
    prob: f64,
) -> (f64, Solution, Solution, BigUint, BigUint) {
    let mut incumbent = init_solution.clone();
    let mut best_solution = init_solution.clone();
    let mut incumbent_score = incumbent.score(strategy);
    let mut best_score = incumbent_score.clone();

    let mut deltas: Vec<f64> = Vec::new();

    for _ in 0..100 {
        let new_solution = operator.apply(&incumbent, state);
        if !new_solution.verify(state) {
            continue;
        }

        let new_score = new_solution.score(strategy);
        let delta = score_delta_log10(&new_score, &incumbent_score);

        if delta > 0.0 {
            incumbent = new_solution;
            incumbent_score = new_score.clone();
            if new_score > best_score {
                best_solution = incumbent.clone();
                best_score = new_score;
            }
        } else {
            if rand::random::<f64>() < prob {
                incumbent = new_solution;
                incumbent_score = new_score;
            }
            deltas.push(-delta);
        }
    }

    let avg_delta = if deltas.is_empty() {
        0.1
    } else {
        deltas.iter().sum::<f64>() / deltas.len() as f64
    };

    (avg_delta, incumbent, best_solution, incumbent_score, best_score)
}

fn score_delta_log10(new_score: &BigUint, old_score: &BigUint) -> f64 {
    score_log10(new_score) - score_log10(old_score)
}

fn score_log10(score: &BigUint) -> f64 {
    if score.is_zero() {
        return 0.0;
    }
    let score_str = score.to_str_radix(10);
    let len = score_str.len();
    let prefix_len = 15.min(len);
    let prefix_str = &score_str[..prefix_len];
    let prefix = prefix_str.parse::<f64>().unwrap_or(0.0);
    if prefix == 0.0 {
        return 0.0;
    }
    (len - prefix_len) as f64 + prefix.log10()
}
