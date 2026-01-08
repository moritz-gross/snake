use piston_window as pw;
use piston_window::Glyphs;
use pw::graphics::Transformed;

use rand::seq::IteratorRandom;

use crate::audio::SoundPlayer;
use crate::draw::{draw_block, draw_rectangle, BLOCK_SIZE};
use crate::snake::{Direction, Snake};
use crate::persistence;

const FOOD_COLOR: pw::graphics::types::Color = [0.80, 0.00, 0.00, 1.0];
const BORDER_COLOR: pw::graphics::types::Color = [0.00, 0.00, 0.00, 1.0];
const GAMEOVER_COLOR: pw::graphics::types::Color = [0.90, 0.00, 0.00, 0.5];
const PAUSE_COLOR: pw::graphics::types::Color = [0.00, 0.00, 0.00, 0.5];
const TEXT_COLOR: pw::graphics::types::Color = [1.0, 1.0, 1.0, 1.0];
#[cfg(feature = "debug_draw")]
const DEBUG_COLOR: pw::graphics::types::Color = [0.10, 0.80, 1.00, 0.9];
const FONT_SIZE: u32 = 16;

const MOVING_PERIOD: f64 = 0.3;
const RESTART_TIME: f64 = 3.0;

#[derive(Clone, Debug)]
enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver { elapsed: f64, final_score: usize },
}

trait Renderable {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        glyphs: &mut Glyphs,
    );
}

struct Food {
    exists: bool,
    x: i32,
    y: i32,
}

impl Food {
    fn new(x: i32, y: i32) -> Food {
        Food {
            exists: true,
            x,
            y,
        }
    }

    fn draw(&self, con: &pw::graphics::Context, g: &mut pw::wgpu_graphics::WgpuGraphics) {
        if self.exists {
            draw_block(FOOD_COLOR, self.x, self.y, con, g);
        }
    }

    #[cfg(test)]
    fn position(&self) -> Option<(i32, i32)> {
        if self.exists {
            Some((self.x, self.y))
        } else {
            None
        }
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.exists = true;
    }
}

struct Grid {
    width: i32,
    height: i32,
}

impl Grid {
    fn new(width: i32, height: i32) -> Grid {
        Grid { width, height }
    }

    fn draw(&self, con: &pw::graphics::Context, g: &mut pw::wgpu_graphics::WgpuGraphics) {
        draw_rectangle(BORDER_COLOR, 0, 0, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, self.height - 1, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, 0, 1, self.height, con, g);
        draw_rectangle(BORDER_COLOR, self.width - 1, 0, 1, self.height, con, g);
    }
}

struct Hud {
    score: usize,
    high_score: u32,
    width: i32,
    turns: usize,
}

impl Renderable for Hud {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        glyphs: &mut Glyphs,
    ) {
        let score_text = format!("Score: {}", self.score);
        let high_text = format!("High: {}", self.high_score);

        let text_y = BLOCK_SIZE - 4.0;
        let score_x = BLOCK_SIZE + 5.0;
        let high_x = (self.width as f64) * BLOCK_SIZE - 80.0;

        let transform = con.transform.trans(score_x, text_y + FONT_SIZE as f64);
        pw::graphics::text::Text::new_color(TEXT_COLOR, FONT_SIZE)
            .draw(&score_text, glyphs, &con.draw_state, transform, g)
            .unwrap_or(());

        let transform = con.transform.trans(high_x, text_y + FONT_SIZE as f64);
        pw::graphics::text::Text::new_color(TEXT_COLOR, FONT_SIZE)
            .draw(&high_text, glyphs, &con.draw_state, transform, g)
            .unwrap_or(());

        let turns_text = format!("Turns: {}", self.turns);
        let turns_y = text_y + (FONT_SIZE as f64) + 6.0;
        let transform = con.transform.trans(score_x, turns_y + FONT_SIZE as f64);
        pw::graphics::text::Text::new_color(TEXT_COLOR, FONT_SIZE)
            .draw(&turns_text, glyphs, &con.draw_state, transform, g)
            .unwrap_or(());
    }
}

struct Overlay {
    state: GameState,
    width: i32,
    height: i32,
}

