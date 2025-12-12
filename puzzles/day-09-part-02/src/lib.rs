pub fn solve(input: &str) -> usize {
    let points = parse_input(input);
    let compressor = CoordCompressor::new(&points);

    // compress + pad to avoid bounds checks in hotpath
    let compressed_points: Vec<Point> = points
        .iter()
        .map(|p| compressor.compress(*p).offset(1, 1))
        .collect();

    // create prefix sum from compressed grid
    let prefix_sum = Grid::new(&compressor)
        .with_shape(&compressed_points)
        .with_outside_filled()
        .with_inside_marked()
        .into_prefix_sum();

    let mut max_area = 0;
    let n = points.len();

    for i in 0..n {
        for j in (i + 1)..n {
            let (c1, c2, p1, p2) = unsafe {
                (
                    *compressed_points.get_unchecked(i),
                    *compressed_points.get_unchecked(j),
                    *points.get_unchecked(i),
                    *points.get_unchecked(j),
                )
            };

            // check if compressed rect is valid
            let compressed_rect = Rect::from_corners(c1, c2);
            if prefix_sum.is_rect_valid(compressed_rect) {
                // if valid, check if max
                let original_rect = Rect::from_corners(p1, p2);
                max_area = max_area.max(original_rect.area());
            }
        }
    }

    max_area
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Unknown = 0,
    Valid = 1,
    Outside = 2,
}

impl Cell {
    #[inline]
    fn is_valid(self) -> bool {
        self == Cell::Valid
    }

    #[inline]
    fn is_unknown(self) -> bool {
        self == Cell::Unknown
    }
}

struct Grid {
    data: Vec<Cell>,
    cols: usize,
    rows: usize,
}

impl Grid {
    // creates grid from compressed coordinates
    pub fn new(compressor: &CoordCompressor) -> Self {
        // +2 padding so flood fill does not need bounds checking
        let cols = compressor.cols() + 2;
        let rows = compressor.rows() + 2;
        let data = vec![Cell::Unknown; cols * rows];

        Self { data, cols, rows }
    }

    // draw shape of all consecutive points
    #[inline]
    pub fn with_shape(mut self, points: &[Point]) -> Self {
        for window in points.windows(2) {
            self.draw_line(window[0], window[1]);
        }
        // laatste punt sluit weer aan op eerste
        if let (Some(&first), Some(&last)) = (points.first(), points.last()) {
            self.draw_line(last, first);
        }
        self
    }

    // flood fill up to shape
    #[inline]
    pub fn with_outside_filled(mut self) -> Self {
        self.df_flood_fill(Point::new(0, 0), Cell::Outside);
        self
    }

    // mark cells not yet marked as valid
    #[inline]
    pub fn with_inside_marked(mut self) -> Self {
        for cell in &mut self.data {
            if *cell == Cell::Unknown {
                *cell = Cell::Valid;
            }
        }
        self
    }

    // convert to prefix sum
    #[inline]
    pub fn into_prefix_sum(self) -> PrefixSum2D {
        PrefixSum2D::new(&self.data, self.cols, self.rows)
    }

    #[inline]
    fn get(&self, point: Point) -> Cell {
        self.data[point.y as usize * self.cols + point.x as usize]
    }

    #[inline]
    fn set(&mut self, point: Point, value: Cell) {
        self.data[point.y as usize * self.cols + point.x as usize] = value;
    }

    fn draw_line(&mut self, from: Point, to: Point) {
        if from.x == to.x {
            // only vertical
            let x = from.x;
            let (min_y, max_y) = if from.y < to.y { (from.y, to.y) } else { (to.y, from.y) };
            for y in min_y..=max_y {
                self.set(Point::new(x, y), Cell::Valid);
            }
        } else {
            // only horizontal
            let y = from.y;
            let (min_x, max_x) = if from.x < to.x { (from.x, to.x) } else { (to.x, from.x) };
            for x in min_x..=max_x {
                self.set(Point::new(x, y), Cell::Valid);
            }
        }
    }

