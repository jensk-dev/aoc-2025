use std::cmp::Reverse;
use std::collections::BinaryHeap;
use wide::i32x8;

pub fn solve(input: &str) -> usize {
    let playground = Playground::parse(input);
    let num_coords = playground.len();

    let mut heap = playground.edges_as_heap();
    let mut circuits = CircuitTracker::new(num_coords);

    let mut last_edge = None;
    let mut edges_added = 0;

    while let Some(Reverse(edge)) = heap.pop() {
        if circuits.connect(edge.from, edge.to) {
            last_edge = Some(edge);
            edges_added += 1;
            if edges_added == num_coords - 1 {
                break;
            }
        }
    }

    let edge = last_edge.expect("MST requires at least one edge");
    playground.x_coord(edge.from) as usize * playground.x_coord(edge.to) as usize
}

#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// vertex (index)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct JunctionBox(u16);

impl JunctionBox {
    #[inline(always)]
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn id(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for JunctionBox {
    #[inline(always)]
    fn from(id: usize) -> Self {
        Self(id as u16)
    }
}

/// squared euclid dist between junction boxes
#[inline(always)]
const fn squared_distance(dx: i64, dy: i64, dz: i64) -> u64 {
    (dx * dx + dy * dy + dz * dz) as u64
}

/// weighted edge between two junction boxes
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StringOfLights {
    pub from: JunctionBox,
    pub to: JunctionBox,
    pub squared_distance: u64,
}

impl StringOfLights {
    #[inline(always)]
    pub const fn new(from: JunctionBox, to: JunctionBox, squared_distance: u64) -> Self {
        Self {
            from,
            to,
            squared_distance,
        }
    }
}

impl Ord for StringOfLights {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.squared_distance.cmp(&other.squared_distance)
    }
}

impl PartialOrd for StringOfLights {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Playground {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
    pub z: Vec<i32>,
}

impl Playground {
    pub fn parse(input: &str) -> Self {
        let lines: Vec<_> = input
            .as_bytes()
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
            .collect();

        let n = lines.len();
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        let mut z = Vec::with_capacity(n);

        unsafe {
            x.set_len(n);
            y.set_len(n);
            z.set_len(n);
        }

        for (idx, line) in lines.into_iter().enumerate() {
            let (px, py, pz) = Self::parse_position(line);
            unsafe {
                debug_assert!(idx < n);
                *x.get_unchecked_mut(idx) = px;
                *y.get_unchecked_mut(idx) = py;
                *z.get_unchecked_mut(idx) = pz;
            }
        }

        Self { x, y, z }
    }

    #[inline]
    fn parse_position(line: &[u8]) -> (i32, i32, i32) {
        let mut parts = line.split(|&b| b == b',');
        (
            Self::parse_i32(parts.next().unwrap()),
            Self::parse_i32(parts.next().unwrap()),
            Self::parse_i32(parts.next().unwrap()),
        )
    }

    #[inline]
    fn parse_i32(s: &[u8]) -> i32 {
        s.iter().fold(0i32, |acc, &b| acc * 10 + (b - b'0') as i32)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Get x-coordinate for a junction box
    #[inline(always)]
    pub fn x_coord(&self, jb: JunctionBox) -> i32 {
        self.x[jb.id()]
    }

    /// Build min-heap of all edges using cache-blocked iteration and i32x8 SIMD
    pub fn edges_as_heap(&self) -> BinaryHeap<Reverse<StringOfLights>> {
        const BLOCK_SIZE: usize = 64;

        let num_coords = self.len();
        let num_edges = num_coords * (num_coords - 1) / 2;

        let mut edges: Vec<Reverse<StringOfLights>> = Vec::with_capacity(num_edges);

        for block_i in (0..num_coords).step_by(BLOCK_SIZE) {
            let block_i_end = (block_i + BLOCK_SIZE).min(num_coords);

            for block_j in (block_i..num_coords).step_by(BLOCK_SIZE) {
                let block_j_end = (block_j + BLOCK_SIZE).min(num_coords);

                for src in block_i..block_i_end {
                    // broadcast source coordinates
                    let src_x = i32x8::splat(self.x[src]);
                    let src_y = i32x8::splat(self.y[src]);
                    let src_z = i32x8::splat(self.z[src]);

                    // avoid duplicate edges: only process dst > src within same block
                    let mut dst = if block_i == block_j { src + 1 } else { block_j };

                    // direct load 8 destinations per iter
                    while dst + 8 <= block_j_end {
                        let dst_x = i32x8::new(self.x[dst..dst + 8].try_into().unwrap());
                        let dst_y = i32x8::new(self.y[dst..dst + 8].try_into().unwrap());
                        let dst_z = i32x8::new(self.z[dst..dst + 8].try_into().unwrap());

                        let delta_x = src_x - dst_x;
                        let delta_y = src_y - dst_y;
                        let delta_z = src_z - dst_z;

                        let delta_x_arr = delta_x.to_array();
                        let delta_y_arr = delta_y.to_array();
                        let delta_z_arr = delta_z.to_array();

                        for lane in 0..8 {
                            let dx = delta_x_arr[lane] as i64;
                            let dy = delta_y_arr[lane] as i64;
                            let dz = delta_z_arr[lane] as i64;
                            edges.push(Reverse(StringOfLights::new(
                                src.into(),
                                (dst + lane).into(),
                                squared_distance(dx, dy, dz),
                            )));
                        }
                        dst += 8;
                    }

                    // handle remaining dst values
                    while dst < block_j_end {
                        let dx = (self.x[src] - self.x[dst]) as i64;
                        let dy = (self.y[src] - self.y[dst]) as i64;
                        let dz = (self.z[src] - self.z[dst]) as i64;

                        edges.push(Reverse(StringOfLights::new(
                            src.into(),
                            dst.into(),
                            squared_distance(dx, dy, dz),
                        )));
                        dst += 1;
                    }
                }
            }
        }

        BinaryHeap::from(edges)
    }
}

/// union-find
pub struct CircuitTracker {
    parent: Vec<u16>,
    rank: Vec<u8>,
    size: Vec<u16>,
}

impl CircuitTracker {
    pub fn new(num_junction_boxes: usize) -> Self {
        Self {
            parent: (0..num_junction_boxes as u16).collect(),
            rank: vec![0; num_junction_boxes],
            size: vec![1; num_junction_boxes],
        }
    }

