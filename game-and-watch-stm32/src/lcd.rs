pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;

use embassy_stm32::{gpio::{Output}, spi::Spi, mode::Blocking};
use embassy_time::{Timer};

pub struct Lcd<'a> {
    backlight1: Output<'a>,
    backlight2: Output<'a>,
    backlight3: Output<'a>,
    disable_3v3: Output<'a>,
    enable_1v8: Output<'a>,
    reset: Output<'a>,
    cs: Output<'a>,
    spi:  Spi<'a, Blocking>,
}

impl<'a> Lcd<'a> {
    pub fn new(
        backlight1: Output<'a>,
        backlight2: Output<'a>,
        backlight3: Output<'a>,
        disable_3v3: Output<'a>,
        enable_1v8: Output<'a>,
        reset: Output<'a>,
        cs: Output<'a>,
        spi: Spi<'a, Blocking>,
    ) -> Self {
        Self {
            backlight1,
            backlight2,
            backlight3,
            disable_3v3,
            enable_1v8,
            reset,
            cs,
            spi,
        }
    }


    pub async fn init (
        &mut self
    )  {
        // reference impl https://github.com/ghidraninja/game-and-watch-base/blob/main/Core/Src/lcd.c 
        // turn everything off
        self.backlight_off();
        self.cs.set_high();
        self.disable_3v3.set_high();
        self.enable_1v8.set_low();


        self.reset.set_low();

        // turn everything back on
        self.backlight_on();
        self.disable_3v3.set_low();
        self.enable_1v8.set_high();
        Timer::after_millis(20).await;

        // boot sequence
        self.reset.set_high();
        Timer::after_millis(1).await;
        self.reset.set_low();
        Timer::after_millis(15).await;
        self.reset.set_high();
        Timer::after_millis(1).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x08u8, 0x80u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x6eu8, 0x80u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x80u8, 0x80u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x68u8, 0x00u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0xd0u8, 0x00u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x1bu8, 0x00u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0xe0u8, 0x00u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x6au8, 0x80u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x80u8, 0x00u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;

        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(&[0x14u8, 0x80u8]).unwrap();
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;
    }

    pub fn backlight_off(
        &mut self
    ) {
        self.backlight1.set_low();
        self.backlight2.set_low();
        self.backlight3.set_low();
    }

    pub fn backlight_on(
        &mut self
    ) {
        self.backlight1.set_high();
        self.backlight2.set_high();
        self.backlight3.set_high();
    }
}