    fn df_flood_fill(&mut self, start: Point, value: Cell) {
        let mut stack = vec![start];

        while let Some(point) = stack.pop() {
            if point.x >= self.cols as u32 || point.y >= self.rows as u32 {
                continue;
            }
            if !self.get(point).is_unknown() {
                continue;
            }

            self.set(point, value);

            if point.x > 0 {
                stack.push(Point::new(point.x - 1, point.y));
            }
            if point.x + 1 < self.cols as u32 {
                stack.push(Point::new(point.x + 1, point.y));
            }
            if point.y > 0 {
                stack.push(Point::new(point.x, point.y - 1));
            }
            if point.y + 1 < self.rows as u32 {
                stack.push(Point::new(point.x, point.y + 1));
            }
        }
    }
}

fn parse_point(s: &[u8]) -> Point {
    let mut parts = s.split(|&b| b == b',');

    let x_bytes = parts.next().unwrap();
    let x = parse_u32(x_bytes);

    let y_bytes = parts.next().unwrap();
    let y = parse_u32(y_bytes);

    Point::new(x, y)
}

fn parse_input(input: &str) -> Vec<Point> {
    input
        .as_bytes()
        .split(|&c| c == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| parse_point(line))
        .collect()
}

const fn parse_u32(bytes: &[u8]) -> u32 {
    let mut result = 0;
    let mut i = 0;

    while i < bytes.len() {
        let digit = bytes[i];
        result = result * 10 + (digit - b'0') as u32;
        i += 1;
    }

    result
}

struct PrefixSum2D {
    data: Vec<u64>,
    cols: usize,
}

impl PrefixSum2D {
    pub fn new(cells: &[Cell], cols: usize, rows: usize) -> Self {
        let mut data = vec![0u64; cols * rows];

        for y in 0..rows {
            for x in 0..cols {
                let idx = y * cols + x;

                // https://www.geeksforgeeks.org/dsa/prefix-sum-2d-array/
                let val = if cells[idx].is_valid() { 1 } else { 0 };

                let left = if x > 0 { data[idx - 1] } else { 0 };
                let above = if y > 0 { data[idx - cols] } else { 0 };
                let diag = if x > 0 && y > 0 { data[idx - cols - 1] } else { 0 };

                data[idx] = val + left + above - diag;
            }
        }

        Self { data, cols }
    }
    
    // use inclusion-exclusion to compute sum in rect
    #[inline]
    pub fn query_unchecked(&self, top_left: Point, bottom_right: Point) -> u64 {
        let (x1, y1) = (top_left.x as usize, top_left.y as usize);
        let (x2, y2) = (bottom_right.x as usize, bottom_right.y as usize);
        let cols = self.cols;

        unsafe {
            let total = *self.data.get_unchecked(y2 * cols + x2);
            let left = *self.data.get_unchecked(y2 * cols + (x1 - 1));
            let above = *self.data.get_unchecked((y1 - 1) * cols + x2);
            let diag = *self.data.get_unchecked((y1 - 1) * cols + (x1 - 1));
            (total + diag) - left - above
        }
    }

    #[inline]
    pub fn is_rect_valid(&self, rect: Rect) -> bool {
        let area = rect.width() as u64 * rect.height() as u64;
        self.query_unchecked(rect.top_left, rect.bottom_right) == area
    }
}

/// A 2D point with u32 coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: u32,
    y: u32,
}

impl From<(u32, u32)> for Point {
    fn from((x, y): (u32, u32)) -> Self {
        Self { x, y }
    }
}

