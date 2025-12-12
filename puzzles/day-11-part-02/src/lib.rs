use graph::Graph;
use node_hash::NodeHash;

const BASE: u16 = 26;
const MAX_HASHES: usize = (BASE * BASE * BASE) as usize;

pub fn solve(input: &str) -> u64 {
    let graph = Graph::from_input(input.as_bytes());

    let svr = graph.get_index_by_hash(NodeHash::from_slice(b"svr"));
    let out = graph.get_index_by_hash(NodeHash::from_slice(b"out"));
    let dac = graph.get_index_by_hash(NodeHash::from_slice(b"dac"));
    let fft = graph.get_index_by_hash(NodeHash::from_slice(b"fft"));

    let num_nodes = graph.node_count();
    let mut memo: Vec<Option<u64>> = vec![None; num_nodes];

    let fft_to_out = graph.count_paths_memo(fft, out, &mut memo);
    let dac_to_out = graph.count_paths_memo(dac, out, &mut memo);

    memo.fill(None);
    let dac_to_fft = graph.count_paths_memo(dac, fft, &mut memo);
    let svr_to_fft = graph.count_paths_memo(svr, fft, &mut memo);

    memo.fill(None);
    let svr_to_dac = graph.count_paths_memo(svr, dac, &mut memo);
    let fft_to_dac = graph.count_paths_memo(fft, dac, &mut memo);

    // svr > dac > fft > out
    let paths_a = svr_to_dac * dac_to_fft * fft_to_out;

    // svr > fft > dac > out
    let paths_b = svr_to_fft * fft_to_dac * dac_to_out;

    paths_a + paths_b
}

mod graph {
    use crate::MAX_HASHES;

    use super::node_index::NodeIndex;
    use super::edge_index::EdgeIndex;
    use super::node_hash::NodeHash;

    pub struct Graph {
        nodes: Nodes,
        edges: Edges,
        hash_to_index: NodeIndexLookupTable,
    }

    struct Nodes {
        nodes: Vec<Node>,
    }

    impl Nodes {
        fn len(&self) -> usize {
            self.nodes.len()
        }

        fn get(&self, index: NodeIndex) -> &Node {
            &self.nodes[index.into_inner() as usize]
        }

        #[cfg(test)]
        fn last(&self) -> Option<&Node> {
            self.nodes.last()
        }
    }

    struct Edges {
        edges: Vec<NodeIndex>,
    }

    impl Edges {
        #[cfg(test)]
        fn len(&self) -> usize {
            self.edges.len()
        }

        fn get(&self, node: &Node) -> &[NodeIndex] {
            &self.edges[node.start_edge.into_inner()..node.end_edge.into_inner()]
        }
    }

    impl Graph {
        pub fn node_count(&self) -> usize {
            self.nodes.len()
        }

        pub fn from_input(input: &[u8]) -> Self {
            let (num_lines, num_edges) = input.iter().fold((0, 0), |(lines, spaces), &b| {
                (lines + (b == b'\n') as usize, spaces + (b == b' ') as usize)
            });

            let mut nodes = Vec::with_capacity(num_lines + 1); // +1 for out
            let mut edges = Vec::with_capacity(num_edges);
            let mut hash_to_index = NodeIndexLookupTableBuilder::new();

            // 1st pass register nodes
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

            // add out node
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

        pub fn get_index_by_hash(&self, hash: NodeHash) -> NodeIndex {
            self.hash_to_index.get(hash)
        }

        pub fn count_paths_memo(&self, current: NodeIndex, end: NodeIndex, memo: &mut [Option<u64>]) -> u64 {
            if current == end {
                return 1;
            }

            if let Some(cached) = memo[current.into_inner() as usize] {
                return cached;
            }

            let node = self.nodes.get(current);
            let mut total = 0u64;
            for &neighbor in self.edges.get(node) {
                total += self.count_paths_memo(neighbor, end, memo);
            }

            memo[current.into_inner() as usize] = Some(total);
            total
        }
    }

    struct NodeIndexLookupTable {
        table: [NodeIndex; MAX_HASHES],
    }

    impl NodeIndexLookupTable {
        fn get(&self, hash: NodeHash) -> NodeIndex {
            debug_assert!(hash.is_valid(), "Cannot get invalid node hash");
            self.table[(hash.into_inner() - 1) as usize]
        }
    }

    struct NodeIndexLookupTableBuilder {
        table: [NodeIndex; MAX_HASHES],
    }

