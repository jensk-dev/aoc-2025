use wide::{CmpEq, u8x32};

pub fn solve(input: &str) -> usize {
    let data = input.as_bytes();
    let width = data.iter().position(|&c| c == b'\n').unwrap_or(data.len());
    let stride = width + 1;
    let height = (data.len() + 1) / stride;

    // beams[i+1] corresponds to column i (padding for bounds)
    let mut beams = vec![0u64; width + 2];
    let center = width / 2 + 1;
    beams[center] = 1;

    let needle = u8x32::splat(b'^');
    let beams_ptr = beams.as_mut_ptr();

    // Process rows with step 2, starting from row 2
    let mut row_ptr = unsafe { data.as_ptr().add(2 * stride) };
    let row_step = 2 * stride;

    for _ in (2..height).step_by(2) {
        // Process full row with SIMD - width is 141, so 4 chunks of 32 + 13 remainder
        let mut x = 0;

        // Unrolled SIMD loop for 128 bytes (4 chunks)
        while x + 128 <= width {
            // Process 4 chunks in sequence
            for chunk_offset in [0, 32, 64, 96] {
                let chunk =
                    u8x32::new(unsafe { *(row_ptr.add(x + chunk_offset) as *const [u8; 32]) });
                let mut mask = chunk.simd_eq(needle).to_bitmask();

                while mask != 0 {
                    let bit_pos = mask.trailing_zeros() as usize;
                    let beam_pos = x + chunk_offset + bit_pos + 1;

                    unsafe {
                        let count = *beams_ptr.add(beam_pos);
                        *beams_ptr.add(beam_pos - 1) += count;
                        *beams_ptr.add(beam_pos) = 0;
                        *beams_ptr.add(beam_pos + 1) += count;
                    }

                    mask &= mask - 1;
                }
            }
            x += 128;
        }

        // Handle remaining chunks
        while x + 32 <= width {
            let chunk = u8x32::new(unsafe { *(row_ptr.add(x) as *const [u8; 32]) });
            let mut mask = chunk.simd_eq(needle).to_bitmask();

            while mask != 0 {
                let bit_pos = mask.trailing_zeros() as usize;
                let beam_pos = x + bit_pos + 1;

                unsafe {
                    let count = *beams_ptr.add(beam_pos);
                    *beams_ptr.add(beam_pos - 1) += count;
                    *beams_ptr.add(beam_pos) = 0;
                    *beams_ptr.add(beam_pos + 1) += count;
                }

                mask &= mask - 1;
            }
            x += 32;
        }

        // Handle remainder with SIMD (mask out invalid bits)
        if x < width {
            let chunk = u8x32::new(unsafe { *(row_ptr.add(x) as *const [u8; 32]) });
            let mut mask = chunk.simd_eq(needle).to_bitmask();
            // Mask out bits beyond width
            let valid_bits = width - x;
            mask &= (1u32 << valid_bits) - 1;

            while mask != 0 {
                let bit_pos = mask.trailing_zeros() as usize;
                let beam_pos = x + bit_pos + 1;

                unsafe {
                    let count = *beams_ptr.add(beam_pos);
                    *beams_ptr.add(beam_pos - 1) += count;
                    *beams_ptr.add(beam_pos) = 0;
                    *beams_ptr.add(beam_pos + 1) += count;
                }

                mask &= mask - 1;
            }
        }

        row_ptr = unsafe { row_ptr.add(row_step) };
    }

    beams.iter().sum::<u64>() as usize
}

#[cfg(test)]
mod tests {
    #[test]
    fn solve_example() {
        let input = std::fs::read_to_string("example.txt").unwrap();
        let result = super::solve(&input);
        assert_eq!(result, 40);
    }
}
