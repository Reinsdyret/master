use std::collections::HashSet;

use crate::TTCState;

pub struct Solution {
    repr: Vec<usize>,
    cycles: Vec<Vec<usize>>
}

impl Solution {
    pub fn new(cycles: Vec<Vec<usize>>, state: TTCState) -> Self {
        let unique_nodes: Vec<usize> = cycles.clone().into_iter().flat_map(|vec| vec.into_iter()).collect();
        let mut unique_non_dummy: Vec<usize> = unique_nodes.into_iter().filter(|p| {
            if let Some(patient) = state.get_patient(*p) {
                !patient.is_dummy
            } else {
                false
            }
        }).collect();

        unique_non_dummy.sort_by(|a, b| {
            let p_a = state.get_patient(*a).unwrap();
            let p_b = state.get_patient(*b).unwrap();
            p_a.priority.cmp(&p_b.priority)
        });

        Self { repr: unique_non_dummy, cycles }
    }

    pub fn verify(&self, state: TTCState) -> bool {
        
    }
}