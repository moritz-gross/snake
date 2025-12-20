mod audio;
mod draw;
mod game;
mod persistence;
mod snake;

use crate::audio::SoundPlayer;
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

    // Load font for score display - try common Windows font paths
    let font_path = find_font();
    let mut glyphs = piston_window
        .load_font(font_path)
        .expect("Failed to load font");

    let sound_player = SoundPlayer::new();
    let mut snake_game: Game = Game::new(WIDTH, HEIGHT, sound_player);

    while let Some(event) = piston_window.next() {
        if let Some(pw::Button::Keyboard(key)) = event.press_args() {
            snake_game.key_pressed(key);
        }

        piston_window.draw_2d(&event, |c, g, device| {
            pw::clear(BACK_COLOR, g);
            snake_game.draw(&c, g, &mut glyphs);
            glyphs.factory.encoder.flush(device);
        });

        event.update(|arg| {
            snake_game.update(arg.dt);
        });
    }
}

fn find_font() -> std::path::PathBuf {
    // Try common font locations on Windows
    let candidates = [
        "C:/Windows/Fonts/consola.ttf",    // Consolas (monospace)
        "C:/Windows/Fonts/arial.ttf",      // Arial
        "C:/Windows/Fonts/segoeui.ttf",    // Segoe UI
        "C:/Windows/Fonts/cour.ttf",       // Courier New
    ];

    for path in candidates {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return p;
        }
    }

    // Fallback - will likely fail but gives a clear error message
    std::path::PathBuf::from("C:/Windows/Fonts/arial.ttf")
}

fn pw_from_constants() -> pw::PistonWindow {
    pw::WindowSettings::new(GAME_TITLE, [to_coord_u32(WIDTH), to_coord_u32(HEIGHT)])
        .exit_on_esc(true)
        .build()
        .expect("Failed to create window")
}
