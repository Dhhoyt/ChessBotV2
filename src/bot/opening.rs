use std::collections::HashMap;
use std::fs;

pub struct OpeningBook {
    pub moves: HashMap<u64, Vec<Move>>
}

impl OpeningBook {
    pub fn new(path: &str) -> Self {
        let bytes = fs::read(path).unwrap();
        let mut moves: HashMap<u64, Vec<Move>> = HashMap::new();
        for i in bytes.chunks(16) {
            let key = u64::from_be_bytes(i[0..8].try_into().unwrap());
            let book_move = &i[8..10];
            let weight = u16::from_be_bytes(i[10..12].try_into().unwrap());
            let full_move = Move {to_file: 0, to_row: 0, from_file: 0, from_row: 0, promotion_piece: PromotionPiece::None, weight: weight};
            match moves.get_mut(&key) {
                Some(value) => value.push(full_move),
                None => {let move_vec = vec![full_move]; moves.insert(key, move_vec);},
            };
        }
        OpeningBook { moves: moves }
    }
}

pub struct Move {
    to_file : u8,
    to_row: u8,
    from_file: u8,
    from_row: u8,
    promotion_piece: PromotionPiece,
    weight: u16,
}

enum PromotionPiece {
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