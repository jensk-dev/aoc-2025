pub fn solve(input: &str) -> usize {
    let points = parse_input(input);
    max_surface_area(&points) as usize
}

fn parse_u32vec2(s: &[u8]) -> Vec2u32 {
    let mut parts = s.split(|&b| b == b',');

    let x_bytes = parts.next().unwrap();
    let x = parse_u32(x_bytes);

    let y_bytes = parts.next().unwrap();
    let y = parse_u32(y_bytes);

    Vec2u32::new(x, y)
}

fn parse_input(input: &str) -> Vec<Vec2u32> {
    input
        .as_bytes()
        .split(|&c| c == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| parse_u32vec2(line))
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

fn max_surface_area(points: &[Vec2u32]) -> u64 {
    // N(N-1)/2. Brute force.

    let mut max_area = 0;
    let n = points.len();

    for i in 0..n {
        for j in (i + 1)..n {
            let area = points[i].surface_between(&points[j]);
            if area > max_area {
                max_area = area;
            }
        }
    }

    max_area
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vec2u32 {
    x: u32,
    y: u32,
}

impl From<(u32, u32)> for Vec2u32 {
    fn from(value: (u32, u32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Vec2u32 {
    #[inline]
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
        }
    }

    #[inline]
    pub fn surface_between(&self, other: &Self) -> u64 {
        let width = self.x.abs_diff(other.x) + 1;
        let height = self.y.abs_diff(other.y) + 1;

        width as u64 * height as u64
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const EXAMPLE_INPUT: &str = "7,1\n11,1\n11,7\n9,7\n9,5\n2,5\n2,3\n7,3";

    #[test]
    fn solve_example() {
        let input = EXAMPLE_INPUT;
        let result = solve(input);

        assert_eq!(result, 50);
    }

    #[test]
    fn parse_input_works() {
        let input = EXAMPLE_INPUT;
        let result = parse_input(input);

        assert_eq!(
            result,
            vec![
                Vec2u32::new(7, 1),
                Vec2u32::new(11, 1),
                Vec2u32::new(11, 7),
                Vec2u32::new(9, 7),
                Vec2u32::new(9, 5),
                Vec2u32::new(2, 5),
                Vec2u32::new(2, 3),
                Vec2u32::new(7, 3),
            ]
        );
    }

    #[test]
    fn parse_u32vec2_works() {
        let input = b"12,34";
        let result = parse_u32vec2(input);

        assert_eq!(result, Vec2u32::new(12, 34));
    }

    #[rstest]
    #[case((2,5).into(), (9,7).into(), 24)]
    #[case((7,1).into(), (11,7).into(), 35)]
    #[case((7,3).into(), (2,3).into(), 6)]
    #[case((2,5).into(), (11,1).into(), 50)]
    fn surface_between(#[case] a: Vec2u32, #[case] b: Vec2u32, #[case] surface: u64) {
        let result = a.surface_between(&b);
        assert_eq!(result, surface);
    }
}
