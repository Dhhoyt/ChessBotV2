use std::collections::HashMap;
use std::fs;
use rand::Rng;

use super::{Board, BitBoard, utils::print_bit_board};

pub struct OpeningBook {
    pub moves: HashMap<u64, Vec<Move>>
}

impl OpeningBook {
    pub fn new(bytes: Vec<u8>) -> Self {
        let mut moves: HashMap<u64, Vec<Move>> = HashMap::new();
        for i in bytes.chunks(16) {
            let key = u64::from_be_bytes(i[0..8].try_into().unwrap());
            let book_move = u16::from_be_bytes(i[8..10].try_into().unwrap());
            let to_file = book_move & 0x7 >> 0;
            let to_row = book_move & 0x38 >> 3;
            let from_file = book_move & 0x1c0 >> 6;
            let from_row = book_move & 0xe00 >> 9;
            
            let weight = u16::from_be_bytes(i[10..12].try_into().unwrap());
            let full_move = Move { to_square: book_move & 0x3f, from_square: (book_move & 0xfc0) >> 6, promotion_piece: PromotionPiece::None, weight: weight};
            match moves.get_mut(&key) {
                Some(value) => value.push(full_move),
                None => {let move_vec = vec![full_move]; moves.insert(key, move_vec);},
            };
        }
        OpeningBook { moves: moves }
    }

    pub fn get_move(&self, key: u64) -> Option<Move>{
        match self.moves.get(&key) {
            Some(value)=>Some(pick_random(value)),
            None=> None
        }
    }
}

impl Board {
    pub fn make_move(self, book_move: Move) -> Board {
        let mut res = self.clone();
        let from_mask = (1 as BitBoard) << book_move.from_square;
        let to_mask = (1 as BitBoard) << book_move.to_square;
        if res.white_pawns & from_mask != 0 {
            res.white_pawns ^= from_mask | to_mask;
            res.capture_black(to_mask);
        }
        else if res.white_knights & from_mask != 0 {
            res.white_knights ^= from_mask | to_mask;
            res.capture_black(to_mask);
        }
        else if res.white_bishops & from_mask != 0 {
            res.white_bishops ^= from_mask | to_mask;
            res.capture_black(to_mask);
        }
        else if res.white_rooks & from_mask != 0 {
            res.white_rooks ^= from_mask | to_mask;
            res.capture_black(to_mask);
        }
        else if res.white_queens & from_mask != 0 {
            res.white_queens ^= from_mask | to_mask;
            res.capture_black(to_mask);
        }
        else if res.white_kings & from_mask != 0 {
            if from_mask | to_mask == 0x90 {
                res.white_kings ^= 0x50;
                res.white_rooks ^= 0xa0;
                res.castle &= !0xff;
            } else if from_mask | to_mask == 0x11 {
                res.white_kings ^= 0x14;
                res.white_rooks ^= 0x9;
                res.castle &= !0xff;
            } else {
                res.white_kings ^= from_mask | to_mask;
                res.capture_black(to_mask);
            }
        }
        else if res.black_pawns & from_mask != 0 {
            res.black_pawns ^= from_mask | to_mask;
            res.capture_white(to_mask);
        }
        else if res.black_knights & from_mask != 0 {
            res.black_knights ^= from_mask | to_mask;
            res.capture_white(to_mask);
        }
        else if res.black_bishops & from_mask != 0 {
            res.black_bishops ^= from_mask | to_mask;
            res.capture_white(to_mask);
        }
        else if res.black_rooks & from_mask != 0 {
            res.black_rooks ^= from_mask | to_mask;
            res.capture_white(to_mask);
        }
        else if res.black_queens & from_mask != 0 {
            res.black_queens ^= from_mask | to_mask;
            res.capture_white(to_mask);
        }
        else if res.black_kings & from_mask != 0 {
            if from_mask | to_mask == 0x9000000000000000 {
                res.black_kings ^= 0x5000000000000000;
                res.black_rooks ^= 0xa000000000000000;
                res.castle &= !0xff00000000000000;
            } else if from_mask | to_mask == 0x1100000000000000 {
                res.black_kings ^= 0x1400000000000000;
                res.black_rooks ^= 0x900000000000000;
                res.castle &= !0xff00000000000000;
            } else {
                res.black_kings ^= from_mask | to_mask;
                res.capture_white(to_mask);
            }
        }
        res.redo_occupied();
        res.white_to_play = ! res.white_to_play;
        res
    }
}

fn pick_random(moves: &Vec<Move>) -> Move {
    let sum = moves.iter().fold(0, |acc, x| acc + x.weight as usize);
    let mut random = rand::thread_rng().gen_range(1..sum + 1);
    for i in moves {
        if random <= i.weight as usize {
            return i.clone();
        }
        random -= i.weight as usize;
    }
    panic!("");
}

#[derive(Clone, Copy)]
pub struct Move {
    pub to_square: u16,
    pub from_square: u16,
    pub promotion_piece: PromotionPiece,
    pub weight: u16,
}

#[derive(Clone, Copy)]
pub enum PromotionPiece {
    None,
    Knight,
    Bishop,
    Rook,
    Queen
}

impl PromotionPiece {
    fn from_num(piece: u8) -> Self {
        match piece {
            0 => Self::None,
            1 => Self::Knight,
            2 => Self::Bishop,
            3 => Self::Rook,
            4 => Self::Queen,
            _ => panic!("Invalid piece")
        }
    }
}