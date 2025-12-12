
#![allow(dead_code)]

use node_index::NodeIndex;
use edge_index::EdgeIndex;
use node_hash::NodeHash;

pub fn solve(input: &str) -> usize {
    let graph = Graph::from_input(input.as_bytes());

    let start = graph.get_by_hash(NodeHash::from_slice(b"you"));
    let end = graph.get_by_hash(NodeHash::from_slice(b"out"));

    graph.find_paths(start, end)
}

struct Graph {
    nodes: Nodes,
    edges: Edges,
    hash_to_index: NodeIndexLookupTable,
}

struct Nodes {
    nodes: Vec<Node>,
}

impl Nodes {
    fn get(&self, index: NodeIndex) -> &Node {
        &self.nodes[index.into_inner() as usize]
    }
}

struct Edges {
    edges: Vec<NodeIndex>,
}

impl Edges {
    fn get(&self, node: &Node) -> &[NodeIndex] {
        &self.edges[node.start_edge.into_inner()..node.end_edge.into_inner()]
    }
}

impl Graph {
    fn from_input(input: &[u8]) -> Self {
        // Count lines and edges in one pass
        let (num_lines, num_edges) = input.iter().fold((0, 0), |(lines, spaces), &b| {
            (lines + (b == b'\n') as usize, spaces + (b == b' ') as usize)
        });

        let mut nodes = Vec::with_capacity(num_lines + 1); // +1 for "out" node
        let mut edges = Vec::with_capacity(num_edges);
        let mut hash_to_index = NodeIndexLookupTableBuilder::new();

        // 1st pass register all node names (just the first 3 chars per line)
        let mut node_idx = 0u16;
        for line in input.split(|&c| c == b'\n').filter(|l| l.len() >= 3) {
            let hash = NodeHash::from_slice(&line[0..3]);
            hash_to_index.insert(hash, NodeIndex::new(node_idx));
            node_idx += 1;
        }

        // add out node cos it doesnt appear in first three bytes
        let out_hash = NodeHash::from_slice(b"out");
        let out_index = NodeIndex::new(node_idx);
        hash_to_index.insert(out_hash, out_index);

        // 2nd pass parse edges and build nodes
        for line in input.split(|&c| c == b'\n').filter(|l| l.len() >= 3) {
            let hash = NodeHash::from_slice(&line[0..3]);
            let start_edge = EdgeIndex::new(edges.len());

            let mut idx = 5;
            while idx + 3 <= line.len() {
                let edge_hash = NodeHash::from_slice(&line[idx..idx + 3]);
                let target_idx = hash_to_index.get(edge_hash);
                edges.push(target_idx);
                idx += 4;
            }

            nodes.push(Node {
                hash,
                start_edge,
                end_edge: EdgeIndex::new(edges.len()),
            });
        }

        // add "out" node with no outgoing edges
        nodes.push(Node {
            hash: out_hash,
            start_edge: EdgeIndex::new(edges.len()),
            end_edge: EdgeIndex::new(edges.len()),
        });

        Graph {
            nodes: Nodes { nodes },
            edges: Edges { edges },
            hash_to_index: hash_to_index.build(),
        }
    }

    fn find_paths(&self, start: &Node, end: &Node) -> usize {
        if start == end {
            return 1;
        }

        let mut path_count = 0;
        for &neighbor_idx in self.edges.get(start) {
            let neighbor = self.nodes.get(neighbor_idx);
            path_count += self.find_paths(neighbor, end);
        }
        path_count
    }

    fn get_by_hash(&self, hash: NodeHash) -> &Node {
        let index = self.hash_to_index.get(hash);
        self.nodes.get(index)
    }
}

mod node_index {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct NodeIndex(u16);

    impl NodeIndex {
        pub const fn new(value: u16) -> Self {
            NodeIndex(value)
        }

        #[inline]
        pub const fn into_inner(self) -> u16 {
            self.0
        }
    }
}

mod edge_index {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct EdgeIndex(usize);

