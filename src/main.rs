#![allow(unused)]
mod bot;
mod gui;

use bot::{opening::OpeningBook, Board};
use std::collections::HashMap;

fn main() {
    gui::start_gui(Board::default());
    bot();
}

fn bot() {
    let mut trans_table = HashMap::new();
    let mut board = Board::default();
    let mut score: f32 = 0.0;
    let mut age = 0;

    let book = OpeningBook::new(include_bytes!("books/codekiddy.bin").to_vec());

    (board, score) = board.find_move(8, age, &mut trans_table, &book);

    println!("{} \n {}", board.to_fen(), score);
    loop {
        let input = get_input();
        let board = Board::from_fen(&input).unwrap();
        let (board, score) = board.find_move(8, age, &mut trans_table, &book);
        println!("{} \n {}", board.to_fen(), score);
    }
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

#[derive(Clone, Copy)]
pub enum Piece {
    WhitePawn,
    BlackPawn,
    WhiteKnight,
    BlackKnight,
    WhiteBishop,
    BlackBishop,
    WhiteRook,
    BlackRook,
    WhiteQueen,
    BlackQueen,
    WhiteKing,
    BlackKing,
    None,
}
