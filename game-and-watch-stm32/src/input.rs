use embassy_stm32::{
    gpio::Input,
    peripherals::{PD11, PD15, PD0, PD14, PD9, PD5, PC1, PC4, PC13, PA0}
};

use embedded_hal::digital::InputPin;

pub struct Buttons<'a> {
    left: Input<'a>/*<PD11>*/,
    right: Input<'a>/*<PD15>*/,
    up: Input<'a>/*<PD0>*/,
    down: Input<'a>/*<PD14>*/,
    a: Input<'a>/*<PD9>*/,
    b: Input<'a>/*<PD5>*/,
    game: Input<'a>/*<PC1>*/,
    time: Input<'a>/*<PC4>*/,
    pause: Input<'a>/*<PC13>*/,
    power: Input<'a>/*<PA0>*/,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ButtonReading {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub game: bool,
    pub time: bool,
    pub pause: bool,
    pub power: bool
}

impl <'a> Buttons <'a> {

    pub fn new(
        left: Input<'a>/*<PD11>*/,
        right: Input<'a>/*<PD15>*/,
        up: Input<'a>/*<PD0>*/,
        down: Input<'a>/*<PD14>*/,
        a: Input<'a>/*<PD9>*/,
        b: Input<'a>/*<PD5>*/,
        game: Input<'a>/*<PC1>*/,
        time: Input<'a>/*<PC4>*/,
        pause: Input<'a>/*<PC13>*/,
        power: Input<'a>/*<PA0>*/,
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

    pub fn read(&self) -> ButtonReading {
        ButtonReading {
            left: self.left.is_low(),
            right: self.right.is_low(),
            up: self.up.is_low(),
            down: self.down.is_low(),
            a: self.a.is_low(),
            b: self.b.is_low(),
            game: self.game.is_low(),
            time: self.time.is_low(),
            pause: self.pause.is_low(),
            power: self.power.is_low(),
        }
    }
}
