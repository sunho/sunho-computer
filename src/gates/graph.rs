use std::collections::{BTreeMap, BTreeSet};

pub enum GraphError {
    Cycle
}

pub struct Graph {
    nodes: BTreeMap<i64, Node>
}

struct Node {
    i: i64,
    in_edges: Vec<i64>,
    out_edges: Vec<i64>
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new()
        }
    }

    pub fn add_node(&mut self, i: i64) {
        self.nodes.insert(i, Node { 
            i: i,
            in_edges: Vec::new(),
            out_edges: Vec::new()
        });
    }

    pub fn add_edge(&mut self, si: i64, ei: i64) {
        {
            let snode = self.nodes.get_mut(&si).unwrap();
            snode.out_edges.push(ei);
        }
        {
            let enode = self.nodes.get_mut(&ei).unwrap();
            enode.in_edges.push(si);
        }
    }

    //
    // dfs: x -> y -> z -> x (cycle) 
    // y -> z -> o
    // o, z, y
    // d -> a -> b -> c
    //      d -> 
    // o -> k ->
    // 
    // k -> o
    // d
    // b -> a -> d (cycle)
    // c
    // 
    // 
    // 
    // b -> c (b,c)
    // a -> d -> c(visited) (a,d)
    // 
    // c 
    // a -> b -> c
    // 
    pub fn topological_sort_with_cycle_detection(&self) -> Result<Vec<i64>, GraphError> {
        let mut finished: BTreeSet<i64> = BTreeSet::new();
        let mut visited: BTreeSet<i64> = BTreeSet::new();
        let mut results: Vec<i64> = Vec::new();
        for i in self.nodes.keys() {
            if self.dfs(*i, &mut results, &mut visited, &mut finished) {
                return Err(GraphError::Cycle);
            }
        }
        return Ok(results.iter().rev().cloned().collect());
    }

    
    fn dfs(&self, i: i64, results: &mut Vec<i64>, visited: &mut BTreeSet<i64>, finished: &mut BTreeSet<i64>) -> bool{
        let neighbors = {
            let node = self.nodes.get(&i).unwrap();
            node.in_edges.clone()
        };
        if (visited.contains(&i)) {
            return false;
        };
        visited.insert(i);
        for x in neighbors.iter() {
            if (!visited.contains(x)) {
                if (self.dfs(*x, results, visited , finished)) {
                    return true;
                }
            } else if (!finished.contains(x)) {
                return true;
            }
        }
        results.push(i);
        finished.insert(i);
        return false;
    }
}