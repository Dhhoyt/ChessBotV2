use std::{pin, f32::INFINITY, collections::HashMap, hash::{Hash, Hasher, BuildHasherDefault}, sync::{Mutex, Arc}, mem};

use self::{
    move_generation::*,
    pseudomoves::*,
    utils::{print_bit_board, BitBoardIter, north_one},
};

mod move_generation;
pub mod pseudomoves;
mod utils;

const CHECKMATE_VALUE: f32 = 1000000.0;
const CHECKMATE_THRESHOLD: f32 = 100000.0;

type BitBoard = u64;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    black_pawns: BitBoard,
    black_knights: BitBoard,
    pub black_bishops: BitBoard,
    black_rooks: BitBoard,
    black_queens: BitBoard,
    black_kings: BitBoard,

    white_pawns: BitBoard,
    white_knights: BitBoard,
    white_bishops: BitBoard,
    white_rooks: BitBoard,
    white_queens: BitBoard,
    white_kings: BitBoard,

    occupied_by_white: BitBoard,
    occupied_by_black: BitBoard,
    occupied: BitBoard,

    castle: BitBoard,

    en_passant: BitBoard,

    pub white_to_play: bool,
}

impl Board {
    fn new() -> Self {
        Board {
            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_kings: 0,
            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_kings: 0,

            occupied_by_white: 0,
            occupied_by_black: 0,
            occupied: 0,

            castle: 0,

            en_passant: 0,

            white_to_play: true,
        }
    }

    #[inline]
    const fn white_or_empty(&self) -> BitBoard {
        !self.occupied_by_black
    }

    #[inline]
    const fn black_or_empty(&self) -> BitBoard {
        !self.occupied_by_white
    }

    pub fn to_fen(&self) -> String {
        let mut res = String::new();
        for y in 0..8 {
            let mut blank_spaces = 0;
            for x in 0..8 {
                let square = (7 - y) * 8 + x;
                let mask: u64 = 1 << square;
                if self.occupied & mask != 0 {
                    if blank_spaces != 0 {
                        res.push_str(&blank_spaces.to_string());
                        blank_spaces = 0;
                    }
                    if self.black_kings & mask != 0 {
                        res.push('k');
                    } else if self.black_queens & mask != 0 {
                        res.push('q');
                    } else if self.black_rooks & mask != 0 {
                        res.push('r');
                    } else if self.black_knights & mask != 0 {
                        res.push('n');
                    } else if self.black_bishops & mask != 0 {
                        res.push('b');
                    } else if self.black_pawns & mask != 0 {
                        res.push('p');
                    } else if self.white_kings & mask != 0 {
                        res.push('K');
                    } else if self.white_queens & mask != 0 {
                        res.push('Q');
                    } else if self.white_rooks & mask != 0 {
                        res.push('R');
                    } else if self.white_knights & mask != 0 {
                        res.push('N');
                    } else if self.white_bishops & mask != 0 {
                        res.push('B');
                    } else if self.white_pawns & mask != 0 {
                        res.push('P');
                    }
                } else {
                    blank_spaces += 1;
                }
            }
            if blank_spaces != 0 {
                res.push_str(&blank_spaces.to_string());
            }
            if y != 7 {
                res.push_str("/");
            }
        }
        res.push_str(&format!(" {} ", if self.white_to_play {'w'} else {'b'}));
        if 0x80 & self.castle != 0 { res.push('K') };
        if 0x1 & self.castle != 0 { res.push('Q') };
        if 0x8000000000000000 & self.castle != 0 { res.push('k') };
        if 0x100000000000000 & self.castle != 0 { res.push('q') };
        if 0x8100000000000081 & self.castle == 0 {res.push('-')};
        res
    }

