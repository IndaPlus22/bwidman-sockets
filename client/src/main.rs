/**
 * Chess GUI template.
 * Author: Isak Larsson <isaklar@kth.se>
 * Last updated: 2022-09-29
 */
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::collections::HashMap;
use std::io::{Write, Read};
use std::net::{TcpStream, Shutdown};

use chess_template::{Colour, Game, PieceType, Piece, Position, GameState};

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use piston::{Button, Key, MouseButton, MouseCursorEvent, PressEvent, ReleaseEvent, Window as pWindow};

/// A chess board is 8x8 tiles.
const GRID_SIZE: i16 = 8;
/// Sutible size of each tile.
const GRID_CELL_SIZE: (i16, i16) = (90, 90);

/// Size of the application window.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE as f32 * GRID_CELL_SIZE.1 as f32 + 40.0,
);

// GUI Color representations
const BLACK: [f32; 4] = [228.0 / 255.0, 196.0 / 255.0, 108.0 / 255.0, 1.0];
const WHITE: [f32; 4] = [188.0 / 255.0, 140.0 / 255.0, 76.0 / 255.0, 1.0];

pub struct App<'a> {
    gl: GlGraphics,                                 // OpenGL drawing backend.
    window: &'a mut Window,
    mouse_pos: [f64; 2],                            // Current mouse postition
    left_click: bool,                               // Indicates if left mouse button is pressed
    moving_piece: Option<(i16, i16)>,               // Contains the coordinates of the piece being moved, None if none is being moved
    sprites: HashMap<Piece, Texture>, // For easy access to the apropriate PNGs
    game: Game, // Save piece positions, which tiles has been clicked, current colour, etc...
}