    impl NodeIndexLookupTableBuilder {
        fn new() -> Self {
            NodeIndexLookupTableBuilder {
                table: [NodeIndex::new(0); MAX_HASHES],
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
        #[cfg(test)]
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

        impl Graph {
            fn find_paths_by_hash(&self, start: NodeHash, end: NodeHash) -> usize {
                let start_node = self.get_by_hash(start);
                let end_node = self.get_by_hash(end);
                self.find_paths(start_node, end_node)
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

            fn get_edges_for_hash(&self, hash: NodeHash) -> impl Iterator<Item = NodeIndex> + '_ {
                let node = self.get_by_hash(hash);
                self.edges.get(node).iter().copied()
            }

            fn out_node_edges_count(&self, hash: NodeHash) -> usize {
                let node = self.get_by_hash(hash);
                self.edges.get(node).len()
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

        mod graph_tests {
            use super::*;

            #[test]
            fn find_paths_single_step() {
                let input = b"abc: out\n";
                let graph = Graph::from_input(input);
                let path_count = graph.find_paths_by_hash(
                    NodeHash::from_slice(b"abc"),
                    NodeHash::from_slice(b"out")
                );
                assert_eq!(path_count, 1);
            }

            #[test]
            fn find_paths_two_steps() {
                let input = b"abc: def\ndef: out\n";
                let graph = Graph::from_input(input);
                let path_count = graph.find_paths_by_hash(
                    NodeHash::from_slice(b"abc"),
                    NodeHash::from_slice(b"out")
                );
                assert_eq!(path_count, 1);
            }

            #[test]
            fn find_paths_multiple_paths() {
                let input = b"abc: def ghi\ndef: out\nghi: out\n";
                let graph = Graph::from_input(input);
                let path_count = graph.find_paths_by_hash(
                    NodeHash::from_slice(b"abc"),
                    NodeHash::from_slice(b"out")
                );
                assert_eq!(path_count, 2);
            }

            #[test]
            fn parses_node_count() {
                let input = b"abc: def ghi\njkl: mno out\n";
                let graph = Graph::from_input(input);
                // 2 lines + out node
                assert_eq!(graph.node_count(), 3);
            }

            #[test]
            fn parses_edge_count() {
                let input = b"abc: def ghi\njkl: out\n";
                let graph = Graph::from_input(input);
                // abc->def, abc->ghi, jkl->out
                assert_eq!(graph.edges.len(), 3);
            }

            #[test]
            fn out_node_has_no_edges() {
                let input = b"abc: out\n";
                let graph = Graph::from_input(input);
                assert_eq!(graph.out_node_edges_count(NodeHash::from_slice(b"out")), 0);
            }

            #[test]
            fn lookup_resolves_correctly() {
                let input = b"abc: def\ndef: out\n";
                let graph = Graph::from_input(input);
                let def_idx = graph.get_index_by_hash(NodeHash::from_slice(b"def"));
                let abc_edges: Vec<_> = graph.get_edges_for_hash(NodeHash::from_slice(b"abc")).collect();
                assert_eq!(abc_edges[0], def_idx);
            }

            #[test]
            fn out_node_index_is_last() {
                let input = b"abc: out\ndef: out\nghi: out\n";
                let graph = Graph::from_input(input);
                let out_idx = graph.get_index_by_hash(NodeHash::from_slice(b"out"));
                // out should be at index 3 (after abc=0, def=1, ghi=2)
                assert_eq!(out_idx.into_inner(), 3);
            }

            #[test]
            fn out_node_hash_is_correct() {
                let input = b"abc: out\n";
                let graph = Graph::from_input(input);
                let out_idx = graph.get_index_by_hash(NodeHash::from_slice(b"out"));
                // Verify the out node is at the expected index
                assert_eq!(out_idx.into_inner(), 1);
            }

            #[test]
            fn edges_point_to_out_node() {
                let input = b"abc: out\ndef: out\n";
                let graph = Graph::from_input(input);
                let out_idx = graph.get_index_by_hash(NodeHash::from_slice(b"out"));

                let abc_edges: Vec<_> = graph.get_edges_for_hash(NodeHash::from_slice(b"abc")).collect();
                let def_edges: Vec<_> = graph.get_edges_for_hash(NodeHash::from_slice(b"def")).collect();

                assert_eq!(abc_edges[0], out_idx);
                assert_eq!(def_edges[0], out_idx);
            }

            #[test]
            fn out_is_last_node_in_vec() {
                let input = b"aaa: out\nbbb: out\n";
                let graph = Graph::from_input(input);
                assert_eq!(graph.nodes.last().map(|n| n.hash), Some(NodeHash::from_slice(b"out")));
            }
        }
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
    use crate::BASE;

    const POWERS_OF_BASE: [u16; 3] = [(BASE * BASE) as u16, BASE as u16, 1];

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct NodeHash(u16);

    impl NodeHash {
        pub const INVALID: Self = NodeHash(0);

        #[inline]
        const fn get_base26_offset(ascii: u8, position: u8) -> u16 {
            (ascii - b'a') as u16 * POWERS_OF_BASE[position as usize] + 1
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_index::NodeIndex;
    use crate::edge_index::EdgeIndex;
    use rstest::rstest;

    const EXAMPLE: &str = indoc::indoc! {
        "svr: aaa bbb
         aaa: fft
         fft: ccc
         bbb: tty
         tty: ccc
         ccc: ddd eee
         ddd: hub
         hub: fff
         eee: dac
         dac: fff
         fff: ggg hhh
         ggg: out
         hhh: out"
    };

    #[test]
    fn solve_example() {
        let input = EXAMPLE;
        let result = super::solve(input);
        assert_eq!(result, 2);
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
        #[case(b"aba", 3 + BASE)]
        #[case(b"baa", 3 + BASE * BASE)]
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
}
