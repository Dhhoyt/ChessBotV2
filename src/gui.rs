use std::time::Duration;
use rand::Rng;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Point, Rect};
use sdl2::image::{self, LoadTexture, InitFlag};
use sdl2::render::{BlendMode, TextureAccess};
use sdl2::mouse::{MouseButton, self};

use crate::{Board, Piece};

const SQUARE_SIZE: u32 = 128;
const PADDING: u32 = 64;
const BORDER_WIDTH: u32 = 32;

const SPRITE_SIZE: u32 = 426;
const WINDOW_SIZE: u32 = (SQUARE_SIZE * 8) + (PADDING * 2) + (BORDER_WIDTH * 2);

pub fn start_gui(board: Board) -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chess Bot", WINDOW_SIZE, WINDOW_SIZE).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.load_texture_bytes(include_bytes!("assets/pieces.png"))
        .map_err(|err| format!("failed to load spritesheet surface: {}", err.to_string()))?;

    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let mut mouse_held_down = false;

    'running: loop {
        canvas.set_draw_color(Color::RGB(13, 27, 42));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        
        //Border
        canvas.set_draw_color(Color::RGB(27, 38, 59));
        canvas.fill_rect(Rect::new(PADDING as i32, PADDING as i32, WINDOW_SIZE - (2 * PADDING), WINDOW_SIZE - (2 * PADDING)));

        //Draw squares 
        let pieces = board.piece_vector();
        for file in 0..8 {
            for rank in 0..8 {
                let x_offset = (SQUARE_SIZE * file + PADDING) as i32 + BORDER_WIDTH as i32;
                let y_offset = (SQUARE_SIZE * rank + PADDING) as i32 + BORDER_WIDTH as i32;
                if (file + rank) % 2 == 0 {
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
                let square = ( 7 - rank ) * 8 + file;
                let src_rect = pieces[square as usize].offset();
                canvas.copy(&texture, src_rect, dest_rect);
            }
        }

        check_for_click(&event_pump, &mut mouse_held_down);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32/60));
    }

    Ok(())
}

fn check_for_click(e: &sdl2::EventPump, mouse_held_down: &mut bool) {
    if e.mouse_state().left() {
        if !*mouse_held_down {
            let x_square = (e.mouse_state().x() - PADDING as i32 - BORDER_WIDTH as i32)/(SQUARE_SIZE as i32);
            let y_square = (e.mouse_state().y() - PADDING as i32 - BORDER_WIDTH as i32)/(SQUARE_SIZE as i32);
            println!("({}, {})", x_square, y_square);
            *mouse_held_down = true;
        }
    } else {
        *mouse_held_down = false;
    }
}

impl Piece {
    fn offset (&self) -> Rect {
        let offset = match *self {
            Self::WhitePawn => (5,0),
            Self::BlackPawn => (5,1),
            Self::WhiteKnight => (3,0),
            Self::BlackKnight => (3,1),
            Self::WhiteBishop => (2,0),
            Self::BlackBishop => (2,1),
            Self::WhiteRook => (4,0),
            Self::BlackRook => (4,1),
            Self::WhiteQueen => (1,0),
            Self::BlackQueen => (1,1),
            Self::WhiteKing => (0,0),
            Self::BlackKing => (0,1),
            Self::None => (10,10),
        };
        Rect::new(SPRITE_SIZE as i32 * offset.0, SPRITE_SIZE as i32 * offset.1, SPRITE_SIZE, SPRITE_SIZE)
    }
}