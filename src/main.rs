mod audio;
mod debug;
mod draw;
mod game;
mod persistence;
mod snake;
#[cfg(feature = "spectator")]
mod spectator;

use crate::audio::SoundPlayer;
use crate::draw::to_coord_u32;
use crate::game::Game;
#[cfg(feature = "spectator")]
use crate::spectator as spectator_server;

use piston_window as pw;
use piston_window::{PressEvent, UpdateEvent};
use piston_window::graphics::Transformed;

const BACK_COLOR: pw::graphics::types::Color = [0.5, 0.5, 0.5, 1.0];
const WIDTH: i32 = 15;
const HEIGHT: i32 = WIDTH;
const GAME_TITLE: &str = "Snake";

fn main() {
    let mut piston_window: pw::PistonWindow = pw_from_constants();

    // Load font for score display - try common Windows font paths
    let font_path = find_font();
    let mut glyphs = piston_window
        .load_font(font_path, pw::wgpu_graphics::TextureSettings::new())
        .expect("Failed to load font");

    let sound_player = SoundPlayer::new();
    let mut snake_game: Game = Game::new(WIDTH, HEIGHT, sound_player);
    #[cfg(feature = "spectator")]
    let spectator = spectator_server::start("0.0.0.0:9001");
    #[cfg(feature = "spectator")]
    spectator_server::start_http("0.0.0.0:8000");

    let base_width = to_coord_u32(WIDTH) as f64;
    let base_height = to_coord_u32(HEIGHT) as f64;

    while let Some(event) = piston_window.next() {
        if let Some(pw::Button::Keyboard(key)) = event.press_args() {
            snake_game.key_pressed(key);
        }

        piston_window.draw_2d(&event, |c, g, _device| {
            pw::graphics::clear(BACK_COLOR, g);
            let (win_w, win_h) = c.viewport
                .map(|vp| (vp.window_size[0], vp.window_size[1]))
                .unwrap_or((base_width, base_height));
            let scale_x = win_w / base_width;
            let scale_y = win_h / base_height;
            let scaled_context = pw::graphics::Context {
                transform: c.transform.scale(scale_x, scale_y),
                ..c
            };
            snake_game.draw(&scaled_context, g, &mut glyphs);
        });

        event.update(|arg| {
            snake_game.update(arg.dt);
            #[cfg(feature = "spectator")]
            spectator.send(snake_game.game_snapshot());
        });
    }
}

fn find_font() -> std::path::PathBuf {
    // Try common font locations on macOS and Windows
    let candidates = [

        // macOS system fonts
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Supplemental/Courier New.ttf",
        "/Library/Fonts/Arial.ttf",

        // Windows fonts
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
    std::path::PathBuf::from("/System/Library/Fonts/Helvetica.ttc")
}

fn pw_from_constants() -> pw::PistonWindow {
    pw::WindowSettings::new(GAME_TITLE, [to_coord_u32(WIDTH), to_coord_u32(HEIGHT)])
        .resizable(true)
        .exit_on_esc(true)
        .build()
        .expect("Failed to create window")
}
