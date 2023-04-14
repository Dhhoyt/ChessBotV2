const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

#[inline]
pub const fn east_one(set: u64) -> u64 {
    (set << 1) & NOT_A_FILE
}

#[inline]
pub const fn west_one(set: u64) -> u64 {
    (set >> 1) & NOT_H_FILE
}

#[inline]
pub const fn north_one(set: u64) -> u64 {
    set << 8
}

#[inline]
pub const fn north_east_one(set: u64) -> u64 {
    (set << 9) & NOT_A_FILE
}

#[inline]
pub const fn north_west_one(set: u64) -> u64 {
    (set << 7) & NOT_H_FILE
}

#[inline]
pub const fn south_one(set: u64) -> u64 {
    set >> 8
}

#[inline]
pub const fn south_east_one(set: u64) -> u64 {
    (set >> 7) & NOT_A_FILE
}

#[inline]
pub const fn south_west_one(set: u64) -> u64 {
    (set >> 9) & NOT_H_FILE
}

pub fn print_bit_board(board: u64) {
    let mut bytes = board.to_ne_bytes();
    bytes.reverse();
    for i in bytes {
        let s = format!("{:#010b}", i);
        let s: String = s[2..s.len()].chars().rev().collect();
        println!("{}", s.replace("0", "."));
    }
    println!();
}

pub struct BitBoardIter(pub u64);

impl Iterator for BitBoardIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let res = self.0.trailing_zeros() as usize;
        self.0 &= !(1 << self.0.trailing_zeros());
        Some(res)
    }
}
