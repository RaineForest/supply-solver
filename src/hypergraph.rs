use indexmap::{IndexSet, IndexMap};
use std::collections::{HashSet, BTreeSet};
use std::hash::Hash;

pub type EdgeIndex = usize;
pub type NodeIndex = usize;

struct Hyperedge<E> {
    src: BTreeSet<NodeIndex>,
    dst: BTreeSet<NodeIndex>,
    weight: E
}

impl<E> PartialEq for Hyperedge<E> {
    fn eq(&self, rhs: &Self) -> bool {
        self.src == rhs.src && self.dst == rhs.dst
    }
}
impl<E> Eq for Hyperedge<E> {}

impl<E> Hash for Hyperedge<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.src.hash(state);
        self.dst.hash(state);
    }
}

struct Hypernode {
    neighbors: HashSet<NodeIndex>,
    neighbor_of: HashSet<NodeIndex>
}

impl Hypernode {
    pub fn new() -> Self {
        Hypernode { neighbors: HashSet::new(), neighbor_of: HashSet::new() }
    }
}

pub struct Hypergraph<N, E>
where N: Copy + Hash + Eq, E: Hash + Eq {
    nodes: IndexMap<N, Hypernode>,
    edges: IndexSet<Hyperedge<E>>
}

impl<N, E> Hypergraph<N, E>
where N: Copy + Hash + Eq, E: Hash + Eq {
    pub fn new() -> Self {
        Self { nodes: IndexMap::new(), edges: IndexSet::new() }
    }

    pub fn insert_node(&mut self, node: N) -> NodeIndex {
        let (index, _) = self.nodes.insert_full(node, Hypernode::new());
        index
    }

    pub fn insert_edge(&mut self, sources: &[N], destinations: &[N], weight: E) -> EdgeIndex {
        let mapping = | node: &N | -> usize { self.nodes.get_index_of(node).unwrap() };
        let (index, _) = self.edges.insert_full(
            Hyperedge::<E> {
                src: sources.iter().map(mapping).collect(),
                dst: destinations.iter().map(mapping).collect(),
                weight
            }
        );
        for src in sources {
            self.nodes.get_mut(src).unwrap().neighbors.insert(index);
        }
        for dst in destinations {
            self.nodes.get_mut(dst).unwrap().neighbor_of.insert(index);
        }
        index
    }

    pub fn order(&self) -> usize {
        self.nodes.len()
    }

    pub fn size(&self) -> usize {
        self.edges.len()
    }

    pub fn neighbors(&self, node: &N) -> Result<Vec<&EdgeIndex>, &str> {
        self.nodes.get(node).ok_or_else(|| "Node does not exist").map(| n | n.neighbors.iter().collect())
    }

    pub fn neighbor_of(&self, node: &N) -> Result<Vec<&EdgeIndex>, &str> {
        self.nodes.get(node).ok_or_else(|| "Node does not exist").map(| n | n.neighbor_of.iter().collect())
    }

    pub fn get_node(&self, n: &NodeIndex) -> Result<&N, &str> {
        self.nodes.get_index(n.clone()).ok_or_else(|| "Node does not exist").map(| (k, _) | k)
    }

    pub fn get_weight(&self, e: &EdgeIndex) -> Result<&E, &str> {
        self.edges.get_index(e.clone()).ok_or_else(|| "Edge does not exist").map(| e | &e.weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_basic_graph() -> Hypergraph<u32, u32> {
        let mut graph = Hypergraph::<u32, u32>::new();
        graph.insert_node(1);
        graph.insert_node(2);
        graph.insert_node(3);
        graph.insert_node(4);
        graph.insert_edge(&vec![1, 2], &vec![3, 4], 15);
        graph.insert_edge(&vec![3], &vec![1], 30);
        graph.insert_edge(&vec![4], &vec![2], 45);
        graph
    }

    #[test]
    fn order_test() {
        let graph = build_basic_graph();
        assert_eq!(graph.order(), 4);
    }

    #[test]
    fn size_test() {
        let graph = build_basic_graph();
        assert_eq!(graph.size(), 3);
    }

    #[test]
    fn neighbor_test() {
        let graph = build_basic_graph();
        let neighbors = graph.neighbors(&1u32);
        let neighbor_of = graph.neighbor_of(&1u32);
        assert_eq!(neighbors, Ok(vec![&0usize]));
        assert_eq!(neighbor_of, Ok(vec![&1usize]));
        assert_eq!(graph.get_weight(neighbors.unwrap()[0]), Ok(&15));
        assert_eq!(graph.get_weight(neighbor_of.unwrap()[0]), Ok(&30));
    }
}
