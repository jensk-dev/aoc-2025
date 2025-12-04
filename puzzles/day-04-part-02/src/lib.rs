use wide::{CmpEq, CmpGt, i8x32};

pub fn solve(input: &str) -> usize {
    let mut pile = PileOfPaperRolls::from_str(input.as_bytes());
    let params = ConvolutionParams {
        char: b'@',
        max_neighbours: 4,
    };
    pile.remove_all_accessible(&params)
}

#[derive(Clone, Copy)]
struct ConvolutionParams {
    char: u8,
    max_neighbours: usize,
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

    #[inline(always)]
    fn simd_kernel_convolution_remove(
        &mut self,
        x: usize,
        y: usize,
        params: &ConvolutionParams,
        validity_mask: u32,
    ) -> usize {
        let row_offset = |dy: usize| (y + dy - 1) * self.padded_width + x;
        let load = |grid: &[u8], offset: usize| {
            let slice: &[i8; 32] = grid[offset..offset + 32]
                .try_into()
                .map(|arr: &[u8; 32]| unsafe { std::mem::transmute(arr) })
                .unwrap();
            i8x32::from(*slice)
        };

        // pak de drie horizontale rijen voor het maken van de 3x3 kernel op basis van de center cel
        let above = row_offset(0);
        let center = row_offset(1);
        let below = row_offset(2);

        // laad de 3x3 kernel als SIMD vectoren
        // dit laadt dus 9x32 cellen tegelijk
        let neighbours = [
            load(&self.grid, above - 1),
            load(&self.grid, above),
            load(&self.grid, above + 1),
            load(&self.grid, center - 1),
            load(&self.grid, center + 1),
            load(&self.grid, below - 1),
            load(&self.grid, below),
            load(&self.grid, below + 1),
        ];
        let center_cells = load(&self.grid, center);

        // mask waarbij de hele vector de waarde van de char heeft
        let char_mask = i8x32::splat(params.char as i8);
        // mask waarbij de hele vector op 1 is gezet
        let one = i8x32::splat(1);

        // check hoeveel neighbours overeenkomen met value
        // en tel deze op door ze eerst te bitwise AND'en
        let neighbor_count = neighbours
            .into_iter()
            .map(|v| v.simd_eq(char_mask) & one)
            .reduce(|a, b| a + b)
            .unwrap();

        // check of het aantal neighbours onder de threshold ligt
        let max_val = i8x32::splat(params.max_neighbours as i8);
        let under_threshold = max_val.simd_gt(neighbor_count);

        // check of de center cel overeenkomt met de waarde
        let center_matches = center_cells.simd_eq(char_mask);

        // combineer de masks om zo de leidende mask te maken voor verwijdering
        let result = under_threshold & center_matches;
        let mut mask = result.to_bitmask() & validity_mask;
        let count = mask.count_ones() as usize;

        // trucje om de cellen te verwijderen die overeenkomen met de mask
        // aangezien we de center index nog hebben
        // kunnen we gewoon de offsets naar rechts gebruiken voor iedere
        // voorvoegnul die we tegenkomen in de mask
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            self.grid[center + i] = b'.';
            mask &= mask - 1;
        }

