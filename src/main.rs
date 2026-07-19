#![no_main]
#![no_std]

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use embedded_hal::digital::InputPin;
use heapless::Deque;
use microbit::{
    board::Board,
    display::nonblocking::{Display, GreyscaleImage},
    hal::{
        Timer,
        clocks::Clocks,
        rtc::{Rtc, RtcInterrupt},
    },
    pac::{self, RTC0, TIMER1, interrupt},
};

use defmt_rtt as _;
use panic_probe as _;

const BOARD_WIDTH: usize = 5;
const BOARD_HEIGHT: usize = 5;
const BOARD_SIZE: usize = BOARD_WIDTH * BOARD_HEIGHT;
const SNAKE_MAX_SIZE: usize = BOARD_SIZE + 1;
const FRAME_TIME: u32 = 10; // milliseconds
const MOVE_TIME: u32 = 400; // milliseconds
const FRAMES_PER_MOVE: u32 = MOVE_TIME / FRAME_TIME;

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

#[derive(Debug, Clone, defmt::Format)]
struct Food(usize, usize);

#[derive(Debug, Clone)]
struct GameState {
    snake: Deque<SnakeSegment, SNAKE_MAX_SIZE>,
    food: Option<Food>,
    move_direction: MoveDirection,
    dead: bool,
}
impl GameState {
    fn new() -> Self {
        let food = Food(4, 4);
        defmt::trace!("Initializing new GameState object with food: {}", food);

        let mut snake_deque = Deque::<_, SNAKE_MAX_SIZE>::new();

        let result = snake_deque.push_front(SnakeSegment(0, 1));
        if result.is_err() {
            defmt::error!("Snake deque full, cannot push segment!");
        }
        let result = snake_deque.push_front(SnakeSegment(0, 2));
        if result.is_err() {
            defmt::error!("Snake deque full, cannot push segment!");
        }
        let result = snake_deque.push_back(SnakeSegment(0, 0));
        if result.is_err() {
            defmt::error!("Snake deque full, cannot push segment!");
        }

        Self {
            snake: snake_deque,
            food: Some(food),
            move_direction: MoveDirection::BoardRight,
            dead: false,
        }
    }

    fn turn_right(&mut self) {
        defmt::trace!("Turning right");
        self.move_direction = match self.move_direction {
            MoveDirection::BoardDown => MoveDirection::BoardLeft,
            MoveDirection::BoardLeft => MoveDirection::BoardUp,
            MoveDirection::BoardUp => MoveDirection::BoardRight,
            MoveDirection::BoardRight => MoveDirection::BoardDown,
        };
    }
    fn turn_left(&mut self) {
        defmt::trace!("Turning left");
        self.move_direction = match self.move_direction {
            MoveDirection::BoardDown => MoveDirection::BoardRight,
            MoveDirection::BoardLeft => MoveDirection::BoardDown,
            MoveDirection::BoardUp => MoveDirection::BoardLeft,
            MoveDirection::BoardRight => MoveDirection::BoardUp,
        };
    }

    fn render_image(&self) -> GreyscaleImage {
        defmt::trace!("Begin render_image call");
        let mut image_matrix = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];

        for snake_segment in self.snake.iter() {
            image_matrix[snake_segment.0][snake_segment.1] = 9;
        }

        if let Some(food) = &self.food {
            image_matrix[food.0][food.1] = 7;
        }