impl App<'_> {
    fn new(opengl: OpenGL, window: &mut Window) -> App {
        App {
            gl: GlGraphics::new(opengl),
            window,
            mouse_pos: [0., 0.],
            left_click: false,
            moving_piece: None,
            game: Game::new(),
            sprites: Self::load_sprites(),
        }
    }

    fn render(&mut self, args: &RenderArgs, glyphs: &mut GlyphCache) {
        use graphics::*; // Now we don't have to use this everytime :D

        let square = rectangle::square(0.0, 0.0, GRID_CELL_SIZE.0 as f64);

        let mouse_pos = self.mouse_cell();
        
        self.gl.draw(args.viewport(), |c, gl| {
            let board = self.game.get_board();
            // Clear the screen.
            clear([0.3, 0.3, 0.3, 1.0], gl);
            // Draw tiles
            for row in 0..8 {
                for col in 0..8 {
                    rectangle(
                        match col % 2 {
                            0 => if row % 2 == 0 { BLACK } else { WHITE },
                            _ => if row % 2 == 0 { WHITE } else { BLACK }
                        },
                        square,
                        c.transform.trans(
                            (col * GRID_CELL_SIZE.0) as f64,
                            (row * GRID_CELL_SIZE.1) as f64,
                        ),
                        gl,
                    );
                }
            }

            // Draw pieces
            for row in 0..8 {
                for col in 0..8 {
                    if let Some(piece) = board[(row * 8 + col) as usize] {
                        let img = Image::new().rect(square);

                        let correct_cell_clicked = mouse_pos == (col, row);
                        let piece_moving = self.moving_piece.is_some() && self.moving_piece.unwrap() == (col, row);
                        
                        let cursor_not_on_other_piece = mouse_pos != (col, row);
                        
                        if (self.left_click && correct_cell_clicked) || // Piece got clicked on
                        (piece_moving && cursor_not_on_other_piece) { // Piece is being moved
                            self.left_click = false;
                            self.moving_piece = Some((col, row));
                            
                            // Draw dots on possible moves
                            let moves = self.game.get_possible_moves(Position::new(row as usize, col as usize).ok().unwrap(), 0);
                            let circle_mask = rectangle::square(0.0, 0.0, 20.0);

                            for position in moves {
                                ellipse(
                                    [0.5, 0.5, 0.5, 0.5],
                                    rectangle::centered(circle_mask),
                                    c.transform.trans(
                                        (position.col as i16 * GRID_CELL_SIZE.0 + GRID_CELL_SIZE.0 / 2) as f64,
                                        (position.row as i16 * GRID_CELL_SIZE.1 + GRID_CELL_SIZE.1 / 2) as f64
                                    ),
                                    gl
                                );
                            }
                            
                            // Follow mouse
                            img.draw(
                                self.sprites.get(&piece).unwrap(),
                                &c.draw_state,
                                c.transform.trans(
                                    self.mouse_pos[0] - GRID_CELL_SIZE.0 as f64 / 2.0,
                                    self.mouse_pos[1] - GRID_CELL_SIZE.1 as f64 / 2.0,
                                ),
                                gl,
                            );
                        } else {
                            img.draw(
                                self.sprites.get(&piece).unwrap(),
                                &c.draw_state,
                                c.transform.trans(
                                    (col * GRID_CELL_SIZE.0) as f64,
                                    (row * GRID_CELL_SIZE.0) as f64,
                                ),
                                gl,
                            );
                        }
                    }
                }
            }

            // Write game state
            let state_text = format!("Game state: {:?}", self.game.get_game_state());
            let state_text_postition = c.transform.trans(
                10.0,
                (SCREEN_SIZE.1 - 10.0) as f64,
            );
            text::Text::new_color([1.0, 1.0, 1.0, 1.0], 24)
                .draw(&state_text, glyphs, &c.draw_state, state_text_postition, gl)
                .unwrap();
            
            // Write who's turn it is
            let turn_text = format!("Turn: {:?}", self.game.get_active_colour());
            let turn_text_postition = c.transform.trans(
                (SCREEN_SIZE.0 - 160.0) as f64,
                (SCREEN_SIZE.1 - 10.0) as f64,
            );
            text::Text::new_color([1.0, 1.0, 1.0, 1.0], 24)
                .draw(&turn_text, glyphs, &c.draw_state, turn_text_postition, gl)
                .unwrap();

            let mut center_text = |text: &str, size: u32, dy: f32| {
                let text_size: (f32, f32) = ((size / 2 * text.len() as u32) as f32, size as f32);
                let text_postition = c.transform.trans(
                    (SCREEN_SIZE.0 / 2.0 - text_size.0 / 2.0) as f64,
                    (SCREEN_SIZE.1 / 2.0 - text_size.1 / 2.0 + dy) as f64,
                );
                text::Text::new_color([1.0, 1.0, 1.0, 1.0], size)
                    .draw(text, glyphs, &c.draw_state, text_postition, gl)
                    .unwrap();
            };
            
            // Announce winner
            if self.game.get_game_state() == GameState::GameOver {
                let text = format!("{:?} is the winner!", self.game.get_active_colour());
                center_text(&text, 45, 0.0);
            }

            // Set promotion
            if self.game.get_game_state() == GameState::WaitingOnPromotionChoice {
                center_text("Promotion!", 30, -75.0);
                center_text("Choose new piece with number keys:", 30, -45.0);
                center_text("1. Queen", 30, -15.0);
                center_text("2. Rook", 30, 15.0);
                center_text("3. Bishop", 30, 45.0);
                center_text("4. Knight", 30, 75.0);
                
                let choice: String;
                if let Some(key) = self.window.wait_event().press_args() {
                    choice = match key {
                        Button::Keyboard(Key::D1) => String::from("queen"),
                        Button::Keyboard(Key::D2) => String::from("rook"),
                        Button::Keyboard(Key::D3) => String::from("bishop"),
                        Button::Keyboard(Key::D4) => String::from("knight"),
                        _ => String::from(""),
                    };
                    self.game.set_promotion(choice).unwrap();
                }
            }
        });
    }

    #[rustfmt::skip]
    /// Loads chess piese images into vector.
    fn load_sprites() -> HashMap<Piece, Texture> {
        use Colour::*;
        use PieceType::*;
        [
            (Piece { colour: Black, piece_type: King }, "resources/black_king.png".to_string()),
            (Piece { colour: Black, piece_type: Queen }, "resources/black_queen.png".to_string()),
            (Piece { colour: Black, piece_type: Rook }, "resources/black_rook.png".to_string()),
            (Piece { colour: Black, piece_type: Pawn }, "resources/black_pawn.png".to_string()),
            (Piece { colour: Black, piece_type: Bishop }, "resources/black_bishop.png".to_string()),
            (Piece { colour: Black, piece_type: Knight }, "resources/black_knight.png".to_string()),
            (Piece { colour: White, piece_type: King }, "resources/white_king.png".to_string()),
            (Piece { colour: White, piece_type: Queen }, "resources/white_queen.png".to_string()),
            (Piece { colour: White, piece_type: Rook }, "resources/white_rook.png".to_string()),
            (Piece { colour: White, piece_type: Pawn }, "resources/white_pawn.png".to_string()),
            (Piece { colour: White, piece_type: Bishop }, "resources/white_bishop.png".to_string()),
            (Piece { colour: White, piece_type: Knight }, "resources/white_knight.png".to_string())
        ]
            .iter()
            .map(|(piece, path)| {
                (*piece, Texture::from_path(path, &TextureSettings::new()).unwrap())
            })
            .collect::<HashMap<Piece, Texture>>()
    }

    /// Returns which cell the mouse is in
    fn mouse_cell(&self) -> (i16, i16) {
        (
            (self.mouse_pos[0] / GRID_CELL_SIZE.0 as f64).floor() as i16, 
            (self.mouse_pos[1] / GRID_CELL_SIZE.1 as f64).floor() as i16
        )
    }
}

