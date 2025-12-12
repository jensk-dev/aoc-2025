#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn solve(input: &str) -> usize {
    let pile = PileOfPaperRolls::from_str(input.as_bytes());
    pile.count_accessible_rolls()
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
    const VALIDITY_CHECK: u32 = 0x3FFFFFFF; // bits 0-29 (30 cells); bit 30-31 niet meegenomen voor rechter boundary

    fn from_str(grid: &[u8]) -> Self {
        let width = grid.iter().position(|&b| b == b'\n').unwrap_or(grid.len());

        let stride = width + 2;

        let padded_width = (stride + Self::SIMD_WIDTH - 1) & !(Self::SIMD_WIDTH - 1);

        let height = grid.len() / (width + 1); // +1 for newline
        let padded_height = height + 2;

        let mut data = vec![b'.'; padded_width * padded_height];

        for (y, line) in grid.split(|&b| b == b'\n').enumerate() {
            // if eof line:
            if line.is_empty() {
                break;
            }
            let start = (y + 1) * padded_width + 1; // +1 to skip the left and top padding
            let _ = &mut data[start..start + width].copy_from_slice(line);
        }

        PileOfPaperRolls {
            grid: data,
            width,
            height,
            padded_width,
        }
    }

    // Only rolls that have fewer than 4 adjacent '@'s are considered accessible
    fn count_accessible_rolls(&self) -> usize {
        unsafe { self.simd_convolution(b'@', b'@', 4) }
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

    unsafe fn simd_convolution(
        &self,
        center_value: u8,
        neighbour_value: u8,
        max_neighbours: usize,
    ) -> usize {
        if self.width < Self::PROC_CELLS {
            return self.scalar_convolution(center_value, neighbour_value, max_neighbours);
        }

        let mut count = 0;

        for y in 1..=self.height {
            let mut x = 1;
            while x + Self::PROC_CELLS <= self.width {
                unsafe {
                    let ptr = self.grid.as_ptr();

                    // todo: kijken of ik _mm256_slli_si256 en _mm256_srli_si256 kan gebruiken om te byte shiften in plaats van 3 loads te doen per rij

                    // laad de registers uit de grid
                    let above_left = _mm256_loadu_si256(
                        ptr.add((y - 1) * self.padded_width + x - 1) as *const __m256i,
                    );
                    let above_center = _mm256_loadu_si256(
                        ptr.add((y - 1) * self.padded_width + x) as *const __m256i
                    );
                    let above_right = _mm256_loadu_si256(
                        ptr.add((y - 1) * self.padded_width + x + 1) as *const __m256i,
                    );
                    let center_left = _mm256_loadu_si256(
                        ptr.add(y * self.padded_width + x - 1) as *const __m256i
                    );
                    let center_mid =
                        _mm256_loadu_si256(ptr.add(y * self.padded_width + x) as *const __m256i);
                    let center_right = _mm256_loadu_si256(
                        ptr.add(y * self.padded_width + x + 1) as *const __m256i
                    );
                    let below_left = _mm256_loadu_si256(
                        ptr.add((y + 1) * self.padded_width + x - 1) as *const __m256i,
                    );
                    let below_center = _mm256_loadu_si256(
                        ptr.add((y + 1) * self.padded_width + x) as *const __m256i
                    );
                    let below_right = _mm256_loadu_si256(
                        ptr.add((y + 1) * self.padded_width + x + 1) as *const __m256i,
                    );

                    // dit stelt een register met 16 bytes in op de waarde van neighbour_value
                    let neighbour_mask = _mm256_set1_epi8(neighbour_value as i8);

                    // vergelijk
                    let above_left_eq = _mm256_cmpeq_epi8(above_left, neighbour_mask);
                    let above_center_eq = _mm256_cmpeq_epi8(above_center, neighbour_mask);
                    let above_right_eq = _mm256_cmpeq_epi8(above_right, neighbour_mask);
                    let center_left_eq = _mm256_cmpeq_epi8(center_left, neighbour_mask);
                    let center_right_eq = _mm256_cmpeq_epi8(center_right, neighbour_mask);
                    let below_left_eq = _mm256_cmpeq_epi8(below_left, neighbour_mask);
                    let below_center_eq = _mm256_cmpeq_epi8(below_center, neighbour_mask);
                    let below_right_eq = _mm256_cmpeq_epi8(below_right, neighbour_mask);

                    // maak een mask met alle bits op 1
                    let one_mask = _mm256_set1_epi8(1);

                    // gebruik bitwise and om de bits te pakken
                    // nu hebben de we hoeveelheid neighbours per cel
                    let above_left_bit = _mm256_and_si256(above_left_eq, one_mask);
                    let above_center_bit = _mm256_and_si256(above_center_eq, one_mask);
                    let above_right_bit = _mm256_and_si256(above_right_eq, one_mask);
                    let center_left_bit = _mm256_and_si256(center_left_eq, one_mask);
                    let center_right_bit = _mm256_and_si256(center_right_eq, one_mask);
                    let below_left_bit = _mm256_and_si256(below_left_eq, one_mask);
                    let below_center_bit = _mm256_and_si256(below_center_eq, one_mask);
                    let below_right_bit = _mm256_and_si256(below_right_eq, one_mask);

                    // tel alles op
                    let sum = _mm256_add_epi8(above_left_bit, above_center_bit);
                    let sum = _mm256_add_epi8(sum, above_right_bit);
                    let sum = _mm256_add_epi8(sum, center_left_bit);
                    let sum = _mm256_add_epi8(sum, center_right_bit);
                    let sum = _mm256_add_epi8(sum, below_left_bit);
                    let sum = _mm256_add_epi8(sum, below_center_bit);
                    let sum = _mm256_add_epi8(sum, below_right_bit);

                    // register met de max waarde
                    let max_neighbours_mask = _mm256_set1_epi8(max_neighbours as i8);
                    let neighbours_threshold = _mm256_cmpgt_epi8(max_neighbours_mask, sum); // max_neighbours > sum van neighbours

                    // check de center cel zelf ook op de juiste waarde
                    let center_mask = _mm256_set1_epi8(center_value as i8);
                    let center_matches = _mm256_cmpeq_epi8(center_mid, center_mask);

                    // combineer de masks via bitwise and
                    let result = _mm256_and_si256(neighbours_threshold, center_matches);
                    let mask = _mm256_movemask_epi8(result) as u32;
                    let matches = (mask & Self::VALIDITY_CHECK).count_ones() as usize;
                    count += matches;
                }

                x += Self::PROC_CELLS;
            }

            if x <= self.width {
                // masked load because we might have less than PROC_CELLS remaining

                let remaining = self.width - x + 1;
                let final_x = x;

                unsafe {
                    let ptr = self.grid.as_ptr();

                    // todo: kijken of ik _mm256_slli_si256 en _mm256_srli_si256 kan gebruiken om te byte shiften in plaats van 3 loads te doen per rij

                    // laad de registers uit de grid
                    let above_left = _mm256_loadu_si256(
                        ptr.add((y - 1) * self.padded_width + final_x - 1) as *const __m256i,
                    );
                    let above_center = _mm256_loadu_si256(
                        ptr.add((y - 1) * self.padded_width + final_x) as *const __m256i,
                    );
                    let above_right = _mm256_loadu_si256(
                        ptr.add((y - 1) * self.padded_width + final_x + 1) as *const __m256i,
                    );
                    let center_left = _mm256_loadu_si256(
                        ptr.add(y * self.padded_width + final_x - 1) as *const __m256i,
                    );
                    let center_mid = _mm256_loadu_si256(
                        ptr.add(y * self.padded_width + final_x) as *const __m256i
                    );
                    let center_right = _mm256_loadu_si256(
                        ptr.add(y * self.padded_width + final_x + 1) as *const __m256i,
                    );
                    let below_left = _mm256_loadu_si256(
                        ptr.add((y + 1) * self.padded_width + final_x - 1) as *const __m256i,
                    );
                    let below_center = _mm256_loadu_si256(
                        ptr.add((y + 1) * self.padded_width + final_x) as *const __m256i,
                    );
                    let below_right = _mm256_loadu_si256(
                        ptr.add((y + 1) * self.padded_width + final_x + 1) as *const __m256i,
                    );

                    // dit stelt een register met 16 bytes in op de waarde van neighbour_value
                    let neighbour_mask = _mm256_set1_epi8(neighbour_value as i8);

                    // vergelijk
                    let above_left_eq = _mm256_cmpeq_epi8(above_left, neighbour_mask);
                    let above_center_eq = _mm256_cmpeq_epi8(above_center, neighbour_mask);
                    let above_right_eq = _mm256_cmpeq_epi8(above_right, neighbour_mask);
                    let center_left_eq = _mm256_cmpeq_epi8(center_left, neighbour_mask);
                    let center_right_eq = _mm256_cmpeq_epi8(center_right, neighbour_mask);
                    let below_left_eq = _mm256_cmpeq_epi8(below_left, neighbour_mask);
                    let below_center_eq = _mm256_cmpeq_epi8(below_center, neighbour_mask);
                    let below_right_eq = _mm256_cmpeq_epi8(below_right, neighbour_mask);

                    // maak een mask met alle bits op 1
                    let one_mask = _mm256_set1_epi8(1);

                    // gebruik bitwise and om de bits te pakken
                    // nu hebben de we hoeveelheid neighbours per cel
                    let above_left_bit = _mm256_and_si256(above_left_eq, one_mask);
                    let above_center_bit = _mm256_and_si256(above_center_eq, one_mask);
                    let above_right_bit = _mm256_and_si256(above_right_eq, one_mask);
                    let center_left_bit = _mm256_and_si256(center_left_eq, one_mask);
                    let center_right_bit = _mm256_and_si256(center_right_eq, one_mask);
                    let below_left_bit = _mm256_and_si256(below_left_eq, one_mask);
                    let below_center_bit = _mm256_and_si256(below_center_eq, one_mask);
                    let below_right_bit = _mm256_and_si256(below_right_eq, one_mask);

                    // tel alles op
                    let sum = _mm256_add_epi8(above_left_bit, above_center_bit);
                    let sum = _mm256_add_epi8(sum, above_right_bit);
                    let sum = _mm256_add_epi8(sum, center_left_bit);
                    let sum = _mm256_add_epi8(sum, center_right_bit);
                    let sum = _mm256_add_epi8(sum, below_left_bit);
                    let sum = _mm256_add_epi8(sum, below_center_bit);
                    let sum = _mm256_add_epi8(sum, below_right_bit);

                    // register met de max waarde
                    let max_neighbours_mask = _mm256_set1_epi8(max_neighbours as i8);
                    let neighbours_threshold = _mm256_cmpgt_epi8(max_neighbours_mask, sum); // max_neighbours > sum van neighbours

                    // check de center cel zelf ook op de juiste waarde
                    let center_mask = _mm256_set1_epi8(center_value as i8);
                    let center_matches = _mm256_cmpeq_epi8(center_mid, center_mask);

                    // combineer de masks via bitwise and
                    let result = _mm256_and_si256(neighbours_threshold, center_matches);
                    let mask = _mm256_movemask_epi8(result) as u32;
                    let remainder_mask = (1u32 << remaining) - 1;
                    let matches = (mask & remainder_mask).count_ones() as usize;

                    count += matches;
                }
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
