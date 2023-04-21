use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::mouse::{self, MouseButton};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, TextureAccess};

use crate::bot::TransEntry;
use crate::bot::board_move::{self, Move, PromotionPiece};
use crate::bot::opening::OpeningBook;
use crate::{Board, Piece};

const SQUARE_SIZE: u32 = 78;
const PADDING: u32 = 64;
const BORDER_WIDTH: u32 = 32;

const SPRITE_SIZE: u32 = 426;
const WINDOW_SIZE: u32 = (SQUARE_SIZE * 8) + (PADDING * 2) + (BORDER_WIDTH * 2);

pub fn start_gui(board: Board) -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chess Bot", WINDOW_SIZE, WINDOW_SIZE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .load_texture_bytes(include_bytes!("assets/pieces.png"))
        .map_err(|err| format!("failed to load spritesheet surface: {}", err.to_string()))?;

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut gui_state = GuiState {
        mouse_held_down: false,
        selected_square: (-1, -1),
        board: board,
        valid_moves: Vec::new(),
        book: OpeningBook::new(include_bytes!("books/Elo2400.bin").to_vec()),
        trans_table: HashMap::new(),
        age: 0,
    };

    'running: loop {
        canvas.set_draw_color(Color::RGB(13, 27, 42));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        //Border
        canvas.set_draw_color(Color::RGB(27, 38, 59));
        canvas.fill_rect(Rect::new(
            PADDING as i32,
            PADDING as i32,
            WINDOW_SIZE - (2 * PADDING),
            WINDOW_SIZE - (2 * PADDING),
        ));

        //Draw squares
        let pieces = gui_state.board.piece_vector();
        gui_state.valid_moves = if gui_state.selected_square == (-1, -1) {
            Vec::new()
        } else {
            gui_state.board.valid_moves(
                ((7 - gui_state.selected_square.1) * 8 + gui_state.selected_square.0) as usize,
            )
        };
        for file in 0..8 {
            for rank in 0..8 {
                let x_offset = (SQUARE_SIZE * file + PADDING) as i32 + BORDER_WIDTH as i32;
                let y_offset = (SQUARE_SIZE * rank + PADDING) as i32 + BORDER_WIDTH as i32;
                if gui_state.selected_square == (file as i32, rank as i32) {
                    canvas.set_draw_color(Color::RGB(65, 0, 0));
                } else if gui_state
                    .valid_moves
                    .contains(&(((7 - rank) * 8 + file) as usize))
                {
                    canvas.set_draw_color(Color::RGB(65, 65, 65));
                } else if (file + rank) % 2 == 0 {
                    //White
                    canvas.set_draw_color(Color::RGB(224, 225, 221));
                } else {
                    //Black
                    canvas.set_draw_color(Color::RGB(65, 90, 119));
                }
                canvas.fill_rect(Rect::new(x_offset, y_offset, SQUARE_SIZE, SQUARE_SIZE));
            }
        }

        //Draw Pieces
        for file in 0..8 {
            for rank in 0..8 {
                let x_offset = (SQUARE_SIZE * file + PADDING) as i32 + BORDER_WIDTH as i32;
                let y_offset = (SQUARE_SIZE * rank + PADDING) as i32 + BORDER_WIDTH as i32;
                let dest_rect = Rect::new(x_offset, y_offset, SQUARE_SIZE, SQUARE_SIZE);
                let square = (7 - rank) * 8 + file;
                let src_rect = pieces[square as usize].offset();
                canvas.copy(&texture, src_rect, dest_rect);
            }
        }

        check_for_click(&event_pump, &mut gui_state);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

fn check_for_click(e: &sdl2::EventPump, gui_state: &mut GuiState) {
    if e.mouse_state().left() {
        if !gui_state.mouse_held_down {
            gui_state.mouse_held_down = true;
            if ((e.mouse_state().x() - PADDING as i32 - BORDER_WIDTH as i32) < 0)
                | ((e.mouse_state().y() - PADDING as i32 - BORDER_WIDTH as i32) < 0)
            {
                gui_state.selected_square = (-1, -1);
                return;
            }
            let x_square =
                (e.mouse_state().x() - PADDING as i32 - BORDER_WIDTH as i32) / (SQUARE_SIZE as i32);
            let y_square =
                (e.mouse_state().y() - PADDING as i32 - BORDER_WIDTH as i32) / (SQUARE_SIZE as i32);
            if (x_square < 8) & (y_square < 8) {
                if gui_state
                    .valid_moves
                    .contains(&(((7 - y_square) * 8 + x_square) as usize))
                {
                    let new_move = Move {
                        to_square: ((7 - y_square) * 8 + x_square) as u16,
                        from_square: ((7 - gui_state.selected_square.1) * 8
                            + gui_state.selected_square.0)
                            as u16,
                        promotion_piece: PromotionPiece::None,
                        weight: 0,
                    };
                    gui_state.board = gui_state.board.make_move(new_move);
                    gui_state.board = gui_state.board.find_move(6, gui_state.age, &mut gui_state.trans_table, &gui_state.book).0;
                    gui_state.age += 1;
                    println!("{} {}", gui_state.age, gui_state.trans_table.len());
                    gui_state.selected_square = (-1, -1);
                } else {
                    gui_state.selected_square = (x_square, y_square);
                }
            } else {
                gui_state.selected_square = (-1, -1);
            }
        }
    } else {
        gui_state.mouse_held_down = false;
    }
}

impl Piece {
    fn offset(&self) -> Rect {
        let offset = match *self {
            Self::WhitePawn => (5, 0),
            Self::BlackPawn => (5, 1),
            Self::WhiteKnight => (3, 0),
            Self::BlackKnight => (3, 1),
            Self::WhiteBishop => (2, 0),
            Self::BlackBishop => (2, 1),
            Self::WhiteRook => (4, 0),
            Self::BlackRook => (4, 1),
            Self::WhiteQueen => (1, 0),
            Self::BlackQueen => (1, 1),
            Self::WhiteKing => (0, 0),
            Self::BlackKing => (0, 1),
            Self::None => (-1, -1),
        };
        Rect::new(
            SPRITE_SIZE as i32 * offset.0,
            SPRITE_SIZE as i32 * offset.1,
            SPRITE_SIZE,
            SPRITE_SIZE,
        )
    }
}

struct GuiState {
    mouse_held_down: bool,
    selected_square: (i32, i32),
    board: Board,
    valid_moves: Vec<usize>,
    book: OpeningBook,
    trans_table: HashMap<Board, TransEntry>,
    age: usize
}