    impl EdgeIndex {
        pub const fn new(value: usize) -> Self {
            EdgeIndex(value)
        }

        #[inline]
        pub const fn into_inner(self) -> usize {
            self.0
        }
    }
}

mod node_hash {
    const POWERS_OF_26: [u16; 3] = [26 * 26, 26, 1];

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct NodeHash(u16);

    impl NodeHash {
        pub const INVALID: Self = NodeHash(0);

        #[inline]
        const fn get_base26_offset(ascii: u8, position: u8) -> u16 {
            (ascii - b'a') as u16 * POWERS_OF_26[position as usize] + 1
        }

        #[inline]
        pub fn is_valid(&self) -> bool {
            self.0 != Self::INVALID.0
        }

        #[inline]
        pub const fn into_inner(self) -> u16 {
            self.0
        }

        pub fn from_slice(s: &[u8]) -> Self {
            debug_assert!(s.len() == 3, "Invalid node name length: {}", std::str::from_utf8(s).unwrap());
            debug_assert!(s.iter().all(|&b| b.is_ascii_lowercase()), "Invalid node name: {}", std::str::from_utf8(s).unwrap());

            let left_most = Self::get_base26_offset(s[0], 0);
            let middle = Self::get_base26_offset(s[1], 1);
            let right_most = Self::get_base26_offset(s[2], 2);
            NodeHash(left_most + middle + right_most)
        }
    }
}

struct NodeIndexLookupTable {
    table: [NodeIndex; 26 * 26 * 26],
}

impl NodeIndexLookupTable {
    fn get(&self, hash: NodeHash) -> NodeIndex {
        debug_assert!(hash.is_valid(), "Cannot get invalid node hash");
        self.table[(hash.into_inner() - 1) as usize]
    }
}

struct NodeIndexLookupTableBuilder {
    table: [NodeIndex; 26 * 26 * 26],
}

impl NodeIndexLookupTableBuilder {
    fn new() -> Self {
        NodeIndexLookupTableBuilder {
            table: [NodeIndex::new(0); 26 * 26 * 26],
        }
    }

    fn insert(&mut self, hash: NodeHash, index: NodeIndex) {
        debug_assert!(hash.is_valid(), "Cannot insert invalid node hash");
        self.table[(hash.into_inner() - 1) as usize] = index;
    }

    fn get(&self, hash: NodeHash) -> NodeIndex {
        self.table[(hash.into_inner() - 1) as usize]
    }