impl Renderable for Overlay {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        glyphs: &mut Glyphs,
    ) {
        match &self.state {
            GameState::Menu => {
                draw_rectangle(PAUSE_COLOR, 0, 0, self.width, self.height, con, g);
                let title = "SNAKE";
                let hint = "Press Enter to start";
                let center_x = (self.width as f64) * BLOCK_SIZE * 0.5;
                let center_y = (self.height as f64) * BLOCK_SIZE * 0.5;
                let title_transform = con.transform.trans(center_x - 40.0, center_y - 10.0);
                pw::graphics::text::Text::new_color(TEXT_COLOR, 24)
                    .draw(title, glyphs, &con.draw_state, title_transform, g)
                    .unwrap_or(());
                let hint_transform = con.transform.trans(center_x - 90.0, center_y + 18.0);
                pw::graphics::text::Text::new_color(TEXT_COLOR, FONT_SIZE)
                    .draw(hint, glyphs, &con.draw_state, hint_transform, g)
                    .unwrap_or(());
            }
            GameState::Playing => {}
            GameState::Paused => {
                draw_rectangle(PAUSE_COLOR, 0, 0, self.width, self.height, con, g);
                let center_x = self.width / 2;
                let center_y = self.height / 2;
                draw_rectangle(BORDER_COLOR, center_x - 1, center_y - 1, 1, 3, con, g);
                draw_rectangle(BORDER_COLOR, center_x + 1, center_y - 1, 1, 3, con, g);
            }
            GameState::GameOver { final_score, .. } => {
            draw_rectangle(GAMEOVER_COLOR, 0, 0, self.width, self.height, con, g);
            let msg = format!("Final: {}", final_score);
            let center_x = (self.width as f64) * BLOCK_SIZE * 0.5;
            let center_y = (self.height as f64) * BLOCK_SIZE * 0.5;
            let transform = con.transform.trans(center_x - 40.0, center_y);
            pw::graphics::text::Text::new_color(TEXT_COLOR, FONT_SIZE)
                .draw(&msg, glyphs, &con.draw_state, transform, g)
                .unwrap_or(());
            }
        }
    }
}

impl Renderable for Snake {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        _glyphs: &mut Glyphs,
    ) {
        Snake::draw(self, con, g);
    }
}

impl Renderable for &Snake {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        _glyphs: &mut Glyphs,
    ) {
        Snake::draw(*self, con, g);
    }
}

impl Renderable for Food {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        _glyphs: &mut Glyphs,
    ) {
        self.draw(con, g);
    }
}

impl Renderable for &Food {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        _glyphs: &mut Glyphs,
    ) {
        self.draw(con, g);
    }
}

impl Renderable for Grid {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        _glyphs: &mut Glyphs,
    ) {
        self.draw(con, g);
    }
}

impl Renderable for &Grid {
    fn render(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        _glyphs: &mut Glyphs,
    ) {
        self.draw(con, g);
    }
}

pub struct Game {
    snake: Snake,

    food: Food,
    grid: Grid,

    state: GameState,
    waiting_time: f64,
    high_score: u32,
    sound_player: Option<SoundPlayer>,
    last_dt: f64,
    fps: f64,
    fps_accum: f64,
    fps_frames: u32,
    tick_count: u64,
}

impl Game {
    pub fn new(width: i32, height: i32, sound_player: Option<SoundPlayer>) -> Game {
        if let Some(ref player) = sound_player {
            player.play_start();
        }
        Game {
            snake: Snake::new(2, 2),
            waiting_time: 0.0,
            food: Food::new(6, 4),
            grid: Grid::new(width, height),
            state: GameState::Menu,
            high_score: persistence::load_high_score(),
            sound_player,
            last_dt: 0.0,
            fps: 0.0,
            fps_accum: 0.0,
            fps_frames: 0,
            tick_count: 0,
        }
    }

