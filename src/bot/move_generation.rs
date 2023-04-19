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
    pub fn black_moves(&self) -> Vec<(Board, i32)> {
        let mut res: Vec<(Board, i32)> = Vec::with_capacity(40);
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                new_board.castle &= !piece_mask;
                res.push((new_board, score + 2));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                new_board.castle &= !piece_mask;
                res.push((new_board, score + 2));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 3));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 3));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 1));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 1));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 1));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 4));
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
                new_board.black_pawns ^= piece_mask;
                if m < 8 {
                    let mut queen_board = new_board.clone();
                    queen_board.black_queens |= new_square;
                    queen_board.redo_occupied();
                    queen_board.white_to_play = true;
                    res.push((queen_board, 1000));
                    let mut rook_board = new_board.clone();
                    rook_board.black_rooks |= new_square;
                    rook_board.redo_occupied();
                    rook_board.white_to_play = true;
                    res.push((rook_board, 900));
                    let mut knight_board = new_board.clone();
                    knight_board.black_knights |= new_square;
                    knight_board.redo_occupied();
                    knight_board.white_to_play = true;
                    res.push((knight_board, 700));
                    let mut bishop_board = new_board.clone();
                    bishop_board.black_bishops |= new_square;
                    bishop_board.redo_occupied();
                    bishop_board.white_to_play = true;
                    res.push((bishop_board, 800));
                } else {
                    new_board.black_pawns |= new_square;
                    new_board.redo_occupied();
                    new_board.white_to_play = true;
                    res.push((new_board, 5));
                }
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
                new_board.black_pawns ^= piece_mask | new_square;
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, 6));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 6));
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
                let score = new_board.capture_white(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 6));
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
                let score = new_board.capture_white(north_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 6));
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
                let score = new_board.capture_white(north_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = true;
                res.push((new_board, score + 6));
            }
        }

        //King Moves
        let king_square = self.black_kings.trailing_zeros() as usize;
        let under_attack = self.under_attack_by_white();
        let moves = KING_MOVES[king_square] & !under_attack & self.white_or_empty();
        for m in BitBoardIter(moves) {
            let new_square = (1 as u64) << m;
            let mut new_board = self.clone();
            new_board.en_passant = 0;
            new_board.black_kings = new_square;
            let score = new_board.capture_white(new_square);
            new_board.redo_occupied();
            new_board.white_to_play = true;
            new_board.castle &= !0xff00000000000000;
            res.push((new_board, score));
        }

        //Castleing
        if (self.castle & 0x9000000000000000 == 0x9000000000000000)
            && under_attack & 0x7000000000000000 == 0
        {
            let mut new_board = self.clone();
            new_board.black_rooks ^= 0xa000000000000000;
            new_board.black_kings ^= 0x5000000000000000;
            new_board.white_to_play = true;
            new_board.redo_occupied();
            res.push((new_board, 0));
        }
        if (self.castle & 0x1100000000000000 == 0x1100000000000000)
            && under_attack & 0x1c00000000000000 == 0
        {
            let mut new_board = self.clone();
            new_board.black_rooks ^= 0x900000000000000;
            new_board.black_kings ^= 0x1400000000000000;
            new_board.white_to_play = true;
            new_board.redo_occupied();
            res.push((new_board, 0));
        }

        res
    }

    #[inline]
    pub fn capture_black(&mut self, mask: BitBoard) -> i32 {
        self.castle &= !mask;
        if mask & self.occupied != 0 {
            if mask & self.black_pawns != 0 {
                self.black_pawns &= !mask;
                return 100;
            }
            if mask & self.black_knights != 0 {
                self.black_knights &= !mask;
                return 200;
            }
            if mask & self.black_bishops != 0 {
                self.black_bishops &= !mask;
                return 300;
            }
            if mask & self.black_rooks != 0 {
                self.black_rooks &= !mask;
                return 400;
            }
            if mask & self.black_queens != 0 {
                self.black_queens &= !mask;
                return 500;
            }
        }
        0
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
    pub fn white_moves(&self) -> Vec<(Board, i32)> {
        let mut res: Vec<(Board, i32)> = Vec::with_capacity(40);
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                new_board.castle &= !piece_mask;
                res.push((new_board, score + 2));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                new_board.castle &= !piece_mask;
                res.push((new_board, score + 2));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 3));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 3));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 1));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 1));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 1));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 4));
            }
        }

        //Unpinned Pawn Pushes
        for i in BitBoardIter(
            self.white_pawns & !pinmask & south_one(!self.occupied) & !0x00FF000000000000,
        ) {
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
                    res.push((queen_board, 1000));
                    let mut rook_board = new_board.clone();
                    rook_board.white_rooks |= new_square;
                    rook_board.redo_occupied();
                    rook_board.white_to_play = false;
                    res.push((rook_board, 900));
                    let mut knight_board = new_board.clone();
                    knight_board.white_knights |= new_square;
                    knight_board.redo_occupied();
                    knight_board.white_to_play = false;
                    res.push((knight_board, 700));
                    let mut bishop_board = new_board.clone();
                    bishop_board.white_bishops |= new_square;
                    bishop_board.redo_occupied();
                    bishop_board.white_to_play = false;
                    res.push((bishop_board, 800));
                } else {
                    new_board.white_pawns |= new_square;
                    new_board.redo_occupied();
                    new_board.white_to_play = false;
                    res.push((new_board, 5));
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
                res.push((new_board, 5));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 5));
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
                let score = new_board.capture_black(new_square);
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 5));
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
                let score = new_board.capture_black(south_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 5));
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
                let score = new_board.capture_black(south_one(new_square));
                new_board.redo_occupied();
                new_board.white_to_play = false;
                res.push((new_board, score + 5));
            }
        }

        //King Moves
        let king_square = self.white_kings.trailing_zeros() as usize;
        let under_attack = self.under_attack_by_black();
        let moves = KING_MOVES[king_square] & !under_attack & self.black_or_empty();
        for m in BitBoardIter(moves) {
            let new_square = (1 as u64) << m;
            let mut new_board = self.clone();
            new_board.white_kings = new_square;
            let score = new_board.capture_black(new_square);
            new_board.redo_occupied();
            new_board.white_to_play = false;
            new_board.castle &= !0xff;
            res.push((new_board, score));
        }

        //Castleing
        if (self.castle & 0x90 == 0x90) && under_attack & 0x70 == 0 {
            let mut new_board = self.clone();
            new_board.white_rooks ^= 0xa0;
            new_board.white_kings ^= 0x50;
            new_board.white_to_play = false;
            new_board.redo_occupied();
            res.push((new_board, 0));
        }
        if (self.castle & 0x11 == 0x11) && under_attack & 0x1c == 0 {
            let mut new_board = self.clone();
            new_board.white_rooks ^= 0x9;
            new_board.white_kings ^= 0x14;
            new_board.white_to_play = false;
            new_board.redo_occupied();
            res.push((new_board, 0));
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
    pub fn capture_white(&mut self, mask: BitBoard) -> i32 {
        self.castle &= !mask;
        if mask & self.occupied != 0 {
            if mask & self.white_pawns != 0 {
                self.white_pawns &= !mask;
                return 100;
            }
            if mask & self.white_knights != 0 {
                self.white_knights &= !mask;
                return 200;
            }
            if mask & self.white_bishops != 0 {
                self.white_bishops &= !mask;
                return 300;
            }
            if mask & self.white_rooks != 0 {
                self.white_rooks &= !mask;
                return 400;
            }
            if mask & self.white_queens != 0 {
                self.white_queens &= !mask;
                return 500;
            }
        }
        0
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
