use std::collections::{HashMap, HashSet};

use good_lp::{variable, variables, Solution as LpSolution, SolverModel, default_solver};

use crate::{TTCState, solution::{ScoringStrategy, Solution}};

fn cycle_score(cycle: &[usize], state: &TTCState, strategy: &ScoringStrategy) -> usize {
    let mut seen = HashSet::new();
    let mut score = 0usize;
    for &id in cycle {
        if !seen.insert(id) {
            continue;
        }
        if let Some(patient) = state.get_patient(id) {
            if patient.is_dummy {
                continue;
            }
            match strategy {
                ScoringStrategy::ByPriority => score = score.saturating_add(patient.priority),
                ScoringStrategy::ByCardinality => score = score.saturating_add(1),
            }
        }
    }
    score
}

pub fn select_cycles_via_ilp(
    cycles: &[Vec<usize>],
    state: &TTCState,
    strategy: ScoringStrategy,
) -> Solution {
    if cycles.is_empty() {
        return Solution::new(vec![], state);
    }

    let mut vars = variables!();
    let x_vars = cycles
        .iter()
        .map(|_| vars.add(variable().binary()))
        .collect::<Vec<_>>();

    let mut objective = 0.0 * x_vars[0];
    for (idx, cycle) in cycles.iter().enumerate() {
        let weight = cycle_score(cycle, state, &strategy) as f64;
        objective = objective + weight * x_vars[idx];
    }

    let mut problem = vars.maximise(objective).using(default_solver);

    let mut patient_to_cycles: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, cycle) in cycles.iter().enumerate() {
        let mut seen = HashSet::new();
        for &id in cycle {
            if !seen.insert(id) {
                continue;
            }
            let patient = match state.get_patient(id) {
                Some(p) => p,
                None => continue,
            };
            if patient.is_dummy {
                continue;
            }
            patient_to_cycles.entry(id).or_default().push(idx);
        }
    }

    for cycle_indices in patient_to_cycles.values() {
        if cycle_indices.is_empty() {
            continue;
        }
        let mut expr = 0.0 * x_vars[cycle_indices[0]];
        for &idx in cycle_indices {
            expr = expr + x_vars[idx];
        }
        problem = problem.with(expr.leq(1));
    }

    let solution = problem.solve().expect("ILP solve failed");

    let mut selected = Vec::new();
    for (idx, cycle) in cycles.iter().enumerate() {
        if solution.value(x_vars[idx]) > 0.5 {
            selected.push(cycle.clone());
        }
    }

    Solution::new(selected, state)
}