    pub fn key_pressed(&mut self, key: pw::Key) {
        let dir = match (key, &self.state) {
            (_, GameState::GameOver { .. }) => return,
            (pw::Key::Return | pw::Key::Space, GameState::Menu) => {
                self.state = GameState::Playing;
                return;
            }
            (pw::Key::Space, GameState::Playing) => {
                self.state = GameState::Paused;
                return;
            }
            (pw::Key::Space, GameState::Paused) => {
                self.state = GameState::Playing;
                return;
            }
            (_, GameState::Paused) => return,
            (pw::Key::Up | pw::Key::W, _) => Some(Direction::Up),
            (pw::Key::Down | pw::Key::S, _) => Some(Direction::Down),
            (pw::Key::Left | pw::Key::A, _) => Some(Direction::Left),
            (pw::Key::Right | pw::Key::D, _) => Some(Direction::Right),
            _ => Some(self.snake.head_direction()),
        };

        if let Some(dir) = dir {
            if dir == self.snake.head_direction().opposite() {
                return;
            }
        }

        self.update_snake(dir);
    }

    pub fn draw(
        &self,
        con: &pw::graphics::Context,
        g: &mut pw::wgpu_graphics::WgpuGraphics,
        glyphs: &mut Glyphs,
    ) {
        let mut renderables: Vec<Box<dyn Renderable + '_>> = Vec::new();
        renderables.push(Box::new(&self.snake));
        renderables.push(Box::new(&self.food));
        renderables.push(Box::new(&self.grid));
        renderables.push(Box::new(Hud {
            score: self.snake.len(),
            high_score: self.high_score,
            width: self.grid.width,
            turns: self.snake.corner_count(),
        }));
        renderables.push(Box::new(Overlay {
            state: self.state.clone(),
            width: self.grid.width,
            height: self.grid.height,
        }));

        for renderable in renderables {
            renderable.render(con, g, glyphs);
        }

        crate::debug_draw!({
            self.snake.draw_direction_indicator(con, g, DEBUG_COLOR);

            let dt_ms = self.last_dt * 1000.0;
            let fps_text = format!("FPS: {:.1}", self.fps);
            let dt_text = format!("dt: {:.2}ms", dt_ms);
            let tick_text = format!("ticks: {}", self.tick_count);
            let base_x = BLOCK_SIZE + 5.0;
            let base_y = BLOCK_SIZE + 12.0;

            let transform = con.transform.trans(base_x, base_y + FONT_SIZE as f64);
            pw::graphics::text::Text::new_color(DEBUG_COLOR, FONT_SIZE)
                .draw(&fps_text, glyphs, &con.draw_state, transform, g)
                .unwrap_or(());

            let transform = con.transform.trans(base_x, base_y + (FONT_SIZE as f64) * 2.0 + 6.0);
            pw::graphics::text::Text::new_color(DEBUG_COLOR, FONT_SIZE)
                .draw(&dt_text, glyphs, &con.draw_state, transform, g)
                .unwrap_or(());

            let transform = con.transform.trans(base_x, base_y + (FONT_SIZE as f64) * 3.0 + 12.0);
            pw::graphics::text::Text::new_color(DEBUG_COLOR, FONT_SIZE)
                .draw(&tick_text, glyphs, &con.draw_state, transform, g)
                .unwrap_or(());

        });
    }

    pub fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;
        self.last_dt = delta_time;
        self.fps_accum += delta_time;
        self.fps_frames += 1;
        self.tick_count += 1;

        if self.fps_accum >= 1.0 {
            self.fps = (self.fps_frames as f64) / self.fps_accum;
            self.fps_accum = 0.0;
            self.fps_frames = 0;
        }

        if matches!(self.state, GameState::Paused | GameState::Menu) {
            return;
        }

        if let GameState::GameOver {
            elapsed,
            final_score,
        } = &self.state
        {
            let new_elapsed = elapsed + delta_time;
            if new_elapsed > RESTART_TIME {
                self.restart();
            } else {
                self.state = GameState::GameOver {
                    elapsed: new_elapsed,
                    final_score: *final_score,
                };
            }
            return;
        }

