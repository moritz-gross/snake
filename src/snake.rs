use piston_window as pw;

use crate::draw::draw_block;

const SNAKE_COLOR: pw::types::Color = [0.00, 0.80, 0.00, 1.0];

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
    body: std::collections::LinkedList<Block>,
    tail: Option<Block>,
}

impl Snake {
    pub fn new(x: i32, y: i32) -> Snake {
        let mut body = std::collections::LinkedList::new();
        body.push_back(Block { x: x + 2, y });
        body.push_back(Block { x: x + 1, y });
        body.push_back(Block { x, y });

        Snake {
            direction: Direction::Right,
            body,
            tail: None,
        }
    }

    pub fn draw(&self, con: &pw::Context, g: &mut pw::G2d) {
        for block in &self.body {
            draw_block(SNAKE_COLOR, block.x, block.y, con, g);
        }
    }

    pub fn head_position(&self) -> (i32, i32) {
        let head_block = self.body.front().unwrap();
        (head_block.x, head_block.y)
    }

    pub fn move_forward(&mut self, dir: Option<Direction>) {
        match dir {
            Some(d) => self.direction = d,
            None => (),
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
        return (
            self.head_position().0 + moving_direction_arr[0],
            self.head_position().1 + moving_direction_arr[1],
        );
    }

    pub fn restore_tail(&mut self) {
        let blk = self.tail.clone().unwrap();
        self.body.push_back(blk);
    }

    pub fn overlap_tail(&self, x: i32, y: i32) -> bool {
        let mut ch = 0;
        for block in &self.body {
            if x == block.x && y == block.y {
                return true;
            }

            ch += 1;
            if ch == self.body.len() - 1 {
                break;
            }
        }
        return false;
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }
}

#[cfg(test)]
mod test {
    mod snake {
        use crate::snake;

        #[test]
        fn array_direction_up() {
            assert_eq!(snake::Direction::Up.as_array(), [0, -1] )
        }

        #[test]
        fn array_direction_down() {
            assert_eq!(snake::Direction::Down.as_array(), [0, 1] )
        }

        #[test]
        fn array_direction_left() {
            assert_eq!(snake::Direction::Left.as_array(), [-1, 0] )
        }

        #[test]
        fn array_direction_right() {
            assert_eq!(snake::Direction::Right.as_array(), [1, 0] )
        }

        #[test]
        fn opposite_direction_up() {
            assert_eq!(snake::Direction::Up.opposite(), snake::Direction::Down )
        }

        #[test]
        fn opposite_direction_down() {
            assert_eq!(snake::Direction::Down.opposite(), snake::Direction::Up )
        }

        #[test]
        fn opposite_direction_left() {
            assert_eq!(snake::Direction::Left.opposite(), snake::Direction::Right )
        }

        #[test]
        fn opposite_direction_right() {
            assert_eq!(snake::Direction::Right.opposite(), snake::Direction::Left )
        }
    }
}