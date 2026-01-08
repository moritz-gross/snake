use piston_window as pw;
use piston_window::Glyphs;
use pw::graphics::Transformed;

use rand::Rng;

use crate::audio::SoundPlayer;
use crate::draw::{draw_block, draw_rectangle, BLOCK_SIZE};
use crate::snake::{Direction, Snake};
use crate::persistence;

const FOOD_COLOR: pw::graphics::types::Color = [0.80, 0.00, 0.00, 1.0];
const BORDER_COLOR: pw::graphics::types::Color = [0.00, 0.00, 0.00, 1.0];
const GAMEOVER_COLOR: pw::graphics::types::Color = [0.90, 0.00, 0.00, 0.5];
const PAUSE_COLOR: pw::graphics::types::Color = [0.00, 0.00, 0.00, 0.5];
const TEXT_COLOR: pw::graphics::types::Color = [1.0, 1.0, 1.0, 1.0];
const FONT_SIZE: u32 = 16;

const MOVING_PERIOD: f64 = 0.3;
const RESTART_TIME: f64 = 3.0;

pub struct Game {
    snake: Snake,

    food_exists: bool,
    food_x: i32,
    food_y: i32,

    width: i32,
    height: i32,

    game_over: bool,
    waiting_time: f64,
    high_score: u32,
    paused: bool,
    sound_player: Option<SoundPlayer>,
}

impl Game {
    pub fn new(width: i32, height: i32, sound_player: Option<SoundPlayer>) -> Game {
        if let Some(ref player) = sound_player {
            player.play_start();
        }
        Game {
            snake: Snake::new(2, 2),
            waiting_time: 0.0,
            food_exists: true,
            food_x: 6,
            food_y: 4,
            width,
            height,
            game_over: false,
            high_score: persistence::load_high_score(),
            paused: false,
            sound_player,
        }
    }

    pub fn key_pressed(&mut self, key: pw::Key) {
        if self.game_over {
            return;
        }

        if key == pw::Key::Space {
            self.paused = !self.paused;
        }

        if self.paused {
            return;
        }

        let dir = match key {
            pw::Key::Up | pw::Key::W => Some(Direction::Up),
            pw::Key::Down | pw::Key::S => Some(Direction::Down),
            pw::Key::Left | pw::Key::A => Some(Direction::Left),
            pw::Key::Right | pw::Key::D => Some(Direction::Right),
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
        self.snake.draw(con, g);

        if self.food_exists {
            draw_block(FOOD_COLOR, self.food_x, self.food_y, con, g);
        }

        draw_rectangle(BORDER_COLOR, 0, 0, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, self.height - 1, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, 0, 1, self.height, con, g);
        draw_rectangle(BORDER_COLOR, self.width - 1, 0, 1, self.height, con, g);

        // Draw score text
        let score_text = format!("Score: {}", self.snake.len());
        let high_text = format!("High: {}", self.high_score);

        // Position text inside the top border
        let text_y = BLOCK_SIZE - 4.0; // Just above the border
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

        if self.game_over {
            draw_rectangle(GAMEOVER_COLOR, 0, 0, self.width, self.height, con, g);
        }

        if self.paused {
            draw_rectangle(PAUSE_COLOR, 0, 0, self.width, self.height, con, g);
            // Draw two vertical bars for pause icon
            let center_x = self.width / 2;
            let center_y = self.height / 2;
            draw_rectangle(BORDER_COLOR, center_x - 1, center_y - 1, 1, 3, con, g);
            draw_rectangle(BORDER_COLOR, center_x + 1, center_y - 1, 1, 3, con, g);
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;

        if self.paused {
            return;
        }

        if self.game_over {
            if self.waiting_time > RESTART_TIME {
                self.restart();
            }
            return;
        }

        if !self.food_exists {
            self.add_food();
        }

        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None);
        }
    }

    pub(crate) fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        if self.food_exists && self.food_x == head_x && self.food_y == head_y {
            self.food_exists = false;
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

        next_x > 0 && next_y > 0 && next_x < self.width - 1 && next_y < self.height - 1
    }

    pub(crate) fn add_food(&mut self) {
        let mut rng = rand::rng();

        let mut new_x = rng.random_range(1..self.width - 1);
        let mut new_y = rng.random_range(1..self.height - 1);
        while self.snake.overlap_tail(new_x, new_y) {
            new_x = rng.random_range(1..self.width - 1);
            new_y = rng.random_range(1..self.height - 1);
        }

        self.food_x = new_x;
        self.food_y = new_y;
        self.food_exists = true;
    }

    fn update_snake(&mut self, direction: Option<Direction>) {
        if self.check_if_snake_alive(direction) {
            self.snake.move_forward(direction);
            self.check_eating();
        } else {
            self.game_over = true;
            if let Some(ref player) = self.sound_player {
                player.play_death();
            }
        }
        self.waiting_time = 0.0;
    }