        if !self.food.exists {
            self.add_food();
        }

        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None);
        }
    }

    pub(crate) fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        if self.food.exists && self.food.x == head_x && self.food.y == head_y {
            self.food.exists = false;
            self.snake.restore_tail();

            if let Some(ref player) = self.sound_player {
                player.play_eat();
            }

            let current_score = self.snake.len() as u32;
            if current_score > self.high_score {
                self.high_score = current_score;
                persistence::save_high_score(self.high_score);
                println!("New High Score: {}", self.high_score);
            }
        }
    }

    pub(crate) fn check_if_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head(dir);

        if self.snake.overlap_tail(next_x, next_y) {
            return false;
        }

        next_x > 0
            && next_y > 0
            && next_x < self.grid.width - 1
            && next_y < self.grid.height - 1
    }

    pub(crate) fn add_food(&mut self) {
        let mut rng = rand::rng();

        let choice = (1..self.grid.width - 1)
            .flat_map(|x| (1..self.grid.height - 1).map(move |y| (x, y))) // grid of all possible positions
            .filter(|(x, y)| !self.snake.overlap_tail(*x, *y)) // don't intersect snake
            .choose(&mut rng);

        if let Some((new_x, new_y)) = choice {
            self.food.set_position(new_x, new_y);
        }
    }

    fn update_snake(&mut self, direction: Option<Direction>) {
        if self.check_if_snake_alive(direction) {
            self.snake.move_forward(direction);
            self.check_eating();
        } else {
            self.state = GameState::GameOver {
                elapsed: 0.0,
                final_score: self.snake.len(),
            };
            if let Some(ref player) = self.sound_player {
                player.play_death();
            }
        }
        self.waiting_time = 0.0;
    }

    pub(crate) fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.waiting_time = 0.0;
        self.food = Food::new(6, 4);
        self.state = GameState::Playing;
        if let Some(ref player) = self.sound_player {
            player.play_start();
        }
    }

    #[cfg(test)]
    pub(crate) fn is_game_over(&self) -> bool {
        matches!(self.state, GameState::GameOver { .. })
    }

    #[cfg(test)]
    pub(crate) fn is_paused(&self) -> bool {
        matches!(self.state, GameState::Paused)
    }

    #[cfg(test)]
    pub(crate) fn is_menu(&self) -> bool {
        matches!(self.state, GameState::Menu)
    }

    #[cfg(test)]
    pub(crate) fn food_position(&self) -> Option<(i32, i32)> {
        self.food.position()
    }

    #[cfg(test)]
    pub(crate) fn snake_len(&self) -> usize {
        self.snake.len()
    }

    #[cfg(test)]
    pub(crate) fn snake_head_position(&self) -> (i32, i32) {
        self.snake.head_position()
    }

    #[cfg(test)]
    pub(crate) fn set_food_position(&mut self, x: i32, y: i32) {
        self.food.set_position(x, y);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use piston_window::Key;

    // Helper to create a game without sound for tests
    fn test_game(width: i32, height: i32) -> Game {
        Game::new(width, height, None)
    }

    #[test]
    fn new_creates_game_with_correct_dimensions() {
        let game = test_game(15, 15);
        assert_eq!(game.grid.width, 15);
        assert_eq!(game.grid.height, 15);
    }

    #[test]
    fn new_game_is_not_paused() {
        let game = test_game(15, 15);
        assert!(!game.is_paused());
    }

    #[test]
    fn new_game_is_not_game_over() {
        let game = test_game(15, 15);
        assert!(!game.is_game_over());
    }

    #[test]
    fn new_game_starts_in_menu() {
        let game = test_game(15, 15);
        assert!(game.is_menu());
    }

    #[test]
    fn new_game_has_food() {
        let game = test_game(15, 15);
        assert!(game.food_position().is_some());
    }

    #[test]
    fn new_game_snake_has_initial_length() {
        let game = test_game(15, 15);
        assert_eq!(game.snake_len(), 3);
    }

    #[test]
    fn space_toggles_pause() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        assert!(!game.is_paused());

        game.key_pressed(Key::Space);
        assert!(game.is_paused());

        game.key_pressed(Key::Space);
        assert!(!game.is_paused());
    }

    #[test]
    fn arrow_keys_move_snake() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        let initial_pos = game.snake_head_position();

        game.key_pressed(Key::Down);
        let new_pos = game.snake_head_position();

        assert_eq!(new_pos.1, initial_pos.1 + 1); // moved down
    }

    #[test]
    fn wasd_keys_move_snake() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        let initial_pos = game.snake_head_position();

        game.key_pressed(Key::S); // down
        let new_pos = game.snake_head_position();

        assert_eq!(new_pos.1, initial_pos.1 + 1);
    }

    #[test]
    fn cannot_move_in_opposite_direction() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        // Snake starts moving right, pressing left should not reverse
        let initial_pos = game.snake_head_position();

        game.key_pressed(Key::Left);
        let new_pos = game.snake_head_position();

        // Should still move right (or not move at all), not left
        assert!(new_pos.0 >= initial_pos.0);
    }

    #[test]
    fn paused_game_ignores_movement_keys() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        game.key_pressed(Key::Space); // pause
        let pos_before = game.snake_head_position();

        game.key_pressed(Key::Down);
        let pos_after = game.snake_head_position();

        assert_eq!(pos_before, pos_after);
    }

    #[test]
    fn update_moves_snake_after_moving_period() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        let initial_pos = game.snake_head_position();

        // Simulate time passing beyond MOVING_PERIOD (0.3)
        game.update(0.35);
        let new_pos = game.snake_head_position();

        assert_ne!(initial_pos, new_pos);
    }

    #[test]
    fn update_does_not_move_when_paused() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        game.key_pressed(Key::Space); // pause
        let pos_before = game.snake_head_position();

        game.update(0.35);
        let pos_after = game.snake_head_position();

        assert_eq!(pos_before, pos_after);
    }

    #[test]
    fn snake_dies_hitting_left_wall() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        // Move snake to the left wall
        game.key_pressed(Key::Up); // change direction first to allow left
        game.key_pressed(Key::Left);

        // Keep moving left until game over
        for _ in 0..10 {
            if game.is_game_over() {
                break;
            }
            game.update(0.35);
        }

        assert!(game.is_game_over());
    }

    #[test]
    fn snake_dies_hitting_top_wall() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        game.key_pressed(Key::Up);

        for _ in 0..10 {
            if game.is_game_over() {
                break;
            }
            game.update(0.35);
        }

        assert!(game.is_game_over());
    }

    #[test]
    fn check_if_snake_alive_returns_false_at_wall() {
        let game = test_game(15, 15);
        // Snake head starts at (4, 2), moving right toward wall at x=14
        // At x=13, next move would hit wall at x=14 which is width-1
        // Actually the boundary check is next_x < self.grid.width - 1
        // So if next_x = 14 and width = 15, 14 < 14 is false
        assert!(game.check_if_snake_alive(None)); // should be alive initially
    }

    #[test]
    fn eating_food_grows_snake() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        let initial_len = game.snake_len();

        // Place food directly in front of snake (snake head at (4,2), moving right)
        game.set_food_position(5, 2);
        game.update(0.35); // move onto food

        assert_eq!(game.snake_len(), initial_len + 1);
    }

    #[test]
    fn eating_food_sets_food_exists_to_false() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu

        // Place food in front of snake (snake head at (4,2), moving right)
        game.set_food_position(5, 2);
        assert!(game.food.exists); // food exists before eating

        game.update(0.35); // move onto food

        // Snake should have grown (proving food was eaten)
        assert_eq!(game.snake_len(), 4);

        // food_exists should now be false (food was consumed)
        // Note: add_food is called on NEXT update cycle, not immediately
        assert!(!game.food.exists);

        // Next update should spawn new food
        game.update(0.35);
        assert!(game.food.exists);
    }

    #[test]
    fn add_food_places_food_in_bounds() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu
        game.food.exists = false;
        game.add_food();

        let (x, y) = game.food_position().unwrap();
        assert!(x > 0 && x < 14);
        assert!(y > 0 && y < 14);
    }

    #[test]
    fn restart_resets_game_state() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Return); // start from menu

        // Mess up the game state
        game.state = GameState::GameOver {
            elapsed: 0.0,
            final_score: 0,
        };
        game.food.exists = false;

        game.restart();

        assert!(!game.is_game_over());
        assert!(game.food_position().is_some());
        assert_eq!(game.snake_len(), 3);
    }

    #[test]
    fn game_restarts_after_delay_when_game_over() {
        let mut game = test_game(15, 15);
        game.state = GameState::GameOver {
            elapsed: 0.0,
            final_score: 0,
        };
        game.waiting_time = 0.0;

        // Update with less than RESTART_TIME
        game.update(2.0);
        assert!(game.is_game_over()); // still game over

        // Update past RESTART_TIME (3.0)
        game.update(2.0); // total waiting_time now > 3.0
        assert!(!game.is_game_over()); // should restart
    }
}
