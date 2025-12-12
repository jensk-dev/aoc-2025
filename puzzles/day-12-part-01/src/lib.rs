const NUM_OF_CELLS: usize = 7;
const START_OFFSET: usize = 95;
const DIMS_OFFSET: usize = DIM_LEN + DIM_LEN + 2; // 12x34:
const DIM_LEN: usize = 2;

pub fn solve(input: &str) -> usize {
    let b = input.as_bytes();
    let mut i = START_OFFSET;
    let mut count = 0;

    while i < b.len() {
        if i + DIMS_OFFSET <= b.len() && b[i + DIM_LEN] == b'x' {
            let width = append(num(b[i]) as usize,  b[i + 1]);
            let height = append(
                num(b[i + 3]) as usize, b[i + 4]);
            i += DIMS_OFFSET;

            let mut total = 0;
            while i < b.len() && b[i] != b'\n' {
                if b[i] >= b'0' && b[i] <= b'9' {
                    let mut n = 0;
                    while i < b.len() && b[i] >= b'0' && b[i] <= b'9' {
                        n = append(n, b[i]);
                        i += 1;
                    }
                    total += n;
                } else {
                    i += 1;
                }
            }
            count += (total * NUM_OF_CELLS <= width * height) as usize;
        }
        while i < b.len() && b[i] != b'\n' {
            i += 1;
        }
        i += 1;
    }
    count
}

#[inline(always)]
const fn append(num: usize, byte: u8) -> usize {
    num * 10 + (byte - b'0') as usize
}

#[inline(always)]
const fn num(byte: u8) -> u8 {
    byte - b'0' 
}