fn assign_color(tcp_stream: &mut TcpStream) -> Colour {
    let mut buffer = vec![0u8; 16];
    tcp_stream.read(&mut buffer).unwrap();

    if String::from_utf8(buffer).unwrap() == "white" {
        return Colour::White;
    }
    return Colour::Black;
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window =
        WindowSettings::new("Chess", [SCREEN_SIZE.0 as f64, SCREEN_SIZE.1 as f64])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Initialize our app state
    let mut app = App::new(opengl, &mut window);

    // Initialize font
    let mut glyphs = GlyphCache::new(
        "resources/AbyssinicaSIL-Regular.ttf",
        (),
        TextureSettings::new(),
    )
    .unwrap();

    let mut tcp_stream = TcpStream::connect("127.0.0.1:6969").unwrap();
    // prevent io stream operation from blocking socket in case of slow communication
    tcp_stream.set_nonblocking(true).expect("Failed to initiate non-blocking!");

    let color = assign_color(&mut tcp_stream);
    
    let mut events = Events::new(EventSettings::new());
    // Our "game loop". Will run until we exit the window
    while let Some(e) = events.next(app.window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &mut glyphs);
        }
        if let Some(pos) = e.mouse_cursor_args() {
            app.mouse_pos = pos;
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            app.left_click = true;
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
            // If a piece is being moved
            if let Some(pos) = app.moving_piece {
                let mouse_pos = app.mouse_cell();
                
                let from = Position::new(pos.1 as usize, pos.0 as usize).ok().unwrap();
                let to = Position::new(mouse_pos.1 as usize, mouse_pos.0 as usize).ok().unwrap();
                
                let move_result = app.game.make_move_pos(from, to);
                app.moving_piece = None;
                
                // Move was legal, send it to server
                if move_result.is_ok() {
                    let move_msg = format!("{} {}", from.idx, to.idx);
                    tcp_stream.write_all(move_msg.as_bytes()).unwrap();
                    tcp_stream.flush().unwrap();
                }
            }
        }

        if app.game.get_active_colour() != color {
            // Get update of other player from server
            let mut buffer = vec![0u8; 5];
            tcp_stream.read_exact(&mut buffer).unwrap();

            // Convert read data into move indices
            let moves = String::from_utf8(buffer).unwrap();
            let mut moves_split = moves.split(" ");

            let from = moves_split.next().unwrap().parse().unwrap();
            let to = moves_split.next().unwrap().parse().unwrap();

            // Make move for opponent on this client
            app.game.make_move_pos(Position::new_from_idx(from).unwrap(), Position::new_from_idx(to).unwrap()).unwrap();
        }
    }

    tcp_stream.shutdown(Shutdown::Both).unwrap();
}
