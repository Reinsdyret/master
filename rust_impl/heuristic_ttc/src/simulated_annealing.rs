use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

use num_bigint::BigUint;
use num_traits::Zero;
use rand::random_range;
use rayon::prelude::*;

use crate::{TTCState, operators::{AnyCycle, Operator}, solution::{ScoringStrategy, Solution}};

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

pub fn run_simulated_annealing_multi(
    init_solution: &Solution,
    state: &TTCState,
    operators: &[&dyn Operator],
    strategy: ScoringStrategy,
    prob: f64,
    t_final: f64,
    iterations: usize,
) -> Solution {
    let (delta_avg, mut incumbent, mut best_solution, mut incumbent_score, mut best_score) =
        estimate_avg_delta_multi(init_solution, state, operators, &strategy, prob);

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

    let output_path = "simulated_annealing_multi_scores.csv";
    let file = File::create(output_path).expect("failed to create simulated_annealing_multi_scores.csv");
    let mut writer = BufWriter::new(file);
    writeln!(writer, "iter,best_score,incumbent_score").expect("failed to write header");

    for i in 0..iterations {
        if i % 1000 == 0 {println!("i: {}", i);}
        let op_idx = random_range(0..operators.len());
        let operator = operators[op_idx];
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

pub fn run_simulated_annealing_multi_collect_cycles(
    init_solution: &Solution,
    state: &TTCState,
    operators: &[&dyn Operator],
    strategy: ScoringStrategy,
    prob: f64,
    t_final: f64,
    iterations: usize,
    max_cycles: usize,
) -> (Solution, Vec<Vec<usize>>) {
    let (delta_avg, mut incumbent, mut best_solution, mut incumbent_score, mut best_score) =
        estimate_avg_delta_multi(init_solution, state, operators, &strategy, prob);

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

    let mut seen: HashSet<String> = HashSet::new();
    let mut pool: Vec<Vec<usize>> = Vec::new();
    let empty_solution = Solution::new(vec![], state);

    fn add_cycle(
        pool: &mut Vec<Vec<usize>>,
        seen: &mut HashSet<String>,
        cycle: &[usize],
        max_cycles: usize,
    ) {
        if pool.len() >= max_cycles {
            return;
        }
        let key = cycle
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("-");
        if seen.insert(key) {
            pool.push(cycle.to_vec());
        }
    }

    for cycle in &incumbent.cycles {
        add_cycle(&mut pool, &mut seen, cycle, max_cycles);
    }

    for i in 0..iterations {
        if i % 1000 == 0 {println!("Iteration: {}, cycles: {}", i, pool.len())}
        let op_idx = random_range(0..operators.len());
        let operator = operators[op_idx];
        let new_solution = operator.apply(&incumbent, state);
        if !new_solution.verify(state) {
            temp *= alpha;
            continue;
        }

        for cycle in &new_solution.cycles {
            add_cycle(&mut pool, &mut seen, cycle, max_cycles);
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

        if pool.len() >= max_cycles {
            break;
        }
        temp *= alpha;

        if pool.len() < max_cycles && i % 50 == 0 {
            let attempts = 64usize;
            let generated: Vec<Vec<usize>> = (0..attempts)
                .into_par_iter()
                .filter_map(|_| {
                    let generator = AnyCycle;
                    let sol = generator.apply(&empty_solution, state);
                    sol.cycles.into_iter().next()
                })
                .collect();
            for cycle in generated {
                add_cycle(&mut pool, &mut seen, &cycle, max_cycles);
                if pool.len() >= max_cycles {
                    break;
                }
            }
        }
    }

    (best_solution, pool)
}

pub fn run_simulated_annealing_timed(
    init_solution: &Solution,
    state: &TTCState,
    operators: &[&dyn Operator],
    duration: Duration,
    strategy: ScoringStrategy,
    prob: f64,
    t_final: f64,
) -> Solution {
    let (delta_avg, mut incumbent, mut best_solution, mut incumbent_score, mut best_score) =
        estimate_avg_delta_multi(init_solution, state, operators, &strategy, prob);

    let t_zero = if delta_avg <= 0.0 {
        1.0
    } else {
        (-1.0 * delta_avg) / (0.8f64).ln()
    };
    let mut temp = t_zero;
    let alpha = 0.9995f64;

    let output_path = "simulated_annealing_timed_scores.csv";
    let file = File::create(output_path).expect("failed to create simulated_annealing_timed_scores.csv");
    let mut writer = BufWriter::new(file);
    writeln!(writer, "elapsed_secs,iter,best_score,incumbent_score,temp").expect("failed to write header");

    let start = Instant::now();
    let mut i: u64 = 0;
    while start.elapsed() < duration {
        i += 1;
        if i % 1000 == 0 {println!("Iteration: {}", i)}
        let op_idx = random_range(0..operators.len());
        let operator = operators[op_idx];
        let new_solution = operator.apply(&incumbent, state);
        if !new_solution.verify(state) {
            temp = (temp * alpha).max(t_final);
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
                println!("New best score: {}", best_score);
            }
        } else {
            let p: f64 = libm::exp(delta / temp);
            if rand::random::<f64>() < p {
                incumbent = new_solution;
                incumbent_score = new_score;
            }
        }

        if i % 1000 == 0 {
            writeln!(
                writer,
                "{:.2},{},{},{},{}",
                start.elapsed().as_secs_f64(),
                i,
                best_score.to_str_radix(10),
                incumbent_score.to_str_radix(10),
                temp
            )
                .expect("failed to write score row");
        }
        temp = (temp * alpha).max(t_final);
    }

    best_solution
}

fn estimate_avg_delta_multi(
    init_solution: &Solution,
    state: &TTCState,
    operators: &[&dyn Operator],
    strategy: &ScoringStrategy,
    prob: f64,
) -> (f64, Solution, Solution, BigUint, BigUint) {
    let mut incumbent = init_solution.clone();
    let mut best_solution = init_solution.clone();
    let mut incumbent_score = incumbent.score(strategy);
    let mut best_score = incumbent_score.clone();

    let mut deltas: Vec<f64> = Vec::new();

    for _ in 0..100 {
        let op_idx = random_range(0..operators.len());
        let operator = operators[op_idx];
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