    pub fn from_fen(fen: &str) -> Result<Self, FENError> {
        let split: Vec<&str> = fen.split(" ").collect();
        if split.len() != 6 {
            return Err(FENError::IncorrrectNumberOfTokens);
        }
        let ranks: Vec<&str> = split.get(0).unwrap().split("/").collect();
        if ranks.len() != 8 {
            return Err(FENError::IncorrrectNumberOfRanks);
        }
        let mut res = Board::new();
        for (i, r) in ranks.into_iter().enumerate() {
            let mut pos: u64 = 0;
            for c in r.chars() {
                let square: u64 = 1 << (((7 - i as u64) * 8) + pos);
                match c {
                    'p' => res.black_pawns += square,
                    'n' => res.black_knights += square,
                    'b' => res.black_bishops += square,
                    'q' => res.black_queens += square,
                    'r' => res.black_rooks += square,
                    'k' => res.black_kings += square,
                    'P' => res.white_pawns += square,
                    'N' => res.white_knights += square,
                    'Q' => res.white_queens += square,
                    'R' => res.white_rooks += square,
                    'K' => res.white_kings += square,
                    'B' => res.white_bishops += square,
                    '1' => pos += 0,
                    '2' => pos += 1,
                    '3' => pos += 2,
                    '4' => pos += 3,
                    '5' => pos += 4,
                    '6' => pos += 5,
                    '7' => pos += 6,
                    '8' => pos += 7,
                    _ => return Err(FENError::InvalidPiece(c)),
                }
                pos += 1;
            }
            if pos != 8 {
                return Err(FENError::InvalidRankLength);
            }
        }
        let turn = split.get(1).unwrap();
        if *turn == "w" {
            res.white_to_play = true;
        } else if *turn == "b" {
            res.white_to_play = false;
        } else {
            return Err(FENError::InvalidTurn(turn.chars().next().unwrap()));
        }
        let castle = split.get(2).unwrap();
        if castle.contains('K') {
            res.castle |= 0x90;
        }
        if castle.contains('Q') {
            res.castle |= 0x11;
        }
        if castle.contains('k') {
            res.castle |= 0x9000000000000000;
        }
        if castle.contains('q') {
            res.castle |= 0x1100000000000000;
        }
        res.redo_occupied();
        Ok(res)
    }

    pub fn hueristic(&self) -> f32 {
        let mut total: f32 = 0.;
        total += (self.white_queens.count_ones() as f32 - self.black_queens.count_ones() as f32)
            * 9.;
        total += (self.white_rooks.count_ones() as f32 - self.black_rooks.count_ones() as f32)
            * 5.;
        total += (self.white_bishops.count_ones() as f32 - self.black_bishops.count_ones() as f32)
            * 3.;
        total += (self.white_knights.count_ones() as f32 - self.black_knights.count_ones() as f32)
            * 3.;
        total += (self.white_pawns.count_ones() as f32 - self.black_pawns.count_ones() as f32)
            * 1.;
        
        total += self.under_attack_by_white().count_ones() as f32 * 0.25;
        total -= self.under_attack_by_black().count_ones() as f32 * 0.25;
        total
    }

    pub fn find_move_interatively(self, depth: usize, trans_table: &mut HashMap<Board, (usize,f32)>) -> (Board, f32) {
        
        let mut best = (Board::default(), 0.0);

        if self.white_to_play {
            best = Board::alpha_beta(self, depth, -CHECKMATE_VALUE - 2., CHECKMATE_VALUE + 2., true, trans_table);
        } else {
            best = Board::alpha_beta(self, depth, -CHECKMATE_VALUE - 2., CHECKMATE_VALUE + 2., false, trans_table);
        }

        best
    }
    
    pub fn find_move(self, depth: usize, trans_table: &mut HashMap<Board, (usize,f32)>) -> (Board, f32) {
        if self.white_to_play {
            Board::alpha_beta(self, depth, -CHECKMATE_VALUE - 2., CHECKMATE_VALUE + 2., true, trans_table)
        } else {
            Board::alpha_beta(self, depth, -CHECKMATE_VALUE - 2., CHECKMATE_VALUE + 2., false, trans_table)
        }
    }
    