    #[inline]
    fn find_circuit(&mut self, jb: JunctionBox) -> u16 {
        let mut x = jb.0;
        let mut root = x;

        // path compression
        unsafe {
            while *self.parent.get_unchecked(root as usize) != root {
                root = *self.parent.get_unchecked(root as usize);
            }
            while *self.parent.get_unchecked(x as usize) != root {
                let next = *self.parent.get_unchecked(x as usize);
                *self.parent.get_unchecked_mut(x as usize) = root;
                x = next;
            }
        }

        root
    }

    // edge-adding
    #[inline]
    pub fn connect(&mut self, a: JunctionBox, b: JunctionBox) -> bool {
        let ra = self.find_circuit(a);
        let rb = self.find_circuit(b);

        if ra == rb {
            return false;
        }

        unsafe {
            let rank_a = *self.rank.get_unchecked(ra as usize);
            let rank_b = *self.rank.get_unchecked(rb as usize);

            let (small, large) = if rank_a < rank_b { (ra, rb) } else { (rb, ra) };

            *self.parent.get_unchecked_mut(small as usize) = large;
            *self.size.get_unchecked_mut(large as usize) +=
                *self.size.get_unchecked(small as usize);

            if rank_a == rank_b {
                *self.rank.get_unchecked_mut(large as usize) += 1;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod squared_distance_tests {
        use super::*;

        #[test]
        fn zero_distance_when_same_point() {
            assert_eq!(squared_distance(0, 0, 0), 0);
        }

        #[test]
        fn unit_distance_along_single_axis() {
            assert_eq!(squared_distance(1, 0, 0), 1);
            assert_eq!(squared_distance(0, 1, 0), 1);
            assert_eq!(squared_distance(0, 0, 1), 1);
        }

        #[test]
        fn negative_deltas_produce_same_result_as_positive() {
            assert_eq!(squared_distance(-3, -4, -5), squared_distance(3, 4, 5));
        }

        #[test]
        fn pythagorean() {
            assert_eq!(squared_distance(3, 4, 0), 25);
        }

        #[test]
        fn diagonal_in_3d() {
            assert_eq!(squared_distance(1, 1, 1), 3);
        }
    }

    mod junction_box {
        use super::*;

        #[test]
        fn id_roundtrip() {
            let jb = JunctionBox::new(42u16);
            assert_eq!(jb.id(), 42);
        }

        #[test]
        fn from_usize_conversion() {
            let jb: JunctionBox = 123usize.into();
            assert_eq!(jb.id(), 123);
        }

        #[test]
        fn equality_based_on_id() {
            let a = JunctionBox::new(5);
            let b = JunctionBox::new(5);
            let c = JunctionBox::new(6);
            assert_eq!(a, b);
            assert_ne!(a, c);
        }
    }

    mod string_of_lights {
        use super::*;

        #[test]
        fn ordering_by_squared_distance() {
            let short = StringOfLights::new(0.into(), 1.into(), 10);
            let long = StringOfLights::new(0.into(), 2.into(), 100);

            assert!(short < long);
            assert!(long > short);
        }

        #[test]
        fn equal_distances_are_equal() {
            let a = StringOfLights::new(0.into(), 1.into(), 50);
            let b = StringOfLights::new(2.into(), 3.into(), 50);

            assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
        }

        #[test]
        fn min_heap_with_reverse_yields_shortest_first() {
            let edges = vec![
                Reverse(StringOfLights::new(0.into(), 1.into(), 100)),
                Reverse(StringOfLights::new(0.into(), 2.into(), 10)),
                Reverse(StringOfLights::new(1.into(), 2.into(), 50)),
            ];
            let mut heap = BinaryHeap::from(edges);

            assert_eq!(heap.pop().unwrap().0.squared_distance, 10);
            assert_eq!(heap.pop().unwrap().0.squared_distance, 50);
            assert_eq!(heap.pop().unwrap().0.squared_distance, 100);
        }
    }

    mod playground {
        use super::*;

        #[test]
        fn parse_positions() {
            let pg = Playground::parse("1,2,3\n4,5,6\n");
            assert_eq!(pg.len(), 2);
            assert_eq!((pg.x[0], pg.y[0], pg.z[0]), (1, 2, 3));
            assert_eq!((pg.x[1], pg.y[1], pg.z[1]), (4, 5, 6));
        }

        #[test]
        fn parse_handles_trailing_newline() {
            let with_trailing = Playground::parse("1,2,3\n");
            let without_trailing = Playground::parse("1,2,3");
            assert_eq!(with_trailing.len(), 1);
            assert_eq!(without_trailing.len(), 1);
        }

        #[test]
        fn x_coord_returns_correct_value() {
            let pg = Playground::parse("10,20,30\n40,50,60\n");
            assert_eq!(pg.x_coord(JunctionBox::new(0)), 10);
            assert_eq!(pg.x_coord(JunctionBox::new(1)), 40);
        }

        #[test]
        fn edges_as_heap_creates_correct_number_of_edges() {
            let pg = Playground::parse("0,0,0\n1,0,0\n0,1,0\n");
            let heap = pg.edges_as_heap();

            assert_eq!(heap.len(), 3);
        }

        #[test]
        fn edges_as_heap_calculates_correct_distances() {
            let pg = Playground::parse("0,0,0\n3,4,0\n0,0,5\n");
            let heap = pg.edges_as_heap();

            let mut distances: Vec<u64> = heap.into_iter().map(|r| r.0.squared_distance).collect();
            distances.sort();

            assert_eq!(distances, vec![25, 25, 50]);
        }
    }

    mod circuit_tracker {
        use super::*;

        #[test]
        fn new_creates_isolated_components() {
            let circuits = CircuitTracker::new(5);
            assert_eq!(circuits.parent.len(), 5);
            for i in 0..5 {
                assert_eq!(circuits.parent[i], i as u16);
            }
        }

        #[test]
        fn connect_returns_true_for_different_circuits() {
            let mut circuits = CircuitTracker::new(3);
            assert!(circuits.connect(JunctionBox::new(0), JunctionBox::new(1)));
        }

        #[test]
        fn connect_same_circuit_returns_false() {
            let mut circuits = CircuitTracker::new(3);
            circuits.connect(JunctionBox::new(0u16), JunctionBox::new(1u16));

            assert!(!circuits.connect(JunctionBox::new(0u16), JunctionBox::new(1u16)));
            assert!(!circuits.connect(JunctionBox::new(1u16), JunctionBox::new(0u16)));
        }

        #[test]
        fn transitive_connectivity() {
            let mut circuits = CircuitTracker::new(4);

            assert!(circuits.connect(JunctionBox::new(0), JunctionBox::new(1)));
            assert!(circuits.connect(JunctionBox::new(2), JunctionBox::new(3)));
            assert!(circuits.connect(JunctionBox::new(0), JunctionBox::new(2)));
            assert!(!circuits.connect(JunctionBox::new(1), JunctionBox::new(3)));
        }

        #[test]
        fn union_by_rank_balances_tree() {
            let mut circuits = CircuitTracker::new(8);

            circuits.connect(JunctionBox::new(0), JunctionBox::new(1));
            circuits.connect(JunctionBox::new(2), JunctionBox::new(3));
            circuits.connect(JunctionBox::new(0), JunctionBox::new(2));

            circuits.connect(JunctionBox::new(4), JunctionBox::new(5));
            circuits.connect(JunctionBox::new(6), JunctionBox::new(7));
            circuits.connect(JunctionBox::new(4), JunctionBox::new(6));

            assert!(circuits.connect(JunctionBox::new(0), JunctionBox::new(4)));
            assert!(!circuits.connect(JunctionBox::new(1), JunctionBox::new(7)));
        }
    }

    mod kruskal {
        use super::*;

        #[test]
        fn example_from_description() {
            let input = std::fs::read_to_string("example.txt").unwrap();
            assert_eq!(solve(&input), 25272);
        }

        #[test]
        fn minimal_mst_with_two_points() {
            let input = "5,0,0\n7,0,0\n";
            assert_eq!(solve(input), 5 * 7);
        }

        #[test]
        fn three_points_mst_picks_two_shortest_edges() {
            let input = "10,0,0\n20,0,0\n100,0,0\n";
            assert_eq!(solve(input), 20 * 100);
        }
    }
}
