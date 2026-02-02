use std::collections::HashSet;

use num_traits::zero;
use rand::{random_bool, random_range, seq::{IteratorRandom, index::sample}};

use crate::{Doctor, TTCState, solution::Solution};


pub trait Operator {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution;
}

pub struct RemoveCycle;
pub struct AddCycle;
pub struct RemoveAndAddCycle;

pub struct RandomRemoveOneAndRepair; // Try to remove on node from a cycle and expand that cycle making it bigger.

pub struct RemoveAndAddCyclePLUSRandomRemoveOneAndRepair;

// Search for cycle with nodes not used, if find any overlap with existing cycle, ban that node and get new cycle try to fix old cycle.

pub struct RandomRemoveAndAddCycle;

impl Operator for RemoveCycle {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution {
        if solution.cycles.is_empty() {return Solution::new(vec![], state)}

        let mut new_cycles = solution.cycles.clone();

        let idx = random_range(0..solution.cycles.len());
        
        new_cycles.remove(idx);

        Solution::new(new_cycles, state)
    }
}

impl Operator for AddCycle {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution {
        let mut rng = rand::rng();
        // First get a set of all used patients from solution
        let unique_nodes: HashSet<usize> = solution.cycles.clone().into_iter().flat_map(|vec| vec.into_iter()).collect();
        let nodes: HashSet<usize> = HashSet::from_iter(state.patients.iter().map(|p| &p.id).cloned());

        let not_used_nodes: HashSet<usize> = nodes.difference(&unique_nodes).cloned().collect();

        // Choose a random non used patient to start cycle from
        let candidate_starts: Vec<usize> = not_used_nodes.iter()
            .filter(|id| state.get_patient(**id).map(|p| !p.is_dummy).unwrap_or(false))
            .cloned()
            .collect();

        let patient_id = match candidate_starts.iter().choose(&mut rng) {
            Some(id) => *id,
            None => return solution.clone(),
        };

        // Find cycle with DFS
        fn dfs(
            state: &TTCState,
            not_used_nodes: &HashSet<usize>,
            start: usize,
            current: usize,
            visited: &mut HashSet<usize>,
            path: &mut Vec<usize>,
            steps_left: &mut usize,
        ) -> bool {
            if *steps_left == 0 {
                return false;
            }
            *steps_left -= 1;

            let preferred_doctor = match state.get_patient(current) {
                Some(patient) => patient.preferred_doctor,
                None => return false,
            };

            let doctor = match state.get_doctor(preferred_doctor) {
                Some(doctor) => doctor,
                None => return false,
            };

            for next_patient in doctor
                .switching_patients
                .iter()
                .map(|p| p.id)
            {
                if *steps_left == 0 {
                    return false;
                }
                if next_patient == start {
                    path.push(next_patient);
                    return true;
                }

                if !not_used_nodes.contains(&next_patient) || visited.contains(&next_patient) {
                    continue;
                }

                visited.insert(next_patient);
                path.push(next_patient);

                if dfs(state, not_used_nodes, start, next_patient, visited, path, steps_left) {
                    return true;
                }

                path.pop();
                visited.remove(&next_patient);
            }

            false
        }

        let mut path = vec![patient_id];
        let mut visited: HashSet<usize> = HashSet::from([patient_id]);
        let mut steps_left = std::cmp::min(
            not_used_nodes.len().saturating_mul(5).max(1),
            1_000_000,
        );
        if dfs(
            state,
            &not_used_nodes,
            patient_id,
            patient_id,
            &mut visited,
            &mut path,
            &mut steps_left,
        ) {
            let mut new_cycles = solution.cycles.clone();
            new_cycles.push(path);
            return Solution::new(new_cycles, state);
        }

        solution.clone()

    }
}

impl Operator for RemoveAndAddCycle {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution {
        let removed = RemoveCycle.apply(solution, state);
        AddCycle.apply(&removed, state)
    }
}

impl Operator for RandomRemoveAndAddCycle {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution {
        if random_bool(0.1) == true {
            let removed = RemoveCycle.apply(solution, state);
            AddCycle.apply(&removed, state)
        } else {
            AddCycle.apply(&solution, state)
        }
    }
}

impl Operator for RandomRemoveOneAndRepair {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution {
        if solution.cycles.len() == 0 {
            return solution.clone();
        }
        // Choose random cycle
        let cycle_idx = random_range(0..solution.cycles.len());
        let mut cycle = solution.cycles[cycle_idx].clone();
        // Choose random node
        let index = random_range(0..cycle.len());
        cycle.remove(index);

        let prev_index = if index == 0 { cycle.len() - 1 } else { index - 1 };
        let next_index = index % cycle.len();

        // Find extended cycle with dfs
        let unique_nodes: HashSet<usize> = solution.cycles.clone().into_iter().flat_map(|vec| vec.into_iter()).collect();
        let nodes: HashSet<usize> = HashSet::from_iter(state.patients.iter().map(|p| &p.id).cloned());

        let not_used_nodes: HashSet<usize> = nodes.difference(&unique_nodes).cloned().collect();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        if dfs(cycle[prev_index], cycle[next_index], state, &not_used_nodes, &mut visited, &mut path) {
            // println!("WORKED!: Extended cycle with length: {}", path.len() - 2);

            assert!(path[0] == cycle[prev_index]);

            path.remove(0);
            // Extend the cycle with this path
            cycle.splice(next_index..next_index, path.into_iter());

            let mut new_solution = solution.clone();
            new_solution.cycles[cycle_idx] = cycle;

            return new_solution;
        }
        
        return solution.clone();
    }
}

impl Operator for RemoveAndAddCyclePLUSRandomRemoveOneAndRepair {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution {
        if random_bool(0.5) {
            RandomRemoveAndAddCycle.apply(solution, state)
        } else {
            RandomRemoveOneAndRepair.apply(solution, state)
        }
    }
}




fn dfs(current: usize, goal: usize, state: &TTCState, not_used: &HashSet<usize>, visited: &mut HashSet<usize>, path: &mut Vec<usize>) -> bool {

    if path.len() > 1 && current == goal {
        return true;
    }

    if visited.contains(&current) {
        // println!("Found another cycle");
        return false;
    }

    // Allow start/goal even though they are "used" already
    if current != goal && !not_used.contains(&current) {
        // println!("Visited non legal");
        return false;
    }

    let doctor_id = match state.get_patient(current) {
        Some(p) => p.preferred_doctor,
        None => return false,
    };

    let doctor = match state.get_doctor(doctor_id) {
        Some(d) => d,
        None => return false,
    };

    visited.insert(current);
    path.push(current);

    for next_id in doctor.switching_patients.iter().map(|p| p.id) {
        if (not_used.contains(&next_id) || next_id == goal) && dfs(next_id, goal, state, not_used, visited, path) {
            return true;
        }
    }

    path.pop();
    visited.remove(&current);

    // println!("Ran out of neighbors");

    return false;
}