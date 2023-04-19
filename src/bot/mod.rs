use std::{
    collections::HashMap,
    f32::INFINITY,
    hash::{BuildHasherDefault, Hash, Hasher},
    mem, pin,
    sync::{Arc, Mutex},
};

use crate::Piece;

use self::{move_generation::*, pseudomoves::*, utils::*, opening::OpeningBook};

mod magic_bitboards;
mod move_generation;
mod search;
pub mod opening;
mod pseudomoves;
mod utils;
mod zobrist;

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
        res.push_str(&format!(" {} ", if self.white_to_play { 'w' } else { 'b' }));
        if 0x80 & self.castle != 0 {
            res.push('K')
        };
        if 0x1 & self.castle != 0 {
            res.push('Q')
        };
        if 0x8000000000000000 & self.castle != 0 {
            res.push('k')
        };
        if 0x100000000000000 & self.castle != 0 {
            res.push('q')
        };
        if 0x8100000000000081 & self.castle == 0 {
            res.push('-')
        };
        res.push(' ');
        let en_passant = self.get_en_passant();
        match en_passant {
            0x10000 => res.push_str("a3"),
            0x20000 => res.push_str("b3"),
            0x40000 => res.push_str("c3"),
            0x80000 => res.push_str("d3"),
            0x100000 => res.push_str("e3"),
            0x200000 => res.push_str("f3"),
            0x400000 => res.push_str("g3"),
            0x800000 => res.push_str("h3"),
            0x10000000000 => res.push_str("a6"),
            0x20000000000 => res.push_str("b6"),
            0x40000000000 => res.push_str("c6"),
            0x80000000000 => res.push_str("d6"),
            0x100000000000 => res.push_str("e6"),
            0x200000000000 => res.push_str("f6"),
            0x400000000000 => res.push_str("g6"),
            0x800000000000 => res.push_str("h6"),
            _ => res.push('-')
        }
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

        match *split.get(1).unwrap() {
            "w" => res.white_to_play = true,
            "b" => res.white_to_play = false,
            _ => return Err(FENError::InvalidTurnToken),
        };

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

        let en_passant = *split.get(3).unwrap();

        match en_passant {
            "-" => (),
            "a3" => res.en_passant = 0x10000,
            "b3" => res.en_passant = 0x20000,
            "c3" => res.en_passant = 0x40000,
            "d3" => res.en_passant = 0x80000,
            "e3" => res.en_passant = 0x100000,
            "f3" => res.en_passant = 0x200000,
            "g3" => res.en_passant = 0x400000,
            "h3" => res.en_passant = 0x800000,
            "a6" => res.en_passant = 0x10000000000,
            "b6" => res.en_passant = 0x20000000000,
            "c6" => res.en_passant = 0x40000000000,
            "d6" => res.en_passant = 0x80000000000,
            "e6" => res.en_passant = 0x100000000000,
            "f6" => res.en_passant = 0x200000000000,
            "g6" => res.en_passant = 0x400000000000,
            "h6" => res.en_passant = 0x800000000000,
            _ => return Err(FENError::InvalidEnPassant),
        }

        res.redo_occupied();
        Ok(res)
    }

    #[inline]
    pub fn hueristic(&self) -> f32 {
        let mut total: f32 = 0.;
        total +=
            (self.white_queens.count_ones() as f32 - self.black_queens.count_ones() as f32) * 9.;
        total += (self.white_rooks.count_ones() as f32 - self.black_rooks.count_ones() as f32) * 5.;
        total +=
            (self.white_bishops.count_ones() as f32 - self.black_bishops.count_ones() as f32) * 3.;
        total +=
            (self.white_knights.count_ones() as f32 - self.black_knights.count_ones() as f32) * 3.;
        total += (self.white_pawns.count_ones() as f32 - self.black_pawns.count_ones() as f32) * 1.;

        total += self.under_attack_by_white().count_ones() as f32 * 0.25;
        total -= self.under_attack_by_black().count_ones() as f32 * 0.25;
        total
    }

    pub fn find_move(&self, depth: usize, age: usize, trans_table: &mut HashMap<Board, TransEntry>, opening_book: &OpeningBook) -> (Board, f32) {
        let score = match opening_book.get_move(self.zobrist()) {
            Some(book_move) => (self.make_move(book_move), 0.),
            None => self.iterative_search(depth, age, trans_table),
        };
        trans_table.retain(|_, v| (v.age - age) < 4);
        score
    }

    fn get_en_passant(&self) -> BitBoard {
        let square = self.en_passant.trailing_zeros();
        if self.en_passant & 0xff0000 != 0 {
            if (self.black_pawns & north_east_one(self.en_passant) != 0)
                | (self.black_pawns & north_west_one(self.en_passant) != 0)
            {
                return self.en_passant;
            }
        } else if self.en_passant & 0xff0000000000 != 0 {
            if (self.white_pawns & south_east_one(self.en_passant) != 0)
                | (self.white_pawns & south_west_one(self.en_passant) != 0)
            {
                return self.en_passant;
            }
        }
        0
    }

    pub fn piece_vector(&self) -> Vec<Piece> {
        let mut res = vec![Piece::None; 64];
        for i in BitBoardIter(self.occupied) {
            let mask = (1 as BitBoard) << i;
            if mask & self.white_kings != 0 {
                res[i] = Piece::WhiteKing;
            } 
            else if mask & self.black_kings != 0 {
                res[i] = Piece::BlackKing;
            }
            else if mask & self.white_queens != 0 {
                res[i] = Piece::WhiteQueen;
            } 
            else if mask & self.black_queens != 0 {
                res[i] = Piece::BlackQueen;
            }
            else if mask & self.white_rooks != 0 {
                res[i] = Piece::WhiteRook;
            } 
            else if mask & self.black_rooks != 0 {
                res[i] = Piece::BlackRook;
            }
            else if mask & self.white_bishops != 0 {
                res[i] = Piece::WhiteBishop;
            } 
            else if mask & self.black_bishops != 0 {
                res[i] = Piece::BlackBishop;
            }
            else if mask & self.white_knights != 0 {
                res[i] = Piece::WhiteKnight;
            } 
            else if mask & self.black_knights!= 0 {
                res[i] = Piece::BlackKnight;
            }
            else if mask & self.white_pawns != 0 {
                res[i] = Piece::WhitePawn;
            } 
            else if mask & self.black_pawns!= 0 {
                res[i] = Piece::BlackPawn;
            }
        }
        res
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

pub struct TransEntry {
    depth: usize,
    lower_bound: f32,
    upper_bound: f32,
    response: Board,
    age: usize,
}

#[derive(Debug)]
pub enum FENError {
    IncorrrectNumberOfTokens,
    IncorrrectNumberOfRanks,
    InvalidPiece(char),
    InvalidTurn(char),
    InvalidRankLength,
    InvalidTurnToken,
    InvalidEnPassant,
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