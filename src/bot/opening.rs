use rand::Rng;
use std::collections::HashMap;
use std::fs;

use super::{
    board_move::{Move, PromotionPiece},
    utils::print_bit_board,
    BitBoard, Board,
};

pub struct OpeningBook {
    pub moves: HashMap<u64, Vec<Move>>,
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
            let full_move = Move {
                to_square: book_move & 0x3f,
                from_square: (book_move & 0xfc0) >> 6,
                promotion_piece: PromotionPiece::None,
                weight: weight,
            };
            match moves.get_mut(&key) {
                Some(value) => value.push(full_move),
                None => {
                    let move_vec = vec![full_move];
                    moves.insert(key, move_vec);
                }
            };
        }
        OpeningBook { moves: moves }
    }

    pub fn get_move(&self, key: u64) -> Option<Move> {
        match self.moves.get(&key) {
            Some(value) => Some(pick_random(value)),
            None => None,
        }
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