        GreyscaleImage::new(&image_matrix)
    }

    fn update(&mut self) {
        defmt::trace!("begin update call");

        if self.dead {
            // don't update anything, just return
            defmt::trace!("Not updating, snake is dead");
            return;
        }

        let old_snake_head =
            defmt::expect!(self.snake.pop_front(), "Snake deque unexpectedly empty!");
        let new_snake_head = match self.move_direction {
            MoveDirection::BoardUp => SnakeSegment(old_snake_head.0 - 1, old_snake_head.1),
            MoveDirection::BoardLeft => SnakeSegment(old_snake_head.0, old_snake_head.1 - 1),
            MoveDirection::BoardRight => SnakeSegment(old_snake_head.0, old_snake_head.1 + 1),
            MoveDirection::BoardDown => SnakeSegment(old_snake_head.0 + 1, old_snake_head.1),
        };
        // wall collision check
        if new_snake_head.0 > (BOARD_HEIGHT - 1) || new_snake_head.1 > (BOARD_WIDTH - 1) {
            // die
            defmt::info!("Snake has died by colliding with a wall");
            self.dead = true;
            defmt::expect!(
                self.snake.push_front(old_snake_head),
                "Snake deque unexpectedly full!"
            ); // have to put the old head back so it renders
        } else {
            if let Some(food) = &self.food {
                if food.0 == new_snake_head.0 && food.1 == new_snake_head.1 {
                    // ate the food, remove from screen and don't shrink the tail
                    self.food = None;
                } else {
                    // no food eaten, remove the tail before we add the new head
                    defmt::expect!(self.snake.pop_back(), "Snake deque unexpectedly empty!");
                }
            } else {
                defmt::expect!(self.snake.pop_back(), "Snake deque unexpectedly empty!");
            }
            defmt::expect!(
                self.snake.push_front(old_snake_head),
                "Snake deque unexpectedly full!"
            );
            defmt::expect!(
                self.snake.push_front(new_snake_head),
                "Snake deque unexpectedly full!"
            );
        }
    }
}

static DISPLAY: Mutex<RefCell<Option<Display<TIMER1>>>> = Mutex::new(RefCell::new(None));
static FRAME_TIMER: Mutex<RefCell<Option<Rtc<RTC0>>>> = Mutex::new(RefCell::new(None));
static GAME_STATE: Mutex<RefCell<Option<

#[entry]
fn main() -> ! {
    defmt::info!("Starting snake-microbit");
    let mut board = defmt::expect!(
        Board::take(),
        "Catastrophic failure, unable to take Board object!"
    );

    Clocks::new(board.CLOCK).start_lfclk(); //start low frequency clock needed by RTC0

    let mut button_a = board.buttons.button_a;
    let mut button_b = board.buttons.button_b;

    let mut display = Display::new(board.TIMER1, board.display_pins);

    let mut rtc0 = Rtc::new(board.RTC0, 63).unwrap();
    rtc0.enable_event(RtcInterrupt::Tick);
    rtc0.enable_interrupt(RtcInterrupt::Tick, None);
    rtc0.enable_counter();

    cortex_m::interrupt::free(move |cs| {
        *DISPLAY.borrow(cs).borrow_mut() = Some(display);
        *FRAME_TIMER.borrow(cs).borrow_mut() = Some(rtc0);
    });

    unsafe {
        board.NVIC.set_priority(pac::Interrupt::TIMER1, 64);
        board.NVIC.set_priority(pac::Interrupt::RTC0, 128);
        pac::NVIC::unmask(pac::Interrupt::TIMER1);
        pac::NVIC::unmask(pac::Interrupt::RTC0);
    }

    let mut game_board = GameState::new();
    let mut frames_in_turn_count = 0;

    let mut left_button_down = false;
    let mut right_button_down = false;

    let mut left_turn_count = 0;
    let mut right_turn_count = 0;

    loop {
        cortex_m::asm::wfi();
    }
}

#[interrupt]
fn TIMER1() {
    cortex_m::interrupt::free(move |cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.handle_display_event();
        }
    });
}

#[interrupt]
fn RTC0() {
    defmt::trace!("Begin RTC frame loop");
    defmt::trace!("Calling display.show");
    let image = game_board.render_image();
    cortex_m::interrupt::free(move |cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(&image);
        }
    });

    // detect a button press on button-up, not button-down, to help avoid repeats
    if !left_button_down
        && button_a
            .is_low()
            .expect("Unexpected button error, button GPIO should be infallible on target platform!")
    {
        left_button_down = true;
    }
    if left_button_down
        && button_a
            .is_high()
            .expect("Unexpected button error, button GPIO should be infallible on target platform!")
    {
        left_turn_count += 1;
        left_button_down = false;
    }

    if !right_button_down
        && button_b
            .is_low()
            .expect("Unexpected button error, button GPIO should be infallible on target platform!")
    {
        right_button_down = true;
    }
    if right_button_down
        && button_b
            .is_high()
            .expect("Unexpected button error, button GPIO should be infallible on target platform!")
    {
        right_turn_count += 1;
        right_button_down = false;
    }

    if frames_in_turn_count == FRAMES_PER_MOVE {
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
