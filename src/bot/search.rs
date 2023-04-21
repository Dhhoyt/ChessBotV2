use std::{collections::HashMap, f32::INFINITY};

use super::{Board, TransEntry, CHECKMATE_THRESHOLD, CHECKMATE_VALUE};

impl Board {
    pub fn iterative_search(
        &self,
        depth: usize,
        age: usize,
        trans_table: &mut HashMap<Board, TransEntry>,
    ) -> (Board, f32) {
        for i in 1..depth {
            self.start_search(depth, age, trans_table);
        }
        self.start_search(depth, age, trans_table)
    }

    pub fn start_search(
        self,
        depth: usize,
        age: usize,
        trans_table: &mut HashMap<Board, TransEntry>,
    ) -> (Board, f32) {
        if self.white_to_play {
            Board::alpha_beta(
                self,
                depth,
                -CHECKMATE_VALUE - 2.,
                CHECKMATE_VALUE + 2.,
                true,
                age,
                trans_table,
            )
        } else {
            Board::alpha_beta(
                self,
                depth,
                -CHECKMATE_VALUE - 2.,
                CHECKMATE_VALUE + 2.,
                false,
                age,
                trans_table,
            )
        }
    }

    fn alpha_beta(
        board: Board,
        depth: usize,
        mut alpha: f32,
        mut beta: f32,
        white: bool,
        age: usize,
        trans_table: &mut HashMap<Board, TransEntry>,
    ) -> (Board, f32) {
        //println!("{}", board.to_fen());

        let lookup = trans_table.get(&board);
        match lookup {
            None => (),
            Some(result) => {
                if result.depth >= depth {
                    if result.lower_bound >= beta {
                        return (result.response, result.lower_bound);
                    }
                    if result.upper_bound <= alpha {
                        return (result.response, result.upper_bound);
                    };
                    alpha = f32::max(alpha, result.lower_bound);
                    beta = f32::min(beta, result.upper_bound);
                }
            }
        }
        if depth == 0 {
            return (board, board.hueristic());
        }
        let mut moves = if white {
            board.white_moves()
        } else {
            board.black_moves()
        };

        let len = moves.len();
        let mut value = 0.;
        let mut best_move: Option<Board> = None;
        if white {
            value = -CHECKMATE_VALUE - 3.0;
            let mut a = alpha;
            if moves.len() == 0 {
                return if board.white_kings & board.under_attack_by_black() != 0 {
                    (board, -CHECKMATE_VALUE + 2.)
                } else {
                    (board, 0.0)
                };
            }

            for i in OrderedMoves(moves) {
                let eval = Board::alpha_beta(i, depth - 1, a, beta, false, age, trans_table);
                if eval.1 > value {
                    value = eval.1;
                    best_move = Some(i);
                }

                a = f32::max(a, value);
                if value >= beta {
                    break;
                }
                
            }
        } else {
            value = CHECKMATE_VALUE + 3.0;
            let mut b = beta;
            if moves.len() == 0 {
                return if board.black_kings & board.under_attack_by_white() != 0 {
                    (board, CHECKMATE_VALUE - 2.)
                } else {
                    (board, 0.0)
                };
            }
            for i in OrderedMoves(moves) {
                let eval = Board::alpha_beta(i, depth - 1, alpha, b, true, age, trans_table);
                if eval.1 < value {
                    value = eval.1;

                    best_move = Some(i);
                }

                b = f32::min(b, value);
                if value <= alpha {
                    break;
                }
                
            }
        }
        if value < -CHECKMATE_THRESHOLD {
            value += 1.;
        }
        if value > CHECKMATE_THRESHOLD {
            value -= 1.;
        }
        match trans_table.get_mut(&board) {
            None => {
                if value <= alpha {
                    trans_table.insert(
                        board,
                        TransEntry {
                            depth: depth,
                            lower_bound: -INFINITY,
                            upper_bound: value,
                            response: best_move.unwrap(),
                            age,
                        },
                    );
                } else if (value > alpha) & (value < beta) {
                    trans_table.insert(
                        board,
                        TransEntry {
                            depth: depth,
                            lower_bound: value,
                            upper_bound: value,
                            response: best_move.unwrap(),
                            age,
                        },
                    );
                } else if value >= beta {
                    trans_table.insert(
                        board,
                        TransEntry {
                            depth: depth,
                            lower_bound: value,
                            upper_bound: INFINITY,
                            response: best_move.unwrap(),
                            age,
                        },
                    );
                }
            }
            Some(result) => {
                if result.depth <= depth {
                    if value <= alpha {
                        *result = TransEntry {
                            depth: depth,
                            lower_bound: -INFINITY,
                            upper_bound: value,
                            response: best_move.unwrap(),
                            age,
                        };
                    } else if (value > alpha) & (value < beta) {
                        *result = TransEntry {
                            depth: depth,
                            lower_bound: value,
                            upper_bound: value,
                            response: best_move.unwrap(),
                            age,
                        };
                    } else if value >= beta {
                        *result = TransEntry {
                            depth: depth,
                            lower_bound: value,
                            upper_bound: INFINITY,
                            response: best_move.unwrap(),
                            age,
                        };
                    }
                }
            }
        }
        return (best_move.unwrap(), value);
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
            if i.1 .1 > best_score {
                best_score = i.1 .1;
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
