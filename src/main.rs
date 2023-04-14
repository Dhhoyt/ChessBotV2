#![allow(unused)]
mod bot;

use std::collections::HashMap;

use bot::Board;

fn main() {
    println!("{}", perft(7, false))
    /*
    let mut score: f32 = 0.0;
    let mut trans_table = HashMap::new();
    let mut board = Board::from_fen("r5rk/5p1p/5R2/4B3/8/8/7P/7K w - - 0 1").unwrap();
    (board, score) = board.find_move_interatively(7, &mut trans_table);
    println!("{} \n {}", board.to_fen(), score);
    loop {
        let input = get_input();
        let board = Board::from_fen(&input).unwrap();
        let (board, score) = board.find_move(5, &mut trans_table);
        println!("{} \n {}", board.to_fen(), score);    
    } */
}

fn perft(depth: usize, print: bool) -> usize {
    perft_recursive(depth, Board::default(), print)
}

fn perft_recursive(depth: usize, board: Board, print: bool) -> usize {
    let mut sum = 0;
    if board.white_to_play {
        if depth == 1 {
            let moves = board.white_moves();
            if print {
                for i in moves.iter() {
                    println!("{}", i.to_fen());
                }
            }
            return moves.len();
        }
        for i in board.white_moves() {
            sum += perft_recursive(depth - 1, i, print);
        }
    } else {
        if depth == 1 {
            let moves = board.black_moves();
            if print {
                for i in moves.iter() {
                    println!("{}", i.to_fen());
                }
            }
            return moves.len();
        }
        for i in board.black_moves() {
            sum += perft_recursive(depth - 1, i, print);
        }
    }
    sum
}

pub fn print_bit_board(board: u64) {
    let mut bytes = board.to_ne_bytes();
    bytes.reverse();
    for i in bytes {
        let s = format!("{:#010b}", i);
        let s: String = s[2..s.len()].chars().rev().collect();
        println!("{}", s.replace("0", "."));
    }
    println!();
}

fn get_input() -> String {
    use std::io::{stdin,stdout,Write};
    let mut s=String::new();
    print!("Please enter some text: ");
    let _=stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }
    s
}