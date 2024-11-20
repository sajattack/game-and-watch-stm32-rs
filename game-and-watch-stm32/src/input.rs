use button_driver::{Button, ButtonConfig, Mode, State, InstantProvider};
use core::{ops::Sub};
use stm32h7xx_hal::{
    pac::{self, interrupt, Interrupt},
    prelude::*,
    timer::{Event},
    gpio::{self, Pin, Input},
};
use embedded_hal::digital::v2::InputPin;
use fugit::{Instant, Duration};


pub struct ButtonPins {
    left: Pin<'D', 11, Input>,
    right: Pin<'D', 15, Input>,
    up: Pin<'D', 0, Input>,
    down: Pin<'D', 14, Input>,
    a: Pin<'D', 9, Input>,
    b: Pin<'D', 5, Input>,
    game: Pin<'C', 1, Input>,
    time: Pin<'C', 4, Input>,
    pause: Pin<'C', 13, Input>,
    power: Pin<'A', 0, Input>,
}

impl ButtonPins {

    pub fn new(
        left: Pin<'D', 11, Input>,
        right: Pin<'D', 15, Input>,
        up: Pin<'D', 0, Input>,
        down: Pin<'D', 14, Input>,
        a: Pin<'D', 9, Input>,
        b: Pin<'D', 5, Input>,
        game: Pin<'C', 1, Input>,
        time: Pin<'C', 4, Input>,
        pause: Pin<'C', 13, Input>,
        power: Pin<'A', 0, Input>,
    ) -> Self {
        Self {
            left,
            right,
            up,
            down, 
            a,
            b,
            time,
            game,
            pause,
            power,
        }
    }
}

pub struct Buttons {
    pub left: Button<Pin<'D', 11, Input>, Instant, Duration<u16, 1, 1000>>,
    pub right: Button<Pin<'D', 15, Input>, Instant, Duration<u16, 1, 1000>>,
    pub up: Button<Pin<'D', 0, Input>, Instant, Duration<u16, 1, 1000>>,
    pub down: Button<Pin<'D', 14, Input>, Instant, Duration<u16, 1, 1000>>,
    pub a: Button<Pin<'D', 9, Input>, Instant, Duration<u16, 1, 1000>>,
    pub b: Button<Pin<'D', 5, Input>, Instant, Duration<u16, 1, 1000>>,
    pub game: Button<Pin<'C', 1, Input>, Instant, Duration<u16, 1, 1000>>,
    pub time: Button<Pin<'C', 4, Input>, Instant, Duration<u16, 1, 1000>>,
    pub pause: Button<Pin<'C', 13, Input>, Instant, Duration<u16, 1, 1000>>,
    pub power: Button<Pin<'A', 0, Input>, Instant, Duration<u16, 1, 1000>>
}


impl Buttons {
    pub fn tick_all(&mut self) {
        self.left.tick();
        self.right.tick();
        self.up.tick();
        self.down.tick();
        self.a.tick();
        self.b.tick();
        self.game.tick();
        self.time.tick();
        self.pause.tick();
        self.power.tick();
    }

    pub fn reset_all(&mut self) {
        self.left.reset();
        self.right.reset();
        self.up.reset();
        self.down.reset();
        self.a.reset();
        self.b.reset();
        self.game.reset();
        self.time.reset();
        self.pause.reset();
        self.power.reset();
    }

    pub fn raw_read_all(&mut self) -> ButtonReading {
        ButtonReading {
            left: self.left.raw_state().clone(),
            right: self.right.raw_state().clone(),
            up: self.up.raw_state().clone(),
            down: self.down.raw_state().clone(),
            a: self.a.raw_state().clone(),
            b: self.b.raw_state().clone(),
            game: self.game.raw_state().clone(),
            time: self.time.raw_state().clone(),
            pause: self.pause.raw_state().clone(),
            power: self.power.raw_state().clone(),
        }
    }

    pub fn read_clicks(&mut self) -> ButtonClick {
       ButtonClick {
            left: self.left.is_clicked(),
            right: self.right.is_clicked(),
            up: self.up.is_clicked(),
            down: self.down.is_clicked(),
            a: self.a.is_clicked(), 
            b: self.b.is_clicked(),
            time: self.time.is_clicked(),
            game: self.game.is_clicked(), 
            pause: self.pause.is_clicked(), 
            power: self.power.is_clicked(), 
       }
    }
}

impl From<ButtonPins> for Buttons {
    fn from(value: ButtonPins) -> Self {
        let fast_config = ButtonConfig::<Duration> {
            mode: Mode::PullUp,
            hold: Duration::from_millis(100),
            release: Duration::from_millis(50),
            ..Default::default()
        };

        let slow_config = ButtonConfig::<Duration> {
            mode: Mode::PullUp,
            ..Default::default()
        };

        Self {
            left: Button::new(value.left, fast_config),
            right: Button::new(value.right, fast_config),
            up: Button::new(value.up, fast_config),
            down: Button::new(value.down, fast_config),
            a:  Button::new(value.a, fast_config),
            b: Button::new(value.b, fast_config),
            game: Button::new(value.game, slow_config),
            time: Button::new(value.time, slow_config),
            pause: Button::new(value.pause, slow_config),
            power: Button::new(value.power, slow_config),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ButtonReading {
    pub left: State<Instant>,
    pub right: State<Instant>,
    pub up: State<Instant>,
    pub down:  State<Instant>,
    pub a: State<Instant>,
    pub b: State<Instant>,
    pub time: State<Instant>,
    pub game:  State<Instant>,
    pub pause: State<Instant>,
    pub power: State<Instant>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ButtonClick {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub time: bool,
    pub game: bool,
    pub pause: bool,
    pub power: bool,
}
