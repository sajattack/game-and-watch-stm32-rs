pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;

use stm32h7xx_hal::{delay::Delay, gpio::{Output, Pin, PushPull}, ltdc::{Ltdc, LtdcLayer1}, pac::SPI2, prelude::*, spi};

pub struct Lcd {
    backlight1: Pin<'A', 4, Output<PushPull>>,
    backlight2: Pin<'A', 5, Output<PushPull>>,
    backlight3: Pin<'A', 6, Output<PushPull>>,
    disable_3v3: Pin<'D', 1, Output<PushPull>>,
    enable_1v8: Pin<'D', 4, Output<PushPull>>,
    reset: Pin<'D', 8, Output<PushPull>>,
    cs: Pin<'B', 12, Output<PushPull>>,
    spi:  spi::Spi<SPI2, spi::Enabled, u8>,
}

impl Lcd {
    pub fn new(
        backlight1: Pin<'A', 4, Output<PushPull>>,
        backlight2: Pin<'A', 5, Output<PushPull>>,
        backlight3: Pin<'A', 6, Output<PushPull>>,
        disable_3v3: Pin<'D', 1, Output<PushPull>>,
        enable_1v8: Pin<'D', 4, Output<PushPull>>,
        reset: Pin<'D', 8, Output<PushPull>>,
        cs: Pin<'B', 12, Output<PushPull>>,
        spi:  spi::Spi<SPI2, spi::Enabled, u8>,
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

    fn spi_write<'a>(
        &mut self,
        buf: &'a [u8],
        delay: &'a mut Delay,
    ) -> Result<(), spi::Error> {
        self.cs.set_low();
        delay.delay_ms(2u8);
        self.spi.write(buf)?;
        delay.delay_ms(2u8);
        self.cs.set_high();
        delay.delay_ms(2u8);
        Ok(())
    }

    pub fn init<'a> (
        &mut self,
        delay: &'a mut Delay
    ) -> Result<(), spi::Error> {
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
        delay.delay_ms(20u8);

        // boot sequence
        self.reset.set_high();
        delay.delay_ms(1u8);
        self.reset.set_low();
        delay.delay_ms(15u8);
        self.reset.set_high();
        delay.delay_ms(1u8);

        self.spi_write(&[0x08; 0x80], delay)?;
        self.spi_write(&[0x6e; 0x80], delay)?;
        self.spi_write(&[0x80; 0x80], delay)?;
        self.spi_write(&[0x68; 0x00], delay)?;
        self.spi_write(&[0xd0; 0x00], delay)?;
        self.spi_write(&[0x1b; 0x00], delay)?;
        self.spi_write(&[0xe0; 0x00], delay)?;
        self.spi_write(&[0x6a; 0x80], delay)?;
        self.spi_write(&[0x80; 0x00], delay)?;
        self.spi_write(&[0x14, 0x80], delay)?;

        Ok(())
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
