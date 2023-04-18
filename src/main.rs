#![allow(unused)]
mod bot;

use std::collections::HashMap;

use bot::{Board, opening::OpeningBook};

fn main() {
    let starting = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap().zobrist();
    let book = OpeningBook::new("books/codekiddy.bin");
    println!("{}", book.moves.get(&starting).unwrap().len());
    let book_move = book.get_move(starting).unwrap();
    println!("({}, {})", book_move.from_square, book_move.to_square);
    bot();
}

fn bot() {
    let mut trans_table = HashMap::new();
    let mut board = Board::default();
    let mut score: f32 = 0.0;

    let book = OpeningBook::new("books/codekiddy.bin");

    (board, score) = board.find_move(6, &mut trans_table, &book);

    println!("{} \n {}", board.to_fen(), score);
    loop {
        let input = get_input();
        let board = Board::from_fen(&input).unwrap();
        let (board, score) = board.find_move(6, &mut trans_table, &book);
        println!("{} \n {}", board.to_fen(), score);
    }
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
                    println!("{}", i.0.to_fen());
                }
            }
            return moves.len();
        }
        for i in board.white_moves() {
            sum += perft_recursive(depth - 1, i.0, print);
        }
    } else {
        if depth == 1 {
            let moves = board.black_moves();
            if print {
                for i in moves.iter() {
                    println!("{}", i.0.to_fen());
                }
            }
            return moves.len();
        }
        for i in board.black_moves() {
            sum += perft_recursive(depth - 1, i.0, print);
        }
    }
    sum
}

fn get_input() -> String {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    print!("Please enter some text: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}
