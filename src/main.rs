#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::digital::InputPin;
use heapless::Deque;
use microbit::{board::Board, display::blocking::Display, hal::Timer};

use panic_rtt_target as _;

const BOARD_WIDTH: usize = 5;
const BOARD_HEIGHT: usize = 5;
const BOARD_SIZE: usize = BOARD_WIDTH * BOARD_HEIGHT;
const FRAME_TIME: u32 = 10; // milliseconds
const TURN_TIME: u32 = 400; // milliseconds
const FRAMES_PER_TURN: u32 = TURN_TIME / FRAME_TIME;

/// Direction to turn, relative to current direction of travel, based on user input
// #[derive(Debug, Clone)]
// enum TurnDirection {
//     Left,
//     Right,
// }

/// Direction of movement for the snake
#[derive(Debug, Clone)]
enum MoveDirection {
    BoardUp,
    BoardDown,
    BoardLeft,
    BoardRight,
}

#[derive(Debug, Clone)]
struct SnakeSegment(usize, usize);

#[derive(Debug, Clone)]
struct Food(usize, usize);

#[derive(Debug, Clone)]
struct GameState {
    snake: Deque<SnakeSegment, BOARD_SIZE>,
    food: Option<Food>,
    move_direction: MoveDirection,
    dead: bool,
}
impl GameState {
    fn new() -> Self {
        let mut snake_deque = Deque::<_, BOARD_SIZE>::new();
        let _ = snake_deque.push_front(SnakeSegment(0, 1));
        let _ = snake_deque.push_front(SnakeSegment(0, 2));
        let _ = snake_deque.push_back(SnakeSegment(0, 0));
        Self {
            snake: snake_deque,
            food: Some(Food(4, 4)),
            move_direction: MoveDirection::BoardRight,
            dead: false,
        }
    }

    fn turn_right(&mut self) {
        self.move_direction = match self.move_direction {
            MoveDirection::BoardDown => MoveDirection::BoardLeft,
            MoveDirection::BoardLeft => MoveDirection::BoardUp,
            MoveDirection::BoardUp => MoveDirection::BoardRight,
            MoveDirection::BoardRight => MoveDirection::BoardDown,
        };
    }
    fn turn_left(&mut self) {
        self.move_direction = match self.move_direction {
            MoveDirection::BoardDown => MoveDirection::BoardRight,
            MoveDirection::BoardLeft => MoveDirection::BoardDown,
            MoveDirection::BoardUp => MoveDirection::BoardLeft,
            MoveDirection::BoardRight => MoveDirection::BoardUp,
        };
    }

    fn render_image(&self) -> [[u8; 5]; 5] {
        let mut image_matrix = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];

        for snake_segment in self.snake.iter() {
            image_matrix[snake_segment.0][snake_segment.1] = 1;
        }

        if let Some(food) = &self.food {
            image_matrix[food.0][food.1] = 1;
        }

        image_matrix
    }

    fn update(&mut self) {
        let old_snake_head = self.snake.pop_front().unwrap();
        let new_snake_head = match self.move_direction {
            MoveDirection::BoardUp => SnakeSegment(old_snake_head.0 - 1, old_snake_head.1),
            MoveDirection::BoardLeft => SnakeSegment(old_snake_head.0, old_snake_head.1 - 1),
            MoveDirection::BoardRight => SnakeSegment(old_snake_head.0, old_snake_head.1 + 1),
            MoveDirection::BoardDown => SnakeSegment(old_snake_head.0 + 1, old_snake_head.1),
        };
        // wall collision check
        if new_snake_head.0 > (BOARD_HEIGHT - 1) || new_snake_head.1 > (BOARD_WIDTH - 1) {
            // die
            self.dead = true;
        } else {
            if let Some(food) = &self.food {
                if food.0 == new_snake_head.0 && food.1 == new_snake_head.1 {
                    // ate the food, remove from screen and don't shrink the tail
                    self.food = None;
                } else {
                    // no food eaten, remove the tail before we add the new head
                    self.snake.pop_back().unwrap();
                }
            } else {
                self.snake.pop_back().unwrap();
            }
            let _ = self.snake.push_front(old_snake_head);
            let _ = self.snake.push_front(new_snake_head);
        }
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();

    let mut button_a = board.buttons.button_a;
    let mut button_b = board.buttons.button_b;

    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut game_board = GameState::new();
    let mut frames_in_turn_count = 0;

    let mut left_button_down = false;
    let mut right_button_down = false;

    let mut left_turn_count = 0;
    let mut right_turn_count = 0;

    loop {
        display.show(&mut timer, game_board.render_image(), FRAME_TIME);

        // detect a button press on button-up, not button-down, to help avoid repeats
        if !left_button_down && button_a.is_low().unwrap() {
            left_button_down = true;
        }
        if left_button_down && button_a.is_high().unwrap() {
            left_turn_count += 1;
            left_button_down = false;
        }

        if !right_button_down && button_b.is_low().unwrap() {
            right_button_down = true;
        }
        if right_button_down && button_b.is_high().unwrap() {
            right_turn_count += 1;
            right_button_down = false;
        }

        if frames_in_turn_count == FRAMES_PER_TURN {
            if right_turn_count > 0 {
                game_board.turn_right();
            } else if left_turn_count > 0 {
                game_board.turn_left();
            }

            game_board.update();
            frames_in_turn_count = 0;
            left_button_down = false;
            right_button_down = false;
            right_turn_count = 0;
            left_turn_count = 0;
        } else {
            frames_in_turn_count += 1;
        }
    }
}
