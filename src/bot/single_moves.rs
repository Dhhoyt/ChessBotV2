use crate::bot::pseudomoves::{bishop_moves, rook_moves, KING_MOVES};

use super::{
    pseudomoves::{KNIGHT_MOVES, PAWN_ATTACKS, PAWN_MOVES},
    utils::{north_one, print_bit_board, south_one, BitBoardIter},
    BitBoard, Board,
};

impl Board {
    pub fn valid_moves(&self, square: usize) -> Vec<usize> {
        let mask = (1 as BitBoard) << square;
        if mask & self.occupied_by_white != 0 {
            let checkmask = self.white_checkmask();
            let pinmask_d = self.white_pinmask_d();
            let pinmask_hv = self.white_pinmask_hv();

            if mask & self.white_pawns != 0 {
                return self.single_pawn_moves(square, true, checkmask, pinmask_d, pinmask_hv);
            }
            else if mask & self.white_knights != 0 {
                return self.single_knight_moves(square, true, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.white_queens != 0 {
                return self.single_queen_moves(square, true, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.white_rooks != 0 {
                return self.single_rook_moves(square, true, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.white_bishops != 0 {
                return self.single_bishop_moves(square, true, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.white_kings != 0 {
                return self.single_king_moves(square, true, checkmask);
            }
        } else if mask & self.occupied_by_black != 0 {
            let checkmask = self.black_checkmask();
            let pinmask_d = self.black_pinmask_d();
            let pinmask_hv = self.black_pinmask_hv();
            if mask & self.black_pawns != 0 {
                return self.single_pawn_moves(square, false, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.black_knights != 0 {
                return self.single_knight_moves(square, false, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.black_queens != 0 {
                return self.single_queen_moves(square, false, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.black_rooks != 0 {
                return self.single_rook_moves(square, false, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.black_bishops != 0 {
                return self.single_bishop_moves(square, false, checkmask, pinmask_d, pinmask_hv);
            } else if mask & self.black_kings != 0 {
                return self.single_king_moves(square, false, checkmask);
            }
        }
        Vec::new()
    }

    fn single_pawn_moves(
        &self,
        square: usize,
        white: bool,
        checkmask: BitBoard,
        pinmask_d: BitBoard,
        pinmask_hv: BitBoard,
    ) -> Vec<usize> {
        let mask = ((1 as BitBoard) << square);
        let mut res = Vec::new();
        if white {
            if (pinmask_d | pinmask_hv) & mask != 0 {
                res.extend(
                    BitBoardIter(PAWN_MOVES[0][square] & pinmask_hv & checkmask & !self.occupied)
                        .filter(
                            (|x| !((x - square == 16) & ((north_one(mask) & self.occupied) != 0))),
                        ),
                );
                res.extend(BitBoardIter(
                    PAWN_ATTACKS[0][square] & pinmask_d & checkmask & (self.occupied_by_black | self.en_passant),
                ));
            } else {
                res.extend(
                    BitBoardIter(PAWN_MOVES[0][square] & checkmask & !self.occupied).filter(|x| {
                        !((x - square == 16) & ((north_one(mask) & self.occupied) != 0))
                    }),
                );
                res.extend(BitBoardIter(
                    PAWN_ATTACKS[0][square] & checkmask & (self.occupied_by_black | self.en_passant),
                ));
            }
        } else {
            if (pinmask_d | pinmask_hv) & mask != 0 {
                res.extend(
                    BitBoardIter(PAWN_MOVES[1][square] & pinmask_hv & checkmask & !self.occupied)
                        .filter(
                            (|x| !((x - square == 16) & ((north_one(mask) & self.occupied) != 0))),
                        ),
                );
                res.extend(BitBoardIter(
                    PAWN_ATTACKS[1][square] & pinmask_d & checkmask & (self.occupied_by_white | self.en_passant),
                ));
            } else {
                res.extend(
                    BitBoardIter(PAWN_MOVES[1][square] & checkmask & !self.occupied).filter(|x| {
                        !((x - square == 16) & ((north_one(mask) & self.occupied) != 0))
                    }),
                );
                res.extend(BitBoardIter(
                    PAWN_ATTACKS[1][square] & checkmask & (self.occupied_by_white | self.en_passant),
                ));
            }
        }
        res
    }

    fn single_knight_moves(
        &self,
        square: usize,
        white: bool,
        checkmask: BitBoard,
        pinmask_d: BitBoard,
        pinmask_hv: BitBoard,
    ) -> Vec<usize> {
        let mask = ((1 as BitBoard) << square);
        if mask & (pinmask_d | pinmask_hv) != 0 {
            return Vec::new();
        }
        if white {
            BitBoardIter(KNIGHT_MOVES[square] & checkmask & self.black_or_empty()).collect()
        } else {
            BitBoardIter(KNIGHT_MOVES[square] & checkmask & self.white_or_empty()).collect()
        }
    }

    fn single_queen_moves(
        &self,
        square: usize,
        white: bool,
        checkmask: BitBoard,
        pinmask_d: BitBoard,
        pinmask_hv: BitBoard,
    ) -> Vec<usize> {
        let mask = ((1 as BitBoard) << square);
        let moveable = if white {self.black_or_empty()} else {self.white_or_empty()};
        if mask & (pinmask_d | pinmask_hv) != 0 {
            let mut moves: Vec<usize> = BitBoardIter(bishop_moves(square, !self.occupied) & checkmask & moveable & pinmask_d).collect();
            moves.extend(BitBoardIter(rook_moves(square, !self.occupied) & checkmask & moveable & pinmask_hv));
            return moves;
        }
        else {
            let mut moves: Vec<usize> = BitBoardIter(bishop_moves(square, !self.occupied) & checkmask & moveable).collect();
            moves.extend(BitBoardIter(rook_moves(square, !self.occupied) & checkmask & moveable));
            return moves;
        }
    }

    fn single_bishop_moves(
        &self,
        square: usize,
        white: bool,
        checkmask: BitBoard,
        pinmask_d: BitBoard,
        pinmask_hv: BitBoard,
    ) -> Vec<usize> {
        let mask = ((1 as BitBoard) << square);
        let moveable = if white {self.black_or_empty()} else {self.white_or_empty()};
        if mask & (pinmask_d | pinmask_hv) != 0 {
            BitBoardIter(bishop_moves(square, !self.occupied) & checkmask & moveable & pinmask_d).collect()
        }
        else {
            BitBoardIter(bishop_moves(square, !self.occupied) & checkmask & moveable).collect()
        }
    }

    fn single_rook_moves(
        &self,
        square: usize,
        white: bool,
        checkmask: BitBoard,
        pinmask_d: BitBoard,
        pinmask_hv: BitBoard,
    ) -> Vec<usize> {
        let mask = ((1 as BitBoard) << square);
        let moveable = if white {self.black_or_empty()} else {self.white_or_empty()};
        if mask & (pinmask_d | pinmask_hv) != 0 {
            BitBoardIter(rook_moves(square, !self.occupied) & checkmask & moveable & pinmask_hv).collect()
        }
        else {
            BitBoardIter(rook_moves(square, !self.occupied) & checkmask & moveable).collect()
        }
    }

    fn single_king_moves(
        &self,
        square: usize,
        white: bool,
        checkmask: BitBoard,
    ) -> Vec<usize> {
        let mask = ((1 as BitBoard) << square);
        let moveable = if white {self.black_or_empty()} else {self.white_or_empty()};
        let under_attack = if white {self.under_attack_by_black()} else {self.under_attack_by_white()};
        let mut moves: Vec<usize> = BitBoardIter(KING_MOVES[square] & !under_attack & moveable).collect();
        if white {
            if (self.castle & 0x90 == 0x90) && under_attack & 0x70 == 0 { moves.push(7); }
            if (self.castle & 0x11 == 0x11) && under_attack & 0x1c == 0 { moves.push(0); }
        } else {
            if (self.castle & 0x9000000000000000 == 0x9000000000000000) && under_attack & 0x7000000000000000 == 0 {moves.push(63)};
            if (self.castle & 0x1100000000000000 == 0x1100000000000000) && under_attack & 0x1c00000000000000 == 0 {moves.push(56)};
        }
        moves
    }
}