impl Point {
    #[inline]
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn offset(self, dx: u32, dy: u32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rect {
    top_left: Point,
    bottom_right: Point,
}

impl Rect {
    // creates rect with proper orientation
    #[inline]
    pub fn from_corners(a: Point, b: Point) -> Self {
        Self {
            top_left: Point::new(a.x.min(b.x), a.y.min(b.y)),
            bottom_right: Point::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.bottom_right.x - self.top_left.x + 1
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.bottom_right.y - self.top_left.y + 1
    }

    #[inline]
    pub fn area(&self) -> usize {
        self.width() as usize * self.height() as usize
    }
}

struct CoordCompressor {
    x: Vec<u32>,
    y: Vec<u32>,
}

impl CoordCompressor {
    fn new(points: &[Point]) -> Self {
        let mut x: Vec<u32> = points.iter().map(|p| p.x).collect();
        let mut y: Vec<u32> = points.iter().map(|p| p.y).collect();

        x.sort_unstable();
        x.dedup();
        y.sort_unstable();
        y.dedup();

        Self { x, y }
    }

    #[inline]
    fn compress_x(&self, x: u32) -> u32 {
        self.x.binary_search(&x).unwrap() as u32
    }

    #[inline]
    fn compress_y(&self, y: u32) -> u32 {
        self.y.binary_search(&y).unwrap() as u32
    }

    #[inline]
    fn compress(&self, p: Point) -> Point {
        Point::new(self.compress_x(p.x), self.compress_y(p.y))
    }

    #[inline]
    fn cols(&self) -> usize {
        self.x.len()
    }

    #[inline]
    fn rows(&self) -> usize {
        self.y.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = "7,1\n11,1\n11,7\n9,7\n9,5\n2,5\n2,3\n7,3";

    #[test]
    fn solve_example() {
        assert_eq!(solve(EXAMPLE_INPUT), 24);
    }

    #[test]
    fn parse_input_works() {
        let result = parse_input(EXAMPLE_INPUT);

        assert_eq!(
            result,
            vec![
                Point::new(7, 1),
                Point::new(11, 1),
                Point::new(11, 7),
                Point::new(9, 7),
                Point::new(9, 5),
                Point::new(2, 5),
                Point::new(2, 3),
                Point::new(7, 3),
            ]
        );
    }

    #[test]
    fn parse_point_works() {
        assert_eq!(parse_point(b"12,34"), Point::new(12, 34));
    }

    #[test]
    fn grid_new_works() {
        let points = vec![
            Point::new(1, 2),
            Point::new(3, 4),
            Point::new(2, 5),
        ];

        let compressor = CoordCompressor::new(&points);
        let grid = Grid::new(&compressor);

        assert_eq!(grid.cols, 5);
        assert_eq!(grid.rows, 5);
    }

    mod point {
        use super::*;

        #[test]
        fn offset_adds_to_coordinates() {
            let point = Point::new(5, 10);
            assert_eq!(point.offset(3, 7), Point::new(8, 17));
        }

        #[test]
        fn from_tuple() {
            let point: Point = (5, 10).into();
            assert_eq!(point, Point::new(5, 10));
        }
    }

    mod rect {
        use super::*;

        #[test]
        fn from_corners_normalizes_order() {
            let r1 = Rect::from_corners(Point::new(1, 2), Point::new(5, 8));
            let r2 = Rect::from_corners(Point::new(5, 8), Point::new(1, 2));
            let r3 = Rect::from_corners(Point::new(1, 8), Point::new(5, 2));

            assert_eq!(r1, r2);
            assert_eq!(r2, r3);
            assert_eq!(r1.top_left, Point::new(1, 2));
            assert_eq!(r1.bottom_right, Point::new(5, 8));
        }

        #[test]
        fn dimensions() {
            let rect = Rect::from_corners(Point::new(2, 3), Point::new(6, 10));

            assert_eq!(rect.width(), 5);
            assert_eq!(rect.height(), 8);
            assert_eq!(rect.area(), 40);
        }

        #[test]
        fn single_point_rect() {
            let rect = Rect::from_corners(Point::new(5, 5), Point::new(5, 5));

            assert_eq!(rect.width(), 1);
            assert_eq!(rect.height(), 1);
            assert_eq!(rect.area(), 1);
        }
    }

    mod coord_compressor {
        use super::*;
        use rstest::rstest;

        fn compressor_from_coords(coords: &[(u32, u32)]) -> CoordCompressor {
            let points: Vec<Point> = coords.iter().map(|&(x, y)| Point::new(x, y)).collect();
            CoordCompressor::new(&points)
        }

        #[rstest]
        #[case(&[(100, 200), (300, 400), (200, 300)], 3, 3)]
        #[case(&[(5, 5), (5, 10), (5, 15)], 1, 3)]
        #[case(&[(1, 1), (2, 1), (3, 1)], 3, 1)]
        #[case(&[(42, 99)], 1, 1)]
        fn dimensions_equal_unique_coordinate_count(
            #[case] coords: &[(u32, u32)],
            #[case] expected_cols: usize,
            #[case] expected_rows: usize,
        ) {
            let compressor = compressor_from_coords(coords);

            assert_eq!(compressor.cols(), expected_cols);
            assert_eq!(compressor.rows(), expected_rows);
        }

        #[rstest]
        #[case(&[(100, 200), (300, 400), (200, 300)], 100, 0)]
        #[case(&[(100, 200), (300, 400), (200, 300)], 200, 1)]
        #[case(&[(100, 200), (300, 400), (200, 300)], 300, 2)]
        fn compress_x_maps_to_sorted_index(
            #[case] coords: &[(u32, u32)],
            #[case] original_x: u32,
            #[case] expected_compressed: u32,
        ) {
            let compressor = compressor_from_coords(coords);

            assert_eq!(compressor.compress_x(original_x), expected_compressed);
        }

        #[rstest]
        #[case(&[(100, 200), (300, 400), (200, 300)], 200, 0)]
        #[case(&[(100, 200), (300, 400), (200, 300)], 300, 1)]
        #[case(&[(100, 200), (300, 400), (200, 300)], 400, 2)]
        fn compress_y_maps_to_sorted_index(
            #[case] coords: &[(u32, u32)],
            #[case] original_y: u32,
            #[case] expected_compressed: u32,
        ) {
            let compressor = compressor_from_coords(coords);

            assert_eq!(compressor.compress_y(original_y), expected_compressed);
        }

        #[rstest]
        #[case((100, 200), (0, 0))]
        #[case((300, 400), (2, 2))]
        #[case((200, 300), (1, 1))]
        #[case((100, 400), (0, 2))]
        #[case((300, 200), (2, 0))]
        fn compress_maps_point_to_grid_indices(
            #[case] original: (u32, u32),
            #[case] expected: (u32, u32),
        ) {
            let compressor = compressor_from_coords(&[(100, 200), (300, 400), (200, 300)]);

            let result = compressor.compress(Point::new(original.0, original.1));

            assert_eq!(result, Point::new(expected.0, expected.1));
        }

        #[rstest]
        #[case(&[(1000, 1), (2000, 2), (3000, 3)])]
        #[case(&[(99999, 88888), (11111, 22222)])]
        fn large_coordinates_compress_to_small_indices(#[case] coords: &[(u32, u32)]) {
            let compressor = compressor_from_coords(coords);

            for &(x, y) in coords {
                let compressed = compressor.compress(Point::new(x, y));
                assert!(compressed.x < coords.len() as u32);
                assert!(compressed.y < coords.len() as u32);
            }
        }

        #[rstest]
        #[case(&[(5, 5), (5, 5), (5, 5)], 1, 1)]
        #[case(&[(1, 2), (1, 2), (3, 2)], 2, 1)]
        fn duplicate_points_are_deduplicated(
            #[case] coords: &[(u32, u32)],
            #[case] expected_cols: usize,
            #[case] expected_rows: usize,
        ) {
            let compressor = compressor_from_coords(coords);

            assert_eq!(compressor.cols(), expected_cols);
            assert_eq!(compressor.rows(), expected_rows);
        }
    }
}