    fn build(self) -> NodeIndexLookupTable {
        NodeIndexLookupTable { table: self.table }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    hash: NodeHash,
    start_edge: EdgeIndex,
    end_edge: EdgeIndex
}

impl Node {
    fn new(hash: NodeHash) -> Self {
        Node {
            hash,
            start_edge: EdgeIndex::new(0),
            end_edge: EdgeIndex::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const EXAMPLE: &str = indoc::indoc! {
        "aaa: you hhh
         you: bbb ccc
         bbb: ddd eee
         ccc: ddd eee fff
         ddd: ggg
         eee: out
         fff: out
         ggg: out
         hhh: ccc fff iii
         iii: out"
    };

    #[test]
    fn solve_example() {
        let input = EXAMPLE;
        let result = super::solve(input);
        assert_eq!(result, 5);
    }

    mod node_index {
        use super::*;

        #[rstest]
        #[case(0)]
        #[case(42)]
        #[case(u16::MAX)]
        fn new_and_into_inner(#[case] value: u16) {
            assert_eq!(NodeIndex::new(value).into_inner(), value);
        }
    }

    mod edge_index {
        use super::*;

        #[rstest]
        #[case(0)]
        #[case(100)]
        #[case(usize::MAX)]
        fn new_and_into_inner(#[case] value: usize) {
            assert_eq!(EdgeIndex::new(value).into_inner(), value);
        }
    }

    mod node_hash {
        use super::*;

        #[rstest]
        #[case(b"aaa", 3)]
        #[case(b"aab", 4)]
        #[case(b"aba", 3 + 26)]
        #[case(b"baa", 3 + 26 * 26)]
        fn from_slice(#[case] input: &[u8], #[case] expected: u16) {
            assert_eq!(NodeHash::from_slice(input).into_inner(), expected);
        }

        #[rstest]
        #[case(b"abc", true)]
        #[case(b"zzz", true)]
        fn is_valid(#[case] input: &[u8], #[case] expected: bool) {
            assert_eq!(NodeHash::from_slice(input).is_valid(), expected);
        }

        #[test]
        fn invalid_is_not_valid() {
            assert!(!NodeHash::INVALID.is_valid());
        }

        #[rstest]
        #[case(b"abc", b"abc")]
        #[case(b"xyz", b"xyz")]
        fn unique_hashes(#[case] a: &[u8], #[case] b: &[u8]) {
            let ha = NodeHash::from_slice(a);
            let hb = NodeHash::from_slice(b);
            assert_eq!(ha == hb, a == b);
        }
    }

    mod lookup_table {
        use super::*;

        #[test]
        fn insert_and_get() {
            let mut builder = NodeIndexLookupTableBuilder::new();
            let hash = NodeHash::from_slice(b"abc");
            builder.insert(hash, NodeIndex::new(42));
            assert_eq!(builder.get(hash).into_inner(), 42);
        }

        #[test]
        fn build_preserves_entries() {
            let mut builder = NodeIndexLookupTableBuilder::new();
            let hash = NodeHash::from_slice(b"xyz");
            builder.insert(hash, NodeIndex::new(99));
            let table = builder.build();
            assert_eq!(table.get(hash).into_inner(), 99);
        }

        #[rstest]
        #[case(b"aaa", 0)]
        #[case(b"bbb", 1)]
        #[case(b"ccc", 2)]
        fn multiple_entries(#[case] name: &[u8], #[case] idx: u16) {
            let mut builder = NodeIndexLookupTableBuilder::new();
            builder.insert(NodeHash::from_slice(b"aaa"), NodeIndex::new(0));
            builder.insert(NodeHash::from_slice(b"bbb"), NodeIndex::new(1));
            builder.insert(NodeHash::from_slice(b"ccc"), NodeIndex::new(2));
            let table = builder.build();
            assert_eq!(table.get(NodeHash::from_slice(name)).into_inner(), idx);
        }
    }

    mod nodes {
        use super::*;

        #[test]
        fn get_by_index() {
            let nodes = Nodes {
                nodes: vec![
                    Node::new(NodeHash::from_slice(b"abc")),
                    Node::new(NodeHash::from_slice(b"def")),
                ],
            };
            assert_eq!(nodes.get(NodeIndex::new(0)).hash, NodeHash::from_slice(b"abc"));
            assert_eq!(nodes.get(NodeIndex::new(1)).hash, NodeHash::from_slice(b"def"));
        }
    }

    mod edges {
        use super::*;

        #[test]
        fn get_returns_slice() {
            let edges = Edges {
                edges: vec![NodeIndex::new(1), NodeIndex::new(2), NodeIndex::new(3)],
            };
            let node = Node {
                hash: NodeHash::from_slice(b"abc"),
                start_edge: EdgeIndex::new(0),
                end_edge: EdgeIndex::new(2),
            };
            let result = edges.get(&node);
            assert_eq!(result.len(), 2);
            assert_eq!(result[0].into_inner(), 1);
            assert_eq!(result[1].into_inner(), 2);
        }

        #[test]
        fn empty_edges() {
            let edges = Edges { edges: vec![] };
            let node = Node {
                hash: NodeHash::from_slice(b"abc"),
                start_edge: EdgeIndex::new(0),
                end_edge: EdgeIndex::new(0),
            };
            assert!(edges.get(&node).is_empty());
        }
    }

    mod graph {
        use super::*;

        #[test]
        fn find_paths_single_step() {
            let input = b"abc: out\n";
            let graph = Graph::from_input(input);
            let start = graph.get_by_hash(NodeHash::from_slice(b"abc"));
            let end = graph.get_by_hash(NodeHash::from_slice(b"out"));
            let path_count = graph.find_paths(start, end);
            assert_eq!(path_count, 1);
        }

        #[test]
        fn find_paths_two_steps() {
            let input = b"abc: def\ndef: out\n";
            let graph = Graph::from_input(input);
            let start = graph.get_by_hash(NodeHash::from_slice(b"abc"));
            let end = graph.get_by_hash(NodeHash::from_slice(b"out"));
            let path_count = graph.find_paths(start, end);
            assert_eq!(path_count, 1);
        }

        #[test]
        fn find_paths_multiple_paths() {
            let input = b"abc: def ghi\n def: out\nghi: out\n";
            let graph = Graph::from_input(input);
            let start = graph.get_by_hash(NodeHash::from_slice(b"abc"));
            let end = graph.get_by_hash(NodeHash::from_slice(b"out"));
            let path_count = graph.find_paths(start, end);
            assert_eq!(path_count, 2);
        }

        #[test]
        fn parses_node_count() {
            let input = b"abc: def ghi\njkl: mno out\n";
            let graph = Graph::from_input(input);
            // 2 lines + out node
            assert_eq!(graph.nodes.nodes.len(), 3);
        }

        #[test]
        fn parses_edge_count() {
            let input = b"abc: def ghi\njkl: out\n";
            let graph = Graph::from_input(input);
            // abc->def, abc->ghi, jkl->out
            assert_eq!(graph.edges.edges.len(), 3);
        }

        #[test]
        fn out_node_has_no_edges() {
            let input = b"abc: out\n";
            let graph = Graph::from_input(input);
            let out_idx = graph.hash_to_index.get(NodeHash::from_slice(b"out"));
            let out_node = graph.nodes.get(out_idx);
            assert!(graph.edges.get(out_node).is_empty());
        }

        #[test]
        fn lookup_resolves_correctly() {
            let input = b"abc: def\ndef: out\n";
            let graph = Graph::from_input(input);
            let abc_idx = graph.hash_to_index.get(NodeHash::from_slice(b"abc"));
            let def_idx = graph.hash_to_index.get(NodeHash::from_slice(b"def"));
            let abc_node = graph.nodes.get(abc_idx);
            let abc_edges = graph.edges.get(abc_node);
            assert_eq!(abc_edges[0], def_idx);
        }
        
        #[test]
        fn out_node_index_is_last() {
            let input = b"abc: out\ndef: out\nghi: out\n";
            let graph = Graph::from_input(input);
            let out_idx = graph.hash_to_index.get(NodeHash::from_slice(b"out"));
            // out should be at index 3 (after abc=0, def=1, ghi=2)
            assert_eq!(out_idx.into_inner(), 3);
        }

        #[test]
        fn out_node_hash_is_correct() {
            let input = b"abc: out\n";
            let graph = Graph::from_input(input);
            let out_idx = graph.hash_to_index.get(NodeHash::from_slice(b"out"));
            let out_node = graph.nodes.get(out_idx);
            assert_eq!(out_node.hash, NodeHash::from_slice(b"out"));
        }

        #[test]
        fn edges_point_to_out_node() {
            let input = b"abc: out\ndef: out\n";
            let graph = Graph::from_input(input);
            let out_idx = graph.hash_to_index.get(NodeHash::from_slice(b"out"));

            let abc_node = graph.nodes.get(graph.hash_to_index.get(NodeHash::from_slice(b"abc")));
            let def_node = graph.nodes.get(graph.hash_to_index.get(NodeHash::from_slice(b"def")));

            assert_eq!(graph.edges.get(abc_node)[0], out_idx);
            assert_eq!(graph.edges.get(def_node)[0], out_idx);
        }

        #[test]
        fn out_is_last_node_in_vec() {
            let input = b"aaa: out\nbbb: out\n";
            let graph = Graph::from_input(input);
            let last_node = graph.nodes.nodes.last().unwrap();
            assert_eq!(last_node.hash, NodeHash::from_slice(b"out"));
        }
    }
}