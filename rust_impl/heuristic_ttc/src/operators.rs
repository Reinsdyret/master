use std::collections::HashSet;

use rand::{random_bool, random_range, seq::IteratorRandom};

use crate::{TTCState, solution::Solution};


pub trait Operator {
    fn apply(&self, solution: &Solution, state: &TTCState) -> Solution;
}

pub struct RemoveCycle;
pub struct AddCycle;
pub struct RemoveAndAddCycle;

pub struct RemoveOneNodeFromOneCycle; // Try to remove on node from a cycle and expand that cycle making it bigger.

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