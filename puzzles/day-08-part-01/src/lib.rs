pub fn solve(input: &str) -> usize {
    solve_with_connections(input, 1000)
}

pub fn solve_with_connections(input: &str, num_connections: usize) -> usize {
    let playground = Playground::parse(input);
    let edges = playground.edges_by_distance();

    let mut circuits = CircuitTracker::new(playground.len());
    for edge in edges.iter().take(num_connections) {
        circuits.connect(edge.from, edge.to);
    }

    circuits.product_of_three_largest_circuits()
}

/// vertex
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct JunctionBox(u32);

impl JunctionBox {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn id(self) -> usize {
        self.0 as usize
    }
}

/// weighted edge
#[derive(Clone, Copy, Debug)]
pub struct StringOfLights {
    pub from: JunctionBox,
    pub to: JunctionBox,
    pub squared_distance: u64,
}

/// graph
pub struct Playground {
    x: Vec<u32>,
    y: Vec<u32>,
    z: Vec<u32>,
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

        for line in lines {
            let (px, py, pz) = Self::parse_position(line);
            x.push(px);
            y.push(py);
            z.push(pz);
        }

        Self { x, y, z }
    }

    #[inline]
    fn parse_position(line: &[u8]) -> (u32, u32, u32) {
        let mut parts = line.split(|&b| b == b',');
        (
            Self::parse_u32(parts.next().unwrap()),
            Self::parse_u32(parts.next().unwrap()),
            Self::parse_u32(parts.next().unwrap()),
        )
    }

    #[inline]
    fn parse_u32(s: &[u8]) -> u32 {
        s.iter().fold(0u32, |acc, &b| acc * 10 + (b - b'0') as u32)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    #[inline(always)]
    fn squared_distance(&self, i: usize, j: usize) -> u64 {
        let dx = self.x[i] as i64 - self.x[j] as i64;
        let dy = self.y[i] as i64 - self.y[j] as i64;
        let dz = self.z[i] as i64 - self.z[j] as i64;
        (dx * dx + dy * dy + dz * dz) as u64
    }

    // step 1
    pub fn edges_by_distance(&self) -> Vec<StringOfLights> {
        const MAX_EDGES_NEEDED: usize = 1000;

        let n = self.len();
        let mut edges = Vec::with_capacity(n * (n - 1) / 2);

        // dit kan ik nog simd maken
        for i in 0..n {
            for j in (i + 1)..n {
                edges.push(StringOfLights {
                    from: JunctionBox::new(i as u32),
                    to: JunctionBox::new(j as u32),
                    squared_distance: self.squared_distance(i, j),
                });
            }
        }

        if edges.len() > MAX_EDGES_NEEDED * 2 {
            edges.select_nth_unstable_by_key(MAX_EDGES_NEEDED, |e| e.squared_distance);
            edges.truncate(MAX_EDGES_NEEDED + 1);
        }
        edges.sort_unstable_by_key(|e| e.squared_distance);

        edges
    }
}

/// union-find
pub struct CircuitTracker {
    parent: Vec<u32>,
    rank: Vec<u8>,
    size: Vec<u32>,
}

impl CircuitTracker {
    pub fn new(num_junction_boxes: usize) -> Self {
        Self {
            parent: (0..num_junction_boxes as u32).collect(),
            rank: vec![0; num_junction_boxes],
            size: vec![1; num_junction_boxes],
        }
    }

    #[inline]
    fn find_circuit(&mut self, jb: JunctionBox) -> u32 {
        let mut x = jb.0;
        let mut root = x;

        // path compression
        while self.parent[root as usize] != root {
            root = self.parent[root as usize];
        }

        while self.parent[x as usize] != root {
            let next = self.parent[x as usize];
            self.parent[x as usize] = root;
            x = next;
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

        let (small, large) = if self.rank[ra as usize] < self.rank[rb as usize] {
            (ra, rb)
        } else {
            (rb, ra)
        };

        self.parent[small as usize] = large;
        self.size[large as usize] += self.size[small as usize];

        if self.rank[ra as usize] == self.rank[rb as usize] {
            self.rank[large as usize] += 1;
        }

        true
    }

    /// aoc result
    pub fn product_of_three_largest_circuits(&self) -> usize {
        let mut top = [1usize; 3];

        for (i, &p) in self.parent.iter().enumerate() {
            if p == i as u32 {
                let s = self.size[i] as usize;
                if s > top[0] {
                    top[2] = top[1];
                    top[1] = top[0];
                    top[0] = s;
                } else if s > top[1] {
                    top[2] = top[1];
                    top[1] = s;
                } else if s > top[2] {
                    top[2] = s;
                }
            }
        }

        top.iter().product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod junction_box {
        use super::*;

        #[test]
        fn id_roundtrip() {
            let jb = JunctionBox::new(42);
            assert_eq!(jb.id(), 42);
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
        fn edges_sorted_by_distance() {
            let pg = Playground::parse("0,0,0\n1,0,0\n10,0,0\n");
            let edges = pg.edges_by_distance();

            assert_eq!(edges.len(), 3);
            assert_eq!(edges[0].squared_distance, 1);
            assert_eq!(edges[1].squared_distance, 81);
            assert_eq!(edges[2].squared_distance, 100);
        }
    }

    mod circuit_tracker {
        use super::*;

        #[test]
        fn initial_state() {
            let circuits = CircuitTracker::new(3);

            assert_eq!(circuits.product_of_three_largest_circuits(), 1);
        }

        #[test]
        fn connect_merges_circuits() {
            let mut circuits = CircuitTracker::new(5);

            circuits.connect(JunctionBox::new(0), JunctionBox::new(1));
            circuits.connect(JunctionBox::new(2), JunctionBox::new(3));
            circuits.connect(JunctionBox::new(3), JunctionBox::new(4));

            assert_eq!(circuits.product_of_three_largest_circuits(), 6);
        }

        #[test]
        fn connect_same_circuit_returns_false() {
            let mut circuits = CircuitTracker::new(3);
            circuits.connect(JunctionBox::new(0), JunctionBox::new(1));

            assert!(!circuits.connect(JunctionBox::new(0), JunctionBox::new(1)));
            assert!(!circuits.connect(JunctionBox::new(1), JunctionBox::new(0)));
        }
    }

    mod kruskal {
        use super::*;

        #[test]
        fn example_from_description() {
            let input = std::fs::read_to_string("example.txt").unwrap();
            assert_eq!(solve_with_connections(&input, 10), 40);
        }
    }
}
