#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn solve(input: &str) -> usize {
    let pile = PileOfPaperRolls::from_str(input.as_bytes());
    unsafe { pile.simd_convolution(b'@', b'@', 4) }
}

struct PileOfPaperRolls {
    grid: Vec<u8>,
    width: usize,
    height: usize,
    padded_width: usize,
}

impl PileOfPaperRolls {
    const SIMD_WIDTH: usize = 32;
    const PROC_CELLS: usize = Self::SIMD_WIDTH - 2;
    const VALIDITY_CHECK: u32 = 0x3FFF_FFFF;

    fn from_str(grid: &[u8]) -> Self {
        let width = grid.iter().position(|&b| b == b'\n').unwrap_or(grid.len());
        let stride = width + 2;
        let padded_width = (stride + Self::SIMD_WIDTH - 1) & !(Self::SIMD_WIDTH - 1);
        let height = grid.len() / (width + 1);

        let mut data = vec![b'.'; padded_width * (height + 2)];

        for (y, line) in grid.split(|&b| b == b'\n').enumerate() {
            if line.is_empty() {
                break;
            }
            let start = (y + 1) * padded_width + 1;
            data[start..start + width].copy_from_slice(line);
        }

        Self {
            grid: data,
            width,
            height,
            padded_width,
        }
    }

    fn scalar_convolution(
        &self,
        center_value: u8,
        neighbour_value: u8,
        max_neighbours: usize,
    ) -> usize {
        let mut count = 0;

        for y in 1..=self.height {
            for x in 1..=self.width {
                let idx = y * self.padded_width + x;

                // Check center cell
                if self.grid[idx] != center_value {
                    continue;
                }

                // Count matching neighbors
                let mut neighbors = 0;
                neighbors += (self.grid[idx - self.padded_width - 1] == neighbour_value) as usize;
                neighbors += (self.grid[idx - self.padded_width] == neighbour_value) as usize;
                neighbors += (self.grid[idx - self.padded_width + 1] == neighbour_value) as usize;
                neighbors += (self.grid[idx - 1] == neighbour_value) as usize;
                neighbors += (self.grid[idx + 1] == neighbour_value) as usize;
                neighbors += (self.grid[idx + self.padded_width - 1] == neighbour_value) as usize;
                neighbors += (self.grid[idx + self.padded_width] == neighbour_value) as usize;
                neighbors += (self.grid[idx + self.padded_width + 1] == neighbour_value) as usize;

                if neighbors < max_neighbours {
                    count += 1;
                }
            }
        }

        count
    }

    #[inline(always)]
    unsafe fn simd_kernel(
        ptr: *const u8,
        y: usize,
        x: usize,
        padded_width: usize,
        center_value: u8,
        neighbour_value: u8,
        max_neighbours: usize,
        validity_mask: u32,
    ) -> usize {
        let row = |dy: usize| (y + dy - 1) * padded_width + x;

        let load = |offset: usize| unsafe { _mm256_loadu_si256(ptr.add(offset) as *const __m256i) };

        let above_left = load(row(0) - 1);
        let above_center = load(row(0));
        let above_right = load(row(0) + 1);
        let center_left = load(row(1) - 1);
        let center_mid = load(row(1));
        let center_right = load(row(1) + 1);
        let below_left = load(row(2) - 1);
        let below_center = load(row(2));
        let below_right = load(row(2) + 1);

        let neighbour_mask = unsafe { _mm256_set1_epi8(neighbour_value as i8) };
        let one = unsafe { _mm256_set1_epi8(1) };

        let count_if_match =
            |v: __m256i| unsafe { _mm256_and_si256(_mm256_cmpeq_epi8(v, neighbour_mask), one) };

        let sum = [
            above_left,
            above_center,
            above_right,
            center_left,
            center_right,
            below_left,
            below_center,
            below_right,
        ]
        .into_iter()
        .map(count_if_match)
        .reduce(|a, b| unsafe { _mm256_add_epi8(a, b) })
        .unwrap();

        let max_mask = unsafe { _mm256_set1_epi8(max_neighbours as i8) };
        let under_threshold = unsafe { _mm256_cmpgt_epi8(max_mask, sum) };
        let center_mask = unsafe { _mm256_set1_epi8(center_value as i8) };
        let center_matches = unsafe { _mm256_cmpeq_epi8(center_mid, center_mask) };

        let result = unsafe { _mm256_and_si256(under_threshold, center_matches) };
        let mask = unsafe { _mm256_movemask_epi8(result) } as u32;

        (mask & validity_mask).count_ones() as usize
    }

    unsafe fn simd_convolution(
        &self,
        center_value: u8,
        neighbour_value: u8,
        max_neighbours: usize,
    ) -> usize {
        if self.width < Self::PROC_CELLS {
            return self.scalar_convolution(center_value, neighbour_value, max_neighbours);
        }

        let ptr = self.grid.as_ptr();
        let mut count = 0;

        for y in 1..=self.height {
            let mut x = 1;

            while x + Self::PROC_CELLS <= self.width {
                count += unsafe {
                    Self::simd_kernel(
                        ptr,
                        y,
                        x,
                        self.padded_width,
                        center_value,
                        neighbour_value,
                        max_neighbours,
                        Self::VALIDITY_CHECK,
                    )
                };
                x += Self::PROC_CELLS;
            }

            if x <= self.width {
                let remaining = self.width - x + 1;
                let remainder_mask = (1u32 << remaining) - 1;
                count += unsafe {
                    Self::simd_kernel(
                        ptr,
                        y,
                        x,
                        self.padded_width,
                        center_value,
                        neighbour_value,
                        max_neighbours,
                        remainder_mask,
                    )
                };
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_example() {
        let input = "..@@.@@@@.\n@@@.@.@.@@\n@@@@@.@.@@\n@.@@@@..@.\n@@.@@@@.@@\n.@@@@@@@.@\n.@.@.@.@@@\n@.@@@.@@@@\n.@@@@@@@@.\n@.@.@@@.@.\n";
        let expected = 13;

        let pile = PileOfPaperRolls::from_str(input.as_bytes());
        let result = pile.scalar_convolution(b'@', b'@', 4);
        assert_eq!(result, expected);
    }

    #[test]
    fn pile_from_str() {
        let input = "..@..\n.@@@.\n..@..\n";
        let pile = PileOfPaperRolls::from_str(input.as_bytes());
        assert_eq!(pile.width, 5);
        assert_eq!(pile.height, 3);
        assert_eq!(pile.padded_width, 32);
    }
}
