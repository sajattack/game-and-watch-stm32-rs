use embassy_stm32::{
    gpio::Input,
    //peripherals::{PD11, PD15, PD0, PD14, PD9, PD5, PC1, PC4, PC13, PA0}
};

use embassy_time::{Instant, Duration};

use button_driver::{Button, ButtonConfig, Mode, State};

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

impl<'a> From<ButtonPins<'a>> for Buttons<'a> {
    fn from(value: ButtonPins<'a>) -> Self {
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
