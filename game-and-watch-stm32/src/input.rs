use embassy_stm32::{
    gpio::Input,
    peripherals::{PD11, PD15, PD0, PD14, PD9, PD5, PC1, PC4, PC13, PA0}
};

use embedded_hal::digital::InputPin;

use embassy_time::{Instant, Duration};

use button_driver::{Button, InstantProvider, ButtonConfig, Mode};

use core::default::Default;

// FIXME Pin type safety

pub struct ButtonPins<'a> {
    left: Input<'a>,
    right: Input<'a>,
    up: Input<'a>,
    down: Input<'a>,
    a: Input<'a>,
    b: Input<'a>,
    game: Input<'a>,
    time: Input<'a>,
    pause: Input<'a>,
    power: Input<'a>,
}

impl <'a> ButtonPins <'a> {

    pub fn new(
        left: Input<'a>,
        right: Input<'a>,
        up: Input<'a>,
        down: Input<'a>,
        a: Input<'a>,
        b: Input<'a>,
        game: Input<'a>,
        time: Input<'a>,
        pause: Input<'a>,
        power: Input<'a>,
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

pub struct Buttons<'a> {
    pub left: Button<Input<'a>, Instant, Duration>,
    pub right: Button<Input<'a>, Instant, Duration>,
    pub up: Button<Input<'a>, Instant, Duration>,
    pub down: Button<Input<'a>, Instant, Duration>,
    pub a: Button<Input<'a>, Instant, Duration>,
    pub b: Button<Input<'a>, Instant, Duration>,
    pub game: Button<Input<'a>, Instant, Duration>,
    pub time: Button<Input<'a>, Instant, Duration>,
    pub pause: Button<Input<'a>, Instant, Duration>,
    pub power: Button<Input<'a>, Instant, Duration>
}


impl <'a> Buttons <'a> {
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

    pub fn read_all(&mut self) -> ButtonReading {
        ButtonReading {
            left: self.left.raw_state().is_held(),
            right: self.right.raw_state().is_held(),
            up: self.up.raw_state().is_held(),
            down: self.down.raw_state().is_held(),
            a: self.a.raw_state().is_held(),
            b: self.b.raw_state().is_held(),
            game: self.game.raw_state().is_held(),
            time: self.time.raw_state().is_held(),
            pause: self.pause.raw_state().is_held(),
            power: self.power.raw_state().is_held(),
        }
    }
}

impl<'a> From<ButtonPins<'a>> for Buttons<'a> {
    fn from(value: ButtonPins<'a>) -> Self {
        let config = ButtonConfig::<Duration> {
            mode: Mode::PullUp,
            hold: Duration::from_millis(16),
            release: Duration::from_millis(8),
            ..Default::default()
        };
        Self {
            left: Button::new(value.left, config),
            right: Button::new(value.right, config),
            up: Button::new(value.up, config),
            down: Button::new(value.down, config),
            a:  Button::new(value.a, config),
            b: Button::new(value.b, config),
            game: Button::new(value.game, config),
            time: Button::new(value.time, config),
            pause: Button::new(value.pause, config),
            power: Button::new(value.power, config),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct ButtonReading {
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
