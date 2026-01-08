use std::collections::VecDeque;

use piston_window as pw;

use crate::draw::draw_block;

const SNAKE_COLOR: pw::graphics::types::Color = [0.00, 0.80, 0.00, 1.0];

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn as_array(&self) -> [i32; 2] {
        match *self {
            Direction::Up => [0, -1],
            Direction::Down => [0, 1],
            Direction::Left => [-1, 0],
            Direction::Right => [1, 0],
        }
    }
}

#[derive(Debug, Clone)]
struct Block {
    x: i32,
    y: i32,
}

pub struct Snake {
    direction: Direction,
    body: VecDeque<Block>,
    tail: Option<Block>,
}

impl Snake {
    pub fn new(x: i32, y: i32) -> Snake {
        let mut body = VecDeque::new();
        body.push_back(Block { x: x + 2, y });
        body.push_back(Block { x: x + 1, y });
        body.push_back(Block { x, y });

        Snake {
            direction: Direction::Right,
            body,
            tail: None,
        }
    }

    pub fn draw(&self, con: &pw::graphics::Context, g: &mut pw::wgpu_graphics::WgpuGraphics) {
        for block in &self.body {
            draw_block(SNAKE_COLOR, block.x, block.y, con, g);
        }
    }

    pub fn head_position(&self) -> (i32, i32) {
        let head_block = self.body.front().unwrap();
        (head_block.x, head_block.y)
    }

    pub fn move_forward(&mut self, dir: Option<Direction>) {
        if let Some(d) = dir {
            self.direction = d;
        }

        let (last_x, last_y): (i32, i32) = self.head_position();

        let direction_arr = self.direction.as_array();
        let new_block_v2 = Block {
            x: last_x + direction_arr[0],
            y: last_y + direction_arr[1],
        };

        self.body.push_front(new_block_v2);
        let removed_block = self.body.pop_back().unwrap();
        self.tail = Some(removed_block);
    }

    pub fn head_direction(&self) -> Direction {
        self.direction
    }

    pub fn next_head(&self, dir: Option<Direction>) -> (i32, i32) {
        let moving_dir = match dir {
            Some(d) => d,
            None => self.direction, // keep direction
        };

        let moving_direction_arr: [i32; 2] = moving_dir.as_array();
        let (hx, hy) = self.head_position();
        (hx + moving_direction_arr[0], hy + moving_direction_arr[1])
    }

    pub fn restore_tail(&mut self) {
        let blk = self.tail.clone().unwrap();
        self.body.push_back(blk);
    }

    pub fn overlap_tail(&self, x: i32, y: i32) -> bool {
        self.body.iter()
            .skip(1)  // skip head
            .any(|block| block.x == x && block.y == y)
    }


    pub fn len(&self) -> usize {
        self.body.len()
    }
}

#[cfg(test)]
mod test {
    mod direction {
        use crate::snake::Direction;

        #[test]
        fn as_array_up() {
            assert_eq!(Direction::Up.as_array(), [0, -1]);
        }

        #[test]
        fn as_array_down() {
            assert_eq!(Direction::Down.as_array(), [0, 1]);
        }

        #[test]
        fn as_array_left() {
            assert_eq!(Direction::Left.as_array(), [-1, 0]);
        }

        #[test]
        fn as_array_right() {
            assert_eq!(Direction::Right.as_array(), [1, 0]);
        }

        #[test]
        fn opposite_up() {
            assert_eq!(Direction::Up.opposite(), Direction::Down);
        }

        #[test]
        fn opposite_down() {
            assert_eq!(Direction::Down.opposite(), Direction::Up);
        }

        #[test]
        fn opposite_left() {
            assert_eq!(Direction::Left.opposite(), Direction::Right);
        }

        #[test]
        fn opposite_right() {
            assert_eq!(Direction::Right.opposite(), Direction::Left);
        }

        #[test]
        fn opposite_is_symmetric() {
            for dir in [Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                assert_eq!(dir.opposite().opposite(), dir);
            }
        }
    }

    mod snake {
        use crate::snake::{Direction, Snake};

        #[test]
        fn new_creates_snake_with_length_3() {
            let snake = Snake::new(2, 2);
            assert_eq!(snake.len(), 3);
        }

        #[test]
        fn new_sets_initial_direction_right() {
            let snake = Snake::new(2, 2);
            assert_eq!(snake.head_direction(), Direction::Right);
        }

        #[test]
        fn new_sets_head_at_correct_position() {
            let snake = Snake::new(2, 2);
            assert_eq!(snake.head_position(), (4, 2)); // x + 2, y
        }

        #[test]
        fn move_forward_updates_head_position() {
            let mut snake = Snake::new(2, 2);
            let initial_head = snake.head_position();
            snake.move_forward(None);
            let new_head = snake.head_position();
            assert_eq!(new_head, (initial_head.0 + 1, initial_head.1)); // moved right
        }

        #[test]
        fn move_forward_with_direction_change() {
            let mut snake = Snake::new(2, 2);
            snake.move_forward(Some(Direction::Down));
            assert_eq!(snake.head_direction(), Direction::Down);
            assert_eq!(snake.head_position(), (4, 3)); // moved down from (4, 2)
        }

        #[test]
        fn move_forward_preserves_length() {
            let mut snake = Snake::new(2, 2);
            let initial_len = snake.len();
            snake.move_forward(None);
            assert_eq!(snake.len(), initial_len);
        }

        #[test]
        fn restore_tail_increases_length() {
            let mut snake = Snake::new(2, 2);
            snake.move_forward(None); // need to move first to have a tail
            let len_before = snake.len();
            snake.restore_tail();
            assert_eq!(snake.len(), len_before + 1);
        }

        #[test]
        fn next_head_predicts_correctly_without_direction() {
            let snake = Snake::new(2, 2);
            let head = snake.head_position();
            let next = snake.next_head(None);
            assert_eq!(next, (head.0 + 1, head.1)); // default direction is right
        }

        #[test]
        fn next_head_predicts_correctly_with_direction() {
            let snake = Snake::new(2, 2);
            let head = snake.head_position();
            let next = snake.next_head(Some(Direction::Up));
            assert_eq!(next, (head.0, head.1 - 1));
        }

        #[test]
        fn overlap_tail_returns_false_for_empty_space() {
            let snake = Snake::new(2, 2);
            assert!(!snake.overlap_tail(100, 100));
        }

        #[test]
        fn overlap_tail_returns_true_for_body_segment() {
            let snake = Snake::new(2, 2);
            // Snake body is at (4,2), (3,2), (2,2) - check middle segment
            assert!(snake.overlap_tail(3, 2));
        }

        #[test]
        fn overlap_tail_skips_head() {
            let snake = Snake::new(2, 2);
            // Head is at (4, 2) - should not count as overlap
            assert!(!snake.overlap_tail(4, 2));
        }

        #[test]
        fn overlap_tail_detects_tail_segment() {
            let snake = Snake::new(2, 2);
            // Tail is at (2, 2)
            assert!(snake.overlap_tail(2, 2));
        }

        #[test]
        fn move_in_all_directions() {
            let mut snake = Snake::new(5, 5);

            snake.move_forward(Some(Direction::Up));
            assert_eq!(snake.head_position(), (7, 4));

            snake.move_forward(Some(Direction::Left));
            assert_eq!(snake.head_position(), (6, 4));

            snake.move_forward(Some(Direction::Down));
            assert_eq!(snake.head_position(), (6, 5));

            snake.move_forward(Some(Direction::Right));
            assert_eq!(snake.head_position(), (7, 5));
        }
    }
}