    fn alpha_beta(board: Board, depth: usize, mut alpha: f32, mut beta: f32, white: bool, trans_table: &mut HashMap<Board, (usize, f32)>) -> (Board, f32) {
        //println!("{}", board.to_fen());
        if depth == 0 {
            return (board, board.hueristic());
        }
        let lookup = trans_table.get(&board);
        match lookup {
            None => (),
            Some(result) => {
                if result.0 >= depth {
                    return (board, result.1);
                }
            }
        }
        let mut moves = if white { board.white_moves() } else { board.black_moves() };

        if white {
            let mut value = -CHECKMATE_VALUE - 2.0;
            if moves.len() == 0 {
                return if board.white_kings & board.under_attack_by_black() != 0 {(board,-CHECKMATE_VALUE)} else {(board,0.0)};
            }
            let mut best_move: Option<Board>= None;
            for i in OrderedMoves(moves) {
                let eval = Board::alpha_beta(i, depth - 1, alpha, beta, false, trans_table);
                if eval.1 > value {
                    value = eval.1;
                    if depth == 7 {
                        println!("Switched: {} {}", i.to_fen(), eval.1);
                        
                    }
                    best_move = Some(i);
                }
                alpha = f32::max(alpha, value);
                
                if value >= beta {
                    break;
                }
            }
            if value > CHECKMATE_THRESHOLD {    
                value -= 1.;
            }
            if value < -CHECKMATE_THRESHOLD {
                value += 1.;
            }
            
            match trans_table.get_mut(&board) {
                None => {trans_table.insert(board, (depth, value));},
                Some(result) => {
                    if result.0 < depth {
                        *result = (depth, value);
                    }
                },
            }
            return (best_move.expect("No best move found?!?!?!?!?"), value);
        } else {
            let mut value = CHECKMATE_VALUE + 2.0;
            if moves.len() == 0 {
                return if board.black_kings & board.under_attack_by_white() != 0 {(board,CHECKMATE_VALUE)} else {(board,0.0)};
            }
            let mut best_move: Option<Board>= None;
            for i in OrderedMoves(moves) {
                let eval = Board::alpha_beta(i, depth - 1, alpha, beta, true, trans_table);
                if eval.1 < value {
                    value = eval.1;
                    best_move = Some(i);
                }

                beta = f32::min(beta, value);
                if value <= alpha {
                    break;
                }
                
            }
            if value < -CHECKMATE_THRESHOLD {
                value += 1.;
            }
            if value > CHECKMATE_THRESHOLD {
                value -= 1.;
            }
            match trans_table.get_mut(&board) {
                None => {trans_table.insert(board, (depth, value));},
                Some(result) => {
                    if result.0 < depth {
                        *result = (depth, value);
                    }
                },
            }
            return (best_move.expect("No best move found?!?!?!?!?"), value);
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            white_kings: 0x0000000000000010,
            white_queens: 0x0000000000000008,
            white_rooks: 0x0000000000000081,
            white_bishops: 0x0000000000000024,
            white_knights: 0x0000000000000042,
            white_pawns: 0x000000000000FF00,
            black_kings: 0x1000000000000000,
            black_queens: 0x0800000000000000,
            black_rooks: 0x8100000000000000,
            black_bishops: 0x2400000000000000,
            black_knights: 0x4200000000000000,
            black_pawns: 0x00FF000000000000,

            occupied: 0xFFFF00000000FFFF,
            occupied_by_black: 0xFFFF000000000000,
            occupied_by_white: 0x000000000000FFFF,

            castle: 0x9100000000000091,

            en_passant: 0,

            white_to_play: true,
        }
    }
}

#[derive(Debug)]
pub enum FENError {
    IncorrrectNumberOfTokens,
    IncorrrectNumberOfRanks,
    InvalidPiece(char),
    InvalidTurn(char),
    InvalidRankLength,
}

impl std::hash::Hash for Board {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        state.write_u64(self.occupied);
        state.finish();
    }
}

struct OrderedMoves(Vec<(Board, i32)>);

impl Iterator for OrderedMoves {
    type Item = Board;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() == 0 {
            return None;
        }
        let mut best_index = 0;
        let mut best_score = 0;
        for i in self.0.iter_mut().enumerate() {
            
            if i.1.1 > best_score {
                best_score = i.1.1;
                best_index = i.0;
            }
        }
        let best = self.0.get_mut(best_index).unwrap();
        if best.1 == -1 {
            return None;
        }
        best.1 = -1;
        Some(best.0)
    }
}
