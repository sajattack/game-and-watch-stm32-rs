use stm32h7xx_hal::{gpio::{Pin, Input}, prelude::*, delay::{Delay, DelayExt}};
use embedded_hal::digital::InputPin;

pub struct ButtonState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub time: bool,
    pub game: bool,
    pub pause: bool,
}

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
}

impl ButtonPins {
    pub fn new(
        pd11: Pin<'D', 11, Input>,
        pd15: Pin<'D', 15, Input>,
        pd0: Pin<'D', 0, Input>,
        pd14: Pin<'D', 14, Input>,
        pd9: Pin<'D', 9, Input>,
        pd5: Pin<'D', 5, Input>,
        pc1: Pin<'C', 1, Input>,
        pc4: Pin<'C', 4, Input>,
        pc13: Pin<'C', 13, Input>,
    ) -> Self {
        Self {
            left: pd11,
            right: pd15,
            up: pd0,
            down: pd14,
            a: pd9,
            b: pd5,
            time: pc4,
            game: pc1,
            pause: pc13
        }
    }

    pub fn read(&self) -> ButtonState {
        ButtonState {
            left: self.left.is_low(),
            right: self.right.is_low(),
            up: self.up.is_low(),
            down: self.down.is_low(),
            a: self.a.is_low(),
            b: self.b.is_low(),
            game: self.game.is_low(),
            time: self.time.is_low(),
            pause:self. pause.is_low(),
        }
    }

    pub fn read_debounced(&self, delay: &mut Delay, total_ms_div3: u32) -> ButtonState {
        let read1 = ButtonState {
            left: self.left.is_low(),
            right: self.right.is_low(),
            up: self.up.is_low(),
            down: self.down.is_low(),
            a: self.a.is_low(),
            b: self.b.is_low(),
            game: self.game.is_low(),
            time: self.time.is_low(),
            pause:self. pause.is_low(),
        };
        delay.delay_ms(total_ms_div3);

        let read2 = ButtonState {
            left: self.left.is_low(),
            right: self.right.is_low(),
            up: self.up.is_low(),
            down: self.down.is_low(),
            a: self.a.is_low(),
            b: self.b.is_low(),
            game: self.game.is_low(),
            time: self.time.is_low(),
            pause:self. pause.is_low(),
        };
        delay.delay_ms(total_ms_div3);

        let read3 = ButtonState {
            left: self.left.is_low(),
            right: self.right.is_low(),
            up: self.up.is_low(),
            down: self.down.is_low(),
            a: self.a.is_low(),
            b: self.b.is_low(),
            game: self.game.is_low(),
            time: self.time.is_low(),
            pause:self. pause.is_low(),
        };
        delay.delay_ms(total_ms_div3);

        let read4 = ButtonState {
            left: self.left.is_low(),
            right: self.right.is_low(),
            up: self.up.is_low(),
            down: self.down.is_low(),
            a: self.a.is_low(),
            b: self.b.is_low(),
            game: self.game.is_low(),
            time: self.time.is_low(),
            pause:self. pause.is_low(),
        };

        ButtonState {
            left: read1.left && read2.left && read3.left && read4.left,
            right: read1.right && read2.right && read3.right && read4.right,
            up: read1.up && read2.up && read3.up && read4.up,
            down: read1.down && read2.down && read3.down && read4.down,
            a: read1.a && read2.a && read3.a && read4.a,
            b: read1.b&& read2.b && read3.b && read4.b,
            game: read1.game && read2.game && read3.game && read4.game,
            time: read1.time && read2.time && read3.time && read4.time,
            pause: read1.pause && read2.pause && read3.pause && read4.pause,
        }
    }
}
