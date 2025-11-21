use piston_window as pw;

use rand::Rng;

use crate::draw::{draw_block, draw_rectangle};
use crate::snake::{Direction, Snake};
use crate::persistence;

const FOOD_COLOR: pw::types::Color = [0.80, 0.00, 0.00, 1.0];
const BORDER_COLOR: pw::types::Color = [0.00, 0.00, 0.00, 1.0];
const GAMEOVER_COLOR: pw::types::Color = [0.90, 0.00, 0.00, 0.5];
const PAUSE_COLOR: pw::types::Color = [0.00, 0.00, 0.00, 0.5];

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
}

impl Game {
    pub fn new(width: i32, height: i32) -> Game {
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

    pub fn draw(&self, con: &pw::Context, g: &mut pw::G2d) {
        self.snake.draw(con, g);

        if self.food_exists {
            draw_block(FOOD_COLOR, self.food_x, self.food_y, con, g);
        }

        draw_rectangle(BORDER_COLOR, 0, 0, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, self.height - 1, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, 0, 1, self.height, con, g);
        draw_rectangle(BORDER_COLOR, self.width - 1, 0, 1, self.height, con, g);

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

    fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        if self.food_exists && self.food_x == head_x && self.food_y == head_y {
            self.food_exists = false;
            self.snake.restore_tail();

            let current_score = self.snake.len() as u32;
            if current_score > self.high_score {
                self.high_score = current_score;
                persistence::save_high_score(self.high_score);
                println!("New High Score: {}", self.high_score);
            }
        }

    }

    fn check_if_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head(dir);

        if self.snake.overlap_tail(next_x, next_y) {
            return false;
        }

        next_x > 0 && next_y > 0 && next_x < self.width - 1 && next_y < self.height - 1
    }

    fn add_food(&mut self) {
        let mut rng = rand::thread_rng();

        let mut new_x = rng.gen_range(1..self.width - 1);
        let mut new_y = rng.gen_range(1..self.height - 1);
        while self.snake.overlap_tail(new_x, new_y) {
            new_x = rng.gen_range(1..self.width - 1);
            new_y = rng.gen_range(1..self.height - 1);
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
        }
        self.waiting_time = 0.0;
    }

    fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.waiting_time = 0.0;
        self.food_exists = true;
        self.food_x = 6;
        self.food_y = 4;
        self.game_over = false;
    }
}
