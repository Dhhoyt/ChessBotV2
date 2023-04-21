use super::{BitBoard, Board, utils::{south_one, north_one}};

impl Board {
    pub fn make_move(self, book_move: Move) -> Board {
        let mut res = self.clone();
        let from_mask = (1 as BitBoard) << book_move.from_square;
        let to_mask = (1 as BitBoard) << book_move.to_square;
        res.en_passant = 0;
        if res.white_pawns & from_mask != 0 {
            res.white_pawns ^= from_mask | to_mask;
            if to_mask & self.en_passant != 0 {
                res.capture_black(south_one(to_mask));
            }
            if book_move.to_square - book_move.from_square == 16 {
                res.en_passant = south_one(to_mask);
            }
            if book_move.to_square < 8 {
                res.white_queens |= to_mask;
            } else {
                res.white_pawns |= to_mask;
            }
            res.capture_black(to_mask);
        } else if res.white_knights & from_mask != 0 {
            res.white_knights ^= from_mask | to_mask;
            res.capture_black(to_mask);
        } else if res.white_bishops & from_mask != 0 {
            res.white_bishops ^= from_mask | to_mask;
            res.capture_black(to_mask);
        } else if res.white_rooks & from_mask != 0 {
            res.white_rooks ^= from_mask | to_mask;
            res.capture_black(to_mask);
        } else if res.white_queens & from_mask != 0 {
            res.white_queens ^= from_mask | to_mask;
            res.capture_black(to_mask);
        } else if res.white_kings & from_mask != 0 {
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
        } else if res.black_pawns & from_mask != 0 {
            res.black_pawns ^= from_mask;
            if to_mask & self.en_passant != 0 {
                res.capture_white(north_one(to_mask));
            }
            if book_move.from_square - book_move.to_square == 16 {
                res.en_passant = north_one(to_mask);
            }
            if book_move.to_square < 8 {
                res.black_queens |= to_mask;
            } else {
                res.black_pawns |= to_mask;
            }
            res.capture_white(to_mask);
        } else if res.black_knights & from_mask != 0 {
            res.black_knights ^= from_mask | to_mask;
            res.capture_white(to_mask);
        } else if res.black_bishops & from_mask != 0 {
            res.black_bishops ^= from_mask | to_mask;
            res.capture_white(to_mask);
        } else if res.black_rooks & from_mask != 0 {
            res.black_rooks ^= from_mask | to_mask;
            res.capture_white(to_mask);
        } else if res.black_queens & from_mask != 0 {
            res.black_queens ^= from_mask | to_mask;
            res.capture_white(to_mask);
        } else if res.black_kings & from_mask != 0 {
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
        res.white_to_play = !res.white_to_play;
        res
    }
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
    Queen,
}

impl PromotionPiece {
    fn from_num(piece: u8) -> Self {
        match piece {
            0 => Self::None,
            1 => Self::Knight,
            2 => Self::Bishop,
            3 => Self::Rook,
            4 => Self::Queen,
            _ => panic!("Invalid piece"),
        }
    }
}