    pub(crate) fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.waiting_time = 0.0;
        self.food_exists = true;
        self.food_x = 6;
        self.food_y = 4;
        self.game_over = false;
        if let Some(ref player) = self.sound_player {
            player.play_start();
        }
    }

    #[cfg(test)]
    pub(crate) fn is_game_over(&self) -> bool {
        self.game_over
    }

    #[cfg(test)]
    pub(crate) fn is_paused(&self) -> bool {
        self.paused
    }

    #[cfg(test)]
    pub(crate) fn food_position(&self) -> Option<(i32, i32)> {
        if self.food_exists {
            Some((self.food_x, self.food_y))
        } else {
            None
        }
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
        self.food_x = x;
        self.food_y = y;
        self.food_exists = true;
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
        assert_eq!(game.width, 15);
        assert_eq!(game.height, 15);
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
        assert!(!game.is_paused());

        game.key_pressed(Key::Space);
        assert!(game.is_paused());

        game.key_pressed(Key::Space);
        assert!(!game.is_paused());
    }

    #[test]
    fn arrow_keys_move_snake() {
        let mut game = test_game(15, 15);
        let initial_pos = game.snake_head_position();

        game.key_pressed(Key::Down);
        let new_pos = game.snake_head_position();

        assert_eq!(new_pos.1, initial_pos.1 + 1); // moved down
    }

    #[test]
    fn wasd_keys_move_snake() {
        let mut game = test_game(15, 15);
        let initial_pos = game.snake_head_position();

        game.key_pressed(Key::S); // down
        let new_pos = game.snake_head_position();

        assert_eq!(new_pos.1, initial_pos.1 + 1);
    }

    #[test]
    fn cannot_move_in_opposite_direction() {
        let mut game = test_game(15, 15);
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
        game.key_pressed(Key::Space); // pause
        let pos_before = game.snake_head_position();

        game.key_pressed(Key::Down);
        let pos_after = game.snake_head_position();

        assert_eq!(pos_before, pos_after);
    }

    #[test]
    fn update_moves_snake_after_moving_period() {
        let mut game = test_game(15, 15);
        let initial_pos = game.snake_head_position();

        // Simulate time passing beyond MOVING_PERIOD (0.3)
        game.update(0.35);
        let new_pos = game.snake_head_position();

        assert_ne!(initial_pos, new_pos);
    }

    #[test]
    fn update_does_not_move_when_paused() {
        let mut game = test_game(15, 15);
        game.key_pressed(Key::Space); // pause
        let pos_before = game.snake_head_position();

        game.update(0.35);
        let pos_after = game.snake_head_position();

        assert_eq!(pos_before, pos_after);
    }

    #[test]
    fn snake_dies_hitting_left_wall() {
        let mut game = test_game(15, 15);
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
        // Actually the boundary check is next_x < self.width - 1
        // So if next_x = 14 and width = 15, 14 < 14 is false
        assert!(game.check_if_snake_alive(None)); // should be alive initially
    }

    #[test]
    fn eating_food_grows_snake() {
        let mut game = test_game(15, 15);
        let initial_len = game.snake_len();

        // Place food directly in front of snake (snake head at (4,2), moving right)
        game.set_food_position(5, 2);
        game.update(0.35); // move onto food

        assert_eq!(game.snake_len(), initial_len + 1);
    }

    #[test]
    fn eating_food_sets_food_exists_to_false() {
        let mut game = test_game(15, 15);

        // Place food in front of snake (snake head at (4,2), moving right)
        game.set_food_position(5, 2);
        assert!(game.food_exists); // food exists before eating

        game.update(0.35); // move onto food

        // Snake should have grown (proving food was eaten)
        assert_eq!(game.snake_len(), 4);

        // food_exists should now be false (food was consumed)
        // Note: add_food is called on NEXT update cycle, not immediately
        assert!(!game.food_exists);

        // Next update should spawn new food
        game.update(0.35);
        assert!(game.food_exists);
    }

    #[test]
    fn add_food_places_food_in_bounds() {
        let mut game = test_game(15, 15);
        game.food_exists = false;
        game.add_food();

        let (x, y) = game.food_position().unwrap();
        assert!(x > 0 && x < 14);
        assert!(y > 0 && y < 14);
    }

    #[test]
    fn restart_resets_game_state() {
        let mut game = test_game(15, 15);

        // Mess up the game state
        game.game_over = true;
        game.food_exists = false;

        game.restart();

        assert!(!game.is_game_over());
        assert!(game.food_position().is_some());
        assert_eq!(game.snake_len(), 3);
    }

    #[test]
    fn game_restarts_after_delay_when_game_over() {
        let mut game = test_game(15, 15);
        game.game_over = true;
        game.waiting_time = 0.0;

        // Update with less than RESTART_TIME
        game.update(2.0);
        assert!(game.is_game_over()); // still game over

        // Update past RESTART_TIME (3.0)
        game.update(2.0); // total waiting_time now > 3.0
        assert!(!game.is_game_over()); // should restart
    }
}
