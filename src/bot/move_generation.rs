use std::pin;

use super::pseudomoves::*;
use super::utils::*;
use super::{BitBoard, Board};

//Black moves here
impl Board {
    #[inline]
    fn black_checkmask(&self) -> BitBoard {
        let king_square = self.black_kings.trailing_zeros() as usize;
        let mut checkmask: BitBoard = 0xFFFFFFFFFFFFFFFF;
        for i in BitBoardIter(self.white_queens) {
            if queen_moves(i, !self.occupied) & self.black_kings != 0 {
                checkmask &= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.white_rooks) {
            if rook_moves(i, !self.occupied) & self.black_kings != 0 {
                checkmask &= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.white_bishops) {
            if bishop_moves(i, !self.occupied) & self.black_kings != 0 {
                checkmask &= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.white_knights) {
            if KNIGHT_MOVES[i] & self.black_kings != 0 {
                checkmask &= (1 as BitBoard) << i;
            }
        }
        for i in BitBoardIter(self.white_pawns) {
            if PAWN_ATTACKS[0][i] & self.black_kings != 0 {
                checkmask &= (1 as BitBoard) << i;
            }
        }
        checkmask
    }

    #[inline]
    pub fn black_pinmask_hv(&self) -> BitBoard {
        let king_square = self.black_kings.trailing_zeros() as usize;
        let mut pinmask: BitBoard = 0;
        for i in BitBoardIter(self.white_queens) {
            let xray = rook_xray(i, !self.occupied);
            if xray & self.black_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.white_rooks) {
            let xray = rook_xray(i, !self.occupied);
            if xray & self.black_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        pinmask
    }

    #[inline]
    pub fn black_pinmask_d(&self) -> BitBoard {
        let king_square = self.black_kings.trailing_zeros() as usize;
        let mut pinmask: BitBoard = 0;
        for i in BitBoardIter(self.white_queens) {
            let xray = bishop_xray(i, !self.occupied);
            if xray & self.black_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.white_bishops) {
            let xray = bishop_xray(i, !self.occupied);
            if xray & self.black_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        pinmask
    }

    #[inline]
    pub fn under_attack_by_black(&self) -> BitBoard {
        let mut res = 0;
        for i in BitBoardIter(self.black_queens) {
            res |= queen_moves(i, !(self.occupied & !self.white_kings));
        }
        for i in BitBoardIter(self.black_rooks) {
            res |= rook_moves(i, !(self.occupied & !self.white_kings));
        }
        for i in BitBoardIter(self.black_bishops) {
            res |= bishop_moves(i, !(self.occupied & !self.white_kings));
        }
        for i in BitBoardIter(self.black_knights) {
            res |= KNIGHT_MOVES[i];
        }
        for i in BitBoardIter(self.black_kings) {
            res |= KING_MOVES[i];
        }
        for i in BitBoardIter(self.black_pawns) {
            res |= PAWN_ATTACKS[1][i];
        }
        res
    }

    #[inline]
    pub fn black_moves(&self) -> Vec<Board> {
        let mut res = Vec::with_capacity(40);
        let checkmask = self.black_checkmask();
        let pinmask_d = self.black_pinmask_d();
        let pinmask_hv = self.black_pinmask_hv();
        let pinmask = pinmask_d | pinmask_hv;
        //Unpinned rooks
        for i in BitBoardIter(self.black_rooks & !pinmask) {
            let moves = rook_moves(i, !self.occupied) & checkmask & self.white_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_rooks ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                res.push(new_board);
            }
        }
        //Pinned rooks
        for i in BitBoardIter(self.black_rooks & pinmask) {
            let moves =
                rook_moves(i, !self.occupied) & checkmask & self.white_or_empty() & pinmask_hv;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_rooks ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //Unpinned Bishops
        for i in BitBoardIter(self.black_bishops & !pinmask) {
            let moves = bishop_moves(i, !self.occupied) & checkmask & self.white_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_bishops ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }
        //Pinned Bishops
        for i in BitBoardIter(self.black_bishops & pinmask) {
            let moves =
                bishop_moves(i, !self.occupied) & checkmask & self.white_or_empty() & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_bishops ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //Unpinned Queens
        for i in BitBoardIter(self.black_queens & !pinmask) {
            let moves = (rook_moves(i, !self.occupied) | bishop_moves(i, !self.occupied))
                & checkmask
                & self.white_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_queens ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }
        //Pinned Queens
        for i in BitBoardIter(self.black_queens & pinmask) {
            //diagonal moves
            let moves =
                bishop_moves(i, !self.occupied) & checkmask & self.white_or_empty() & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_queens ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
            //hv moves
            let moves =
                rook_moves(i, !self.occupied) & checkmask & self.white_or_empty() & pinmask_hv;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_queens ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //A pinned knight can never move
        for i in BitBoardIter(self.black_knights & !pinmask) {
            let moves = KNIGHT_MOVES[i] & checkmask & self.white_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_knights ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //Unpinned Pawn Pushes
        for i in BitBoardIter(self.black_pawns & !pinmask & north_one(!self.occupied)) {
            let moves = PAWN_MOVES[1][i] & checkmask & !self.occupied;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                if i - m == 16 {
                    new_board.en_passant |= north_one(new_square);
                }
                new_board.black_pawns ^= piece_mask | new_square;
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //Pinned Pawn Pushes
        for i in BitBoardIter(self.black_pawns & pinmask & north_one(!self.occupied)) {
            let moves = PAWN_MOVES[1][i] & checkmask & !self.occupied & pinmask_hv;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                if i - m == 16 {
                    new_board.en_passant |= north_one(new_square);
                }
                new_board.black_pawns ^= piece_mask;
                if m > 8 {
                    let mut queen_board = new_board.clone();
                    queen_board.black_queens |= new_square;
                    queen_board.redo_occupied();
                    queen_board.white_to_play = true    ;
                    res.push(queen_board);
                    let mut rook_board = new_board.clone();
                    rook_board.black_rooks |= new_square;
                    rook_board.redo_occupied();
                    rook_board.white_to_play = true;
                    res.push(rook_board);
                    let mut knight_board = new_board.clone();
                    knight_board.black_knights |= new_square;
                    knight_board.redo_occupied();
                    knight_board.white_to_play = true;
                    res.push(knight_board);
                    let mut bishop_board = new_board.clone();
                    bishop_board.black_bishops |= new_square;
                    bishop_board.redo_occupied();
                    bishop_board.white_to_play = true;
                    res.push(bishop_board);
                }
                else {
                    new_board.white_pawns |= new_square;
                    new_board.redo_occupied();
                    new_board.white_to_play = false;
                    res.push(new_board);
                } 
            }
        }

        //Unpinned Pawn Attacks
        for i in BitBoardIter(self.black_pawns & !pinmask) {
            let moves = PAWN_ATTACKS[1][i] & checkmask & self.occupied_by_white;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_pawns ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //Pinned Pawn Attacks
        for i in BitBoardIter(self.black_pawns & pinmask) {
            let moves = PAWN_ATTACKS[1][i] & checkmask & self.occupied_by_white & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_pawns ^= piece_mask | new_square;
                new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //Unpinned En Passant
        for i in BitBoardIter(self.black_pawns & !pinmask) {
            let moves = PAWN_ATTACKS[0][i] & checkmask & self.en_passant;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_pawns ^= piece_mask | new_square;
                new_board.capture_white(north_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //Pinned En Passant
        for i in BitBoardIter(self.black_pawns & pinmask) {
            let moves = PAWN_ATTACKS[0][i] & checkmask & self.en_passant & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.black_pawns ^= piece_mask | new_square;
                new_board.capture_white(north_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push(new_board);
            }
        }

        //King Moves
        let king_square = self.black_kings.trailing_zeros() as usize;
        let moves = KING_MOVES[king_square] & !self.under_attack_by_white() & self.white_or_empty();
        for m in BitBoardIter(moves) {
            let new_square = (1 as u64) << m;
            let mut new_board = self.clone();
            new_board.en_passant = 0;
            new_board.black_kings = new_square;
            new_board.capture_white(new_square);
            new_board.redo_occupied();
            new_board.white_to_play = true;
            res.push(new_board);
        }

        res
    }

    #[inline]
    pub fn capture_black(&mut self, mask: BitBoard) {
        self.black_queens &= !mask;
        self.black_rooks &= !mask;
        self.black_bishops &= !mask;
        self.black_knights &= !mask;
        self.black_pawns &= !mask;
    }
}

//White moves here
impl Board {
    #[inline]
    fn white_checkmask(&self) -> BitBoard {
        let king_square = self.white_kings.trailing_zeros() as usize;
        let mut checkmask: BitBoard = 0xFFFFFFFFFFFFFFFF;
        for i in BitBoardIter(self.black_queens) {
            if queen_moves(i, !self.occupied) & self.white_kings != 0 {
                checkmask &= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.black_rooks) {
            if rook_moves(i, !self.occupied) & self.white_kings != 0 {
                checkmask &= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.black_bishops) {
            if bishop_moves(i, !self.occupied) & self.white_kings != 0 {
                checkmask &= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.black_knights) {
            if KNIGHT_MOVES[i] & self.white_kings != 0 {
                checkmask &= (1 as BitBoard) << i;
            }
        }
        for i in BitBoardIter(self.black_pawns) {
            if PAWN_ATTACKS[1][i] & self.white_kings != 0 {
                checkmask &= (1 as BitBoard) << i;
            }
        }
        checkmask
    }

    #[inline]
    pub fn white_pinmask_hv(&self) -> BitBoard {
        let king_square = self.white_kings.trailing_zeros() as usize;
        let mut pinmask: BitBoard = 0;
        for i in BitBoardIter(self.black_queens) {
            let xray = rook_xray(i, !self.occupied);
            if xray & self.white_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.black_rooks) {
            let xray = rook_xray(i, !self.occupied);
            if xray & self.white_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        pinmask
    }

    #[inline]
    pub fn white_pinmask_d(&self) -> BitBoard {
        let king_square = self.white_kings.trailing_zeros() as usize;
        let mut pinmask: BitBoard = 0;
        for i in BitBoardIter(self.black_queens) {
            let xray = bishop_xray(i, !self.occupied);
            if xray & self.white_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        for i in BitBoardIter(self.black_bishops) {
            let xray = bishop_xray(i, !self.occupied);
            if xray & self.white_kings != 0 {
                pinmask |= PATH_BETWEEN[king_square][i];
            }
        }
        pinmask
    }

    #[inline]
    pub fn white_moves(&self) -> Vec<Board> {
        let mut res = Vec::with_capacity(40);
        let checkmask = self.white_checkmask();
        let pinmask_d = self.white_pinmask_d();
        let pinmask_hv = self.white_pinmask_hv();
        let pinmask = pinmask_d | pinmask_hv;
        //Unpinned rooks
        for i in BitBoardIter(self.white_rooks & !pinmask) {
            let moves = rook_moves(i, !self.occupied) & checkmask & self.black_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_rooks ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }
        //Pinned rooks
        for i in BitBoardIter(self.white_rooks & pinmask) {
            let moves =
                rook_moves(i, !self.occupied) & checkmask & self.black_or_empty() & pinmask_hv;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_rooks ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //Unpinned Bishops
        for i in BitBoardIter(self.white_bishops & !pinmask) {
            let moves = bishop_moves(i, !self.occupied) & checkmask & self.black_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_bishops ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }
        //Pinned Bishops
        for i in BitBoardIter(self.white_bishops & pinmask) {
            let moves =
                bishop_moves(i, !self.occupied) & checkmask & self.black_or_empty() & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_bishops ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //Unpinned Queens
        for i in BitBoardIter(self.white_queens & !pinmask) {
            let moves = (rook_moves(i, !self.occupied) | bishop_moves(i, !self.occupied))
                & checkmask
                & self.black_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_queens ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }
        //Pinned Queens
        for i in BitBoardIter(self.white_queens & pinmask) {
            //diagonal moves
            let moves =
                bishop_moves(i, !self.occupied) & checkmask & self.black_or_empty() & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_queens ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
            //hv moves
            let moves =
                rook_moves(i, !self.occupied) & checkmask & self.black_or_empty() & pinmask_hv;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_queens ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //A pinned knight can never move
        for i in BitBoardIter(self.white_knights & !pinmask) {
            let moves = KNIGHT_MOVES[i] & checkmask & self.black_or_empty();
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_knights ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //Unpinned Pawn Pushes
        for i in BitBoardIter(self.white_pawns & !pinmask & south_one(!self.occupied) & !0x00FF000000000000) {
            let moves = PAWN_MOVES[0][i] & checkmask & !self.occupied;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                if m - i == 16 {
                    new_board.en_passant |= south_one(new_square);
                }
                new_board.white_pawns ^= piece_mask;
                if m > 55 {
                    let mut queen_board = new_board.clone();
                    queen_board.white_queens |= new_square;
                    queen_board.redo_occupied();
                    queen_board.white_to_play = false;
                    res.push(queen_board);
                    let mut rook_board = new_board.clone();
                    rook_board.white_rooks |= new_square;
                    rook_board.redo_occupied();
                    rook_board.white_to_play = false;
                    res.push(rook_board);
                    let mut knight_board = new_board.clone();
                    knight_board.white_knights |= new_square;
                    knight_board.redo_occupied();
                    knight_board.white_to_play = false;
                    res.push(knight_board);
                    let mut bishop_board = new_board.clone();
                    bishop_board.white_bishops |= new_square;
                    bishop_board.redo_occupied();
                    bishop_board.white_to_play = false;
                    res.push(bishop_board);
                }
                else {
                    new_board.white_pawns |= new_square;
                    new_board.redo_occupied();
                    new_board.white_to_play = false;
                    res.push(new_board);
                } 
            }
        }

        //Pinned Pawn Pushes
        for i in BitBoardIter(self.white_pawns & pinmask & south_one(!self.occupied)) {
            let moves = PAWN_MOVES[0][i] & checkmask & !self.occupied & pinmask_hv;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                if m - i == 16 {
                    new_board.en_passant |= south_one(new_square);
                }
                new_board.white_pawns ^= piece_mask | new_square;
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //Unpinned Pawn Attacks
        for i in BitBoardIter(self.white_pawns & !pinmask) {
            let moves = PAWN_ATTACKS[0][i] & checkmask & self.occupied_by_black;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_pawns ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //Pinned Pawn Attacks
        for i in BitBoardIter(self.white_pawns & pinmask) {
            let moves = PAWN_ATTACKS[0][i] & checkmask & self.occupied_by_black & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_pawns ^= piece_mask | new_square;
                new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //Unpinned En Passant
        for i in BitBoardIter(self.white_pawns & !pinmask) {
            let moves = PAWN_ATTACKS[0][i] & checkmask & self.en_passant;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_pawns ^= piece_mask | new_square;
                new_board.capture_black(south_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //Pinned En Passant
        for i in BitBoardIter(self.white_pawns & pinmask) {
            let moves = PAWN_ATTACKS[0][i] & checkmask & self.en_passant & pinmask_d;
            for m in BitBoardIter(moves) {
                let piece_mask = (1 as u64) << i;
                let new_square = (1 as u64) << m;
                let mut new_board = self.clone();
                new_board.en_passant = 0;
                new_board.white_pawns ^= piece_mask | new_square;
                new_board.capture_black(south_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push(new_board);
            }
        }

        //King Moves
        let king_square = self.white_kings.trailing_zeros() as usize;
        let moves = KING_MOVES[king_square] & !self.under_attack_by_black() & self.black_or_empty();
        for m in BitBoardIter(moves) {
            let new_square = (1 as u64) << m;
            let mut new_board = self.clone();
            new_board.white_kings = new_square;
            new_board.capture_black(new_square);
            new_board.redo_occupied();
            new_board.white_to_play = false;
            res.push(new_board);
        }

        res
    }

    #[inline]
    pub fn under_attack_by_white(&self) -> BitBoard {
        let mut res = 0;
        for i in BitBoardIter(self.white_queens) {
            res |= queen_moves(i, !(self.occupied & !self.black_kings));
        }
        for i in BitBoardIter(self.white_rooks) {
            res |= rook_moves(i, !(self.occupied & !self.black_kings));
        }
        for i in BitBoardIter(self.white_bishops) {
            res |= bishop_moves(i, !(self.occupied & !self.black_kings));
        }
        for i in BitBoardIter(self.white_knights) {
            res |= KNIGHT_MOVES[i];
        }
        for i in BitBoardIter(self.white_kings) {
            res |= KING_MOVES[i];
        }
        for i in BitBoardIter(self.white_pawns) {
            res |= PAWN_ATTACKS[0][i];
        }
        res
    }

    #[inline]
    pub fn capture_white(&mut self, mask: BitBoard) {
        self.white_queens &= !mask;
        self.white_rooks &= !mask;
        self.white_bishops &= !mask;
        self.white_knights &= !mask;
        self.white_pawns &= !mask;
    }
}

impl Board {
    #[inline]
    pub fn redo_occupied(&mut self) {
        self.occupied_by_white = self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_kings;
        self.occupied_by_black = self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_kings;
        self.occupied = self.occupied_by_white | self.occupied_by_black;
    }
}
