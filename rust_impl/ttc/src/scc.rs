/// Kosaraju's Strongly Connected Components Algorithm
/// 
/// This module implements Kosaraju's algorithm for finding strongly connected components
/// in a directed graph. Simpler and often faster than Tarjan's for TTC problems.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct KosajaruSCC {
    
}

impl KosajaruSCC {
    pub fn new() -> Self {
        KosajaruSCC {
            
        }
    }

    /// Find all strongly connected components in the graph
    /// 
    /// # Arguments
    /// * `graph` - Adjacency list representation: node_id -> Vec<neighbor_ids>
    /// 
    /// # Returns
    /// Vector of SCCs, where each SCC is a vector of node IDs
    pub fn find_sccs(&mut self, graph: &HashMap<usize, Vec<usize>>) -> Vec<Vec<usize>> {
        // Step 1: First DFS to get finish order
        let mut visited = HashSet::with_capacity(graph.len());
        let mut finish_order = Vec::with_capacity(graph.len());
        
        for &node in graph.keys() {
            if !visited.contains(&node) {
                self.dfs_first_pass(node, graph, &mut visited, &mut finish_order);
            }
        }
        
        // Step 2: Create transpose graph
        let transpose = self.transpose_graph(graph);
        
        // Step 3: Second DFS on transpose in reverse finish order
        let mut visited = HashSet::with_capacity(graph.len());
        let mut sccs = Vec::new();
        
        while let Some(node) = finish_order.pop() {
            if !visited.contains(&node) {
                let mut scc = Vec::new();
                self.dfs_second_pass(node, &transpose, &mut visited, &mut scc);
                sccs.push(scc);
            }
        }
        
        sccs
    }
    
    fn dfs_first_pass(
        &self,
        node: usize,
        graph: &HashMap<usize, Vec<usize>>,
        visited: &mut HashSet<usize>,
        finish_order: &mut Vec<usize>,
    ) {
        visited.insert(node);
        
        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    self.dfs_first_pass(neighbor, graph, visited, finish_order);
                }
            }
        }
        
        finish_order.push(node);
    }
    
    fn transpose_graph(&self, graph: &HashMap<usize, Vec<usize>>) -> HashMap<usize, Vec<usize>> {
        let mut transpose = HashMap::with_capacity(graph.len());
        
        // Initialize empty adjacency lists for all nodes
        for &node in graph.keys() {
            transpose.insert(node, Vec::new());
        }
        
        // Reverse all edges, but only add to nodes that exist in original graph
        for (&from, neighbors) in graph {
            for &to in neighbors {
                if graph.contains_key(&to) {
                    transpose.get_mut(&to).unwrap().push(from);
                }
            }
        }
        
        transpose
    }
    
    fn dfs_second_pass(
        &self,
        node: usize,
        transpose: &HashMap<usize, Vec<usize>>,
        visited: &mut HashSet<usize>,
        scc: &mut Vec<usize>,
    ) {
        visited.insert(node);
        scc.push(node);
        
        if let Some(neighbors) = transpose.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    self.dfs_second_pass(neighbor, transpose, visited, scc);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cycle() {
        let mut kosaraju = KosajaruSCC::new();
        let mut graph = HashMap::new();
        
        // Simple 3-node cycle: 1 -> 2 -> 3 -> 1
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1]);
        
        let sccs = kosaraju.find_sccs(&graph);
        
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 3);
        
        let mut scc_sorted = sccs[0].clone();
        scc_sorted.sort();
        assert_eq!(scc_sorted, vec![1, 2, 3]);
    }

    #[test]
    fn test_no_cycles() {
        let mut kosaraju = KosajaruSCC::new();
        let mut graph = HashMap::new();
        
        // Linear chain: 1 -> 2 -> 3
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![]);
        
        let sccs = kosaraju.find_sccs(&graph);
        
        // Each node should be its own SCC
        assert_eq!(sccs.len(), 3);
        for scc in sccs {
            assert_eq!(scc.len(), 1);
        }
    }
}