        count
    }

    fn remove_accessible(&mut self, params: &ConvolutionParams) -> usize {
        if self.width < Self::PROC_CELLS {
            return self.scalar_remove_accessible(params);
        }

        let mut count = 0;

        for y in 1..=self.height {
            let mut x = 1;

            while x + Self::PROC_CELLS <= self.width {
                count += self.simd_kernel_convolution_remove(x, y, params, Self::VALIDITY_CHECK);
                x += Self::PROC_CELLS;
            }

            if x <= self.width {
                let remaining = self.width - x + 1;
                let remainder_mask = (1u32 << remaining) - 1;
                count += self.simd_kernel_convolution_remove(x, y, params, remainder_mask);
            }
        }

        count
    }

    fn scalar_remove_accessible(&mut self, params: &ConvolutionParams) -> usize {
        let mut count = 0;
        let mut to_remove = Vec::new();

        for y in 1..=self.height {
            for x in 1..=self.width {
                let idx = y * self.padded_width + x;

                if self.grid[idx] != params.char {
                    continue;
                }

                let mut neighbours = 0;
                neighbours += (self.grid[idx - self.padded_width - 1] == params.char) as usize;
                neighbours += (self.grid[idx - self.padded_width] == params.char) as usize;
                neighbours += (self.grid[idx - self.padded_width + 1] == params.char) as usize;
                neighbours += (self.grid[idx - 1] == params.char) as usize;
                neighbours += (self.grid[idx + 1] == params.char) as usize;
                neighbours += (self.grid[idx + self.padded_width - 1] == params.char) as usize;
                neighbours += (self.grid[idx + self.padded_width] == params.char) as usize;
                neighbours += (self.grid[idx + self.padded_width + 1] == params.char) as usize;

                if neighbours < params.max_neighbours {
                    to_remove.push(idx);
                    count += 1;
                }
            }
        }

        for idx in to_remove {
            self.grid[idx] = b'.';
        }

        count
    }

    fn remove_all_accessible(&mut self, params: &ConvolutionParams) -> usize {
        let mut total = 0;
        loop {
            let removed = self.remove_accessible(params);
            if removed == 0 {
                break;
            }
            total += removed;
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;
    use rstest::rstest;

    const EXAMPLE: &str = "..@@.@@@@.\n@@@.@.@.@@\n@@@@@.@.@@\n@.@@@@..@.\n@@.@@@@.@@\n.@@@@@@@.@\n.@.@.@.@@@\n@.@@@.@@@@\n.@@@@@@@@.\n@.@.@@@.@.\n";

    #[rstest]
    #[case(1, 13)]
    #[case(2, 12)]
    #[case(3, 7)]
    #[case(4, 5)]
    #[case(5, 2)]
    #[case(6, 1)]
    #[case(7, 1)]
    #[case(8, 1)]
    #[case(9, 1)]
    fn removal_round(#[case] round: usize, #[case] expected_removed: usize) {
        let mut pile = PileOfPaperRolls::from_str(EXAMPLE.as_bytes());
        let params = ConvolutionParams {
            char: b'@',
            max_neighbours: 4,
        };

        for _ in 1..round {
            pile.remove_accessible(&params);
        }
        assert_eq!(pile.remove_accessible(&params), expected_removed);
    }

    #[rstest]
    #[case(EXAMPLE, 43)]
    fn remove_all(#[case] input: &str, #[case] expected_total: usize) {
        let mut pile = PileOfPaperRolls::from_str(input.as_bytes());
        let params = ConvolutionParams {
            char: b'@',
            max_neighbours: 4,
        };
        assert_eq!(pile.remove_all_accessible(&params), expected_total);
    }

    #[test]
    fn simd_matches_scalar_removal() {
        let input = read_to_string("input.txt").unwrap();
        let mut scalar_pile = PileOfPaperRolls::from_str(input.as_bytes());
        let mut simd_pile = PileOfPaperRolls::from_str(input.as_bytes());
        let params = ConvolutionParams {
            char: b'@',
            max_neighbours: 4,
        };

        let scalar_result = loop {
            let removed = scalar_pile.scalar_remove_accessible(&params);
            if removed == 0 {
                break 0;
            }
        };
        let simd_result = loop {
            let removed = simd_pile.remove_accessible(&params);
            if removed == 0 {
                break 0;
            }
        };
        assert_eq!(scalar_result, simd_result);
        assert_eq!(scalar_pile.grid, simd_pile.grid);
    }

    #[rstest]
    #[case("..@..\n.@@@.\n..@..\n", 5, 3, 32)]
    fn pile_from_str(
        #[case] input: &str,
        #[case] width: usize,
        #[case] height: usize,
        #[case] padded_width: usize,
    ) {
        let pile = PileOfPaperRolls::from_str(input.as_bytes());
        assert_eq!(pile.width, width);
        assert_eq!(pile.height, height);
        assert_eq!(pile.padded_width, padded_width);
    }
}
