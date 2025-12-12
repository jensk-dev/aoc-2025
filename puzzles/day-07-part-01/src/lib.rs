pub fn solve(input: &str) -> usize {
    let manifold = TachyonManifold::from_slice(input.as_bytes());

    manifold.trace_beam()
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x: u32,
    y: u32,
}

impl Position {
    fn move_down(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
        }
    }

    fn move_left(&self) -> Self {
        Self {
            x: self.x.wrapping_sub(1),
            y: self.y,
        }
    }

    fn move_right(&self) -> Self {
        Self {
            x: self.x.wrapping_add(1),
            y: self.y,
        }
    }
}

struct TachyonManifold<'a> {
    data: &'a [u8],
    width: u32,
    stride: u32,
    height: u32,
}

impl<'a> TachyonManifold<'a> {
    fn from_slice(input: &'a [u8]) -> Self {
        let width = input
            .iter()
            .position(|&c| c == b'\n')
            .unwrap_or(input.len()) as u32;
        let stride = width + 1_u32;
        let height = (input.len() + 1) as u32 / stride;

        Self {
            data: input,
            width,
            stride,
            height,
        }
    }

    fn starting_position(&self) -> Position {
        Position {
            x: self.width / 2,
            y: 0,
        }
    }

    fn trace_beam(self) -> usize {
        use std::collections::VecDeque;

        let mut split_count = 0;
        let mut visited = vec![false; self.data.len()];
        let mut position_queue = VecDeque::with_capacity(1024);
        position_queue.push_back(self.starting_position().move_down());

        while let Some(current_position) = position_queue.pop_front() {
            if current_position.y >= self.height || current_position.x >= self.width {
                continue;
            }

            let index = self.idx(&current_position);
            if visited[index] {
                continue;
            }
            visited[index] = true;

            let current_byte = self.get(&current_position);

            if Self::should_split(current_byte) {
                split_count += 1;
                position_queue.push_back(current_position.move_left());
                position_queue.push_back(current_position.move_right());
            } else {
                position_queue.push_back(current_position.move_down());
            }
        }

        split_count
    }

    #[inline]
    fn idx(&self, position: &Position) -> usize {
        (position.y * self.stride + position.x) as usize
    }

    #[inline]
    fn get(&self, position: &Position) -> u8 {
        let index = self.idx(position);
        unsafe { *self.data.get_unchecked(index) }
    }

    #[inline]
    const fn should_split(next_byte: u8) -> bool {
        next_byte == b'^'
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn solve_example() {
        let input = std::fs::read_to_string("example.txt").unwrap();
        let result = super::solve(&input);
        assert_eq!(result, 21);
    }

    mod position {
        use super::super::Position;

        #[test]
        fn move_down() {
            let pos = Position { x: 5, y: 5 };
            let new_pos = pos.move_down();
            assert_eq!(new_pos.x, 5);
            assert_eq!(new_pos.y, 6);
        }

        #[test]
        fn move_left() {
            let pos = Position { x: 5, y: 5 };
            let new_pos = pos.move_left();
            assert_eq!(new_pos.x, 4);
            assert_eq!(new_pos.y, 5);
        }

        #[test]
        fn move_right() {
            let pos = Position { x: 5, y: 5 };
            let new_pos = pos.move_right();
            assert_eq!(new_pos.x, 6);
            assert_eq!(new_pos.y, 5);
        }
    }

    mod tachyon_manifold {
        use std::fs::read_to_string;

        use rstest::rstest;

        #[test]
        fn from_slice() {
            let input = read_to_string("example.txt").unwrap();
            let manifold = crate::TachyonManifold::from_slice(input.as_bytes());

            assert_eq!(manifold.width, 15);
            assert_eq!(manifold.height, 16);
            assert_eq!(manifold.stride, 16);
        }

        #[rstest]
        #[case(0, 0, b'.')]
        #[case(7, 2, b'^')]
        #[case(7, 0, b'S')]
        #[case(14, 15, b'.')]
        fn get(#[case] x: u32, #[case] y: u32, #[case] expected: u8) {
            let input = read_to_string("example.txt").unwrap();
            let manifold = crate::TachyonManifold::from_slice(input.as_bytes());
            let position = crate::Position { x, y };
            let byte = manifold.get(&position);

            assert_eq!(byte, expected);
        }

        #[rstest]
        #[case(0, 0, 0)]
        #[case(7, 2, 39)]
        #[case(7, 0, 7)]
        #[case(14, 15, 254)]
        fn idx(#[case] x: u32, #[case] y: u32, #[case] expected: usize) {
            let input = read_to_string("example.txt").unwrap();
            let manifold = crate::TachyonManifold::from_slice(input.as_bytes());
            let position = crate::Position { x, y };
            let index = manifold.idx(&position);

            assert_eq!(index, expected);
        }

        #[test]
        fn should_split() {
            assert!(crate::TachyonManifold::should_split(b'^'));
            assert!(!crate::TachyonManifold::should_split(b'.'));
            assert!(!crate::TachyonManifold::should_split(b'S'));
        }
    }
}
