use crate::{Patient, TTCState};
/// Graph utilities for TTC algorithm
///
/// This module provides utilities for converting TTC problem instances
/// into graph representations suitable for SCC analysis.
use std::collections::HashMap;

/// Graph representation for TTC problems
#[derive(Debug, Clone)]
pub struct TTCGraph {
    /// Adjacency list: patient_id -> vec of patients they point to
    pub adjacency_list: HashMap<usize, Vec<usize>>,
    /// Reverse mapping: patient_id -> patients pointing to them
    pub reverse_adjacency: HashMap<usize, Vec<usize>>,
    /// All nodes (patient IDs) in the graph
    pub nodes: Vec<usize>,
}

impl TTCGraph {
    /// Build a TTC graph from the current state
    ///
    /// In TTC graph:
    /// - Each patient is a node
    /// - Edge from patient A to patient B exists if:
    ///   - A wants to switch
    ///   - A's preferred doctor currently has B as a patient
    ///   - B is willing to switch (or has lower priority than A)
    pub fn from_ttc_state(state: &TTCState) -> Self {
        let mut adjacency_list = HashMap::new();
        let mut reverse_adjacency = HashMap::new();
        let mut nodes = Vec::new();

        // Initialize all patients as nodes
        for patient in &state.patients {
            if patient.wants_to_switch && !patient.is_stuck {
                nodes.push(patient.id);
                adjacency_list.insert(patient.id, Vec::new());
                reverse_adjacency.insert(patient.id, Vec::new());
            }
        }

        // Build edges based on TTC rules
        for patient in &state.patients {
            if !patient.wants_to_switch || patient.is_stuck {
                continue;
            }

            // Find the doctor this patient wants to switch to
            if let Some(preferred_doctor) = state.get_doctor(patient.preferred_doctor) {
                // Add edges to all patients currently with this doctor who are willing to switch
                for target_patient in &preferred_doctor.switching_patients {
                    if target_patient.id != patient.id
                        && target_patient.wants_to_switch
                        && !target_patient.is_stuck
                    {
                        adjacency_list
                            .get_mut(&patient.id)
                            .unwrap()
                            .push(target_patient.id);

                        reverse_adjacency
                            .get_mut(&target_patient.id)
                            .unwrap()
                            .push(patient.id);
                    }
                }
            }
        }

        TTCGraph {
            adjacency_list,
            reverse_adjacency,
            nodes,
        }
    }

    /// Get all nodes in the graph
    pub fn get_nodes(&self) -> &Vec<usize> {
        &self.nodes
    }

    /// Get neighbors of a node
    pub fn get_neighbors(&self, node: usize) -> Option<&Vec<usize>> {
        self.adjacency_list.get(&node)
    }

    /// Get the adjacency list (for use with Tarjan's algorithm)
    pub fn get_adjacency_list(&self) -> &HashMap<usize, Vec<usize>> {
        &self.adjacency_list
    }

    /// Check if there's an edge between two nodes
    pub fn has_edge(&self, from: usize, to: usize) -> bool {
        self.adjacency_list
            .get(&from)
            .map_or(false, |neighbors| neighbors.contains(&to))
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        let node_count = self.nodes.len();
        let edge_count: usize = self
            .adjacency_list
            .values()
            .map(|neighbors| neighbors.len())
            .sum();

        GraphStats {
            node_count,
            edge_count,
            avg_degree: if node_count > 0 {
                edge_count as f64 / node_count as f64
            } else {
                0.0
            },
        }
    }

    /// Extract subgraph containing only the specified nodes
    pub fn subgraph(&self, nodes: &[usize]) -> TTCGraph {
        let node_set: std::collections::HashSet<usize> = nodes.iter().copied().collect();
        let mut adjacency_list = HashMap::new();
        let mut reverse_adjacency = HashMap::new();

        for &node in nodes {
            adjacency_list.insert(node, Vec::new());
            reverse_adjacency.insert(node, Vec::new());
        }

        for &node in nodes {
            if let Some(neighbors) = self.adjacency_list.get(&node) {
                for &neighbor in neighbors {
                    if node_set.contains(&neighbor) {
                        adjacency_list.get_mut(&node).unwrap().push(neighbor);
                        reverse_adjacency.get_mut(&neighbor).unwrap().push(node);
                    }
                }
            }
        }

        TTCGraph {
            adjacency_list,
            reverse_adjacency,
            nodes: nodes.to_vec(),
        }
    }
}

#[derive(Debug)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_degree: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Doctor, Patient};

    fn create_test_state() -> TTCState {
        let patients = vec![
            Patient::new(1, 1, 2, Some(1)), // Patient 1: wants doctor 2, has doctor 1
            Patient::new(2, 2, 3, Some(2)), // Patient 2: wants doctor 3, has doctor 2
            Patient::new(3, 3, 1, Some(3)), // Patient 3: wants doctor 1, has doctor 3
        ];

        let mut doctors = vec![Doctor::new(1), Doctor::new(2), Doctor::new(3)];

        // Manually populate switching patients for doctors
        doctors[0].add_switching_patient(patients[0].clone()); // Doctor 1 has Patient 1 wanting to switch
        doctors[1].add_switching_patient(patients[1].clone()); // Doctor 2 has Patient 2 wanting to switch
        doctors[2].add_switching_patient(patients[2].clone()); // Doctor 3 has Patient 3 wanting to switch

        TTCState::new(patients, doctors)
    }

    #[test]
    fn test_graph_creation() {
        let state = create_test_state();
        let graph = TTCGraph::from_ttc_state(&state);

        assert_eq!(graph.nodes.len(), 3);

        let stats = graph.stats();
        assert_eq!(stats.node_count, 3);
    }
}
