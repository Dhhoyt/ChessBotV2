#![allow(unused)]
mod bot;

use std::collections::HashMap;
use bot::{Board, opening::OpeningBook};

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;
use sdl2::image::{self, LoadTexture, InitFlag};

const SQUARE_SIZE: u32 = 64;
const PADDING: u32 = 32;


fn main() {
    bot();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chess Bot", (SQUARE_SIZE * 8) + (PADDING * 2), (SQUARE_SIZE * 8) + (PADDING * 2)).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0,255,255));

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(50,50,64 ));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        
        //Draw squares 
        for file in 0..8 {
            for rank in 0..8 {
                let x_offset = (SQUARE_SIZE * file + PADDING) as i32;
                let y_offset = (SQUARE_SIZE * rank + PADDING) as i32;
                if (file + rank) % 2 == 0 {
                    canvas.set_draw_color(Color::RGB(255,255,220 ));
                } else {
                    canvas.set_draw_color(Color::RGB(128,255,128 ));
                }
                canvas.fill_rect(Rect::new(x_offset, y_offset, SQUARE_SIZE, SQUARE_SIZE));
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32/60));
    }

}

fn bot() {
    let mut trans_table = HashMap::new();
    let mut board = Board::default();
    let mut score: f32 = 0.0;
    let mut age = 0;

    let book = OpeningBook::new(include_bytes!("books/codekiddy.bin").to_vec());

    (board, score) = board.find_move(7, age, &mut trans_table, &book);

    println!("{} \n {}", board.to_fen(), score);
    loop {
        let input = get_input();
        let board = Board::from_fen(&input).unwrap();
        let (board, score) = board.find_move(7, age, &mut trans_table, &book);
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
