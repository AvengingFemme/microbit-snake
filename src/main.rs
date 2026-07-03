#![deny(unsafe_code)]
#![no_main]
#![no_std]

use crate::GameEntity::{Empty, Food, SnakeHead, SnakeTail, SnakeWake};
use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use microbit::{board::Board, display::blocking::Display, hal::Timer};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
/// Direction to turn, relative to current direction of travel, based on user input
#[derive(Debug, Clone)]
enum TurnDirection {
    Left,
    Right,
}

/// Direction of movement for the snake
#[derive(Debug, Clone)]
enum MoveDirection {
    BoardUp,
    BoardDown,
    BoardLeft,
    BoardRight,
}

/// Game entities that can exist on the game board
#[derive(Debug, Clone)]
enum GameEntity {
    SnakeHead(MoveDirection),
    SnakeTail,
    SnakeWake,
    Food,
    Empty,
}

#[derive(Debug, Clone)]
struct GameEntityCoord(usize, usize);

#[derive(Debug, Clone)]
struct GameBoard {
    board_matrix: [[GameEntity; 5]; 5],
}
impl GameBoard {
    fn new() -> Self {
        GameBoard {
            board_matrix: [
                [
                    SnakeTail,
                    SnakeHead(MoveDirection::BoardRight),
                    Empty,
                    Empty,
                    Empty,
                ],
                [Empty, Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty, Empty],
            ],
        }
    }

    fn render_image(&self) -> [[u8; 5]; 5] {
        let mut image_matrix = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];

        for (row_num, row_val) in self.board_matrix.iter().enumerate() {
            for (col_num, col_val) in row_val.iter().enumerate() {
                image_matrix[row_num][col_num] = match col_val {
                    SnakeHead(_) => 1,
                    SnakeTail => 1,
                    SnakeWake => 0,
                    Food => 1,
                    Empty => 0,
                };
            }
        }

        image_matrix
    }

    fn update(&self) {
        // find the head
        let mut snake_head_coord = GameEntityCoord(0, 0);
        let mut snake_head_dir = MoveDirection::BoardRight;
        for (row_num, row_val) in self.board_matrix.iter().enumerate() {
            for (col_num, col_val) in row_val.iter().enumerate() {
                match col_val {
                    SnakeHead(dir) => {
                        snake_head_coord = GameEntityCoord(row_num, col_num);
                        snake_head_dir = dir.clone();
                    }
                    SnakeTail => {}
                    SnakeWake => {}
                    Food => {}
                    Empty => {}
                }
            }
        }

        // TODO: move the snake head in its direction of travel
        // TODO: move the snake tail entities along the wake
        // I think I'm going to have have a singly linked list representing the snake though
        // so the snake tail entity movements can be properly coordinated, since they
        // need to start from the head and move back along the tail
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let game_board = GameBoard::new();

    loop {
        // Show the game board for 1000ms
        display.show(&mut timer, game_board.render_image(), 1000);
    }
}
