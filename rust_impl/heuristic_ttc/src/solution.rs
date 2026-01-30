use num_bigint::BigUint;

use crate::TTCState;


#[derive(Copy, Clone)]
pub enum ScoringStrategy {
    ByPriority,
    ByCardinality
}

#[derive(Clone)]
pub struct Solution {
    pub repr: Vec<usize>,
    pub cycles: Vec<Vec<usize>>
}

impl Solution {
    pub fn new(cycles: Vec<Vec<usize>>, state: &TTCState) -> Self {
        let unique_nodes: Vec<usize> = cycles.clone().into_iter().flat_map(|vec| vec.into_iter()).collect();
        let mut unique_non_dummy: Vec<usize> = unique_nodes.into_iter().filter(|p| {
            if let Some(patient) = state.get_patient(*p) {
                !patient.is_dummy
            } else {
                false
            }
        }).map(| id| {
            let patient = state.get_patient(id).unwrap();
            patient.priority
        }).collect();

        unique_non_dummy.sort();
        unique_non_dummy.reverse();

        Self { repr: unique_non_dummy, cycles }
    }

    fn verify_cycle(&self, cycle: &Vec<usize>, state: &TTCState) -> bool {
        if cycle[0] != cycle[cycle.len()-1] {return false}; // Not a cycle
        
        for (i, node) in cycle[0..cycle.len()-1].iter().enumerate() {
            let patient = state.get_patient(*node).unwrap();
            let next_patient = state.get_patient(cycle[i+1]).unwrap();

            if patient.preferred_doctor != next_patient.current_doctor.unwrap() {return false} // No edge between them
        }

        return true;
    }

    pub fn verify(&self, state: &TTCState) -> bool {
        self.cycles.iter().map(|c| self.verify_cycle(c, &state)).all(|t| t)
    }

    fn score_by_priority(&self) -> BigUint {
        if self.repr.is_empty() {return BigUint::from(0u8);}
        let score_str: String = self.repr.iter()
            .map(|x| x.to_string())
            .collect();
        BigUint::parse_bytes(score_str.as_bytes(), 10)
            .unwrap_or_else(|| BigUint::from(0u8))
    }

    fn score_by_size(&self) -> BigUint {
        BigUint::from(self.repr.len())
    }

    pub fn score(&self, strategy: &ScoringStrategy) -> BigUint {
        match strategy {
            ScoringStrategy::ByPriority => self.score_by_priority(),
            ScoringStrategy::ByCardinality => self.score_by_size(),
        }
    }

    
}