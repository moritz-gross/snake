use piston_window;

mod draw;
mod game;
mod snake;
mod persistence;

use crate::draw::to_coord_u32;
use crate::game::Game;

use piston_window as pw;
use piston_window::{PressEvent, UpdateEvent};

const BACK_COLOR: pw::types::Color = [0.5, 0.5, 0.5, 1.0];
const WIDTH: i32 = 15;
const HEIGHT: i32 = WIDTH;
const GAME_TITLE: &str = "Snake";

fn main() {
    let mut piston_window: pw::PistonWindow = pw_from_constants();
    let mut snake_game: Game = Game::new(WIDTH, HEIGHT);

    while let Some(event) = piston_window.next() {
        if let Some(pw::Button::Keyboard(key)) = event.press_args() {
            snake_game.key_pressed(key);
        }

        piston_window.draw_2d(&event, |c, g, _| {
            pw::clear(BACK_COLOR, g);
            snake_game.draw(&c, g);
        });

        event.update(
            |arg| {
            snake_game.update(arg.dt);
        });
    }
}

fn pw_from_constants() -> pw::PistonWindow {
    pw::WindowSettings::new(GAME_TITLE, [to_coord_u32(WIDTH), to_coord_u32(HEIGHT)])
        .exit_on_esc(true)
        .build()
        .expect("Failed to create window")
}
