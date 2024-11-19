pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;

use embassy_stm32::{gpio::{Output}, spi::{self, Spi}, mode::Blocking,
    ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayerConfig, PolarityActive, PolarityEdge},
};
use embassy_time::{Timer};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;

pub struct Lcd<'a> {
    backlight1: Output<'a>,
    backlight2: Output<'a>,
    backlight3: Output<'a>,
    disable_3v3: Output<'a>,
    enable_1v8: Output<'a>,
    reset: Output<'a>,
    cs: Output<'a>,
    spi:  Spi<'a, Blocking>,
    backlight_state: bool,
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
            backlight_state: false,
        }
    }

    async fn spi_write(&mut self, buf: &[u8]) -> Result<(), spi::Error> {
        self.cs.set_low();
        Timer::after_millis(2).await;
        self.spi.blocking_write(buf)?;
        Timer::after_millis(2).await;
        self.cs.set_high();
        Timer::after_millis(2).await;
        Ok(())
    }

    async fn reset(&mut self) {
        self.reset.set_high();
        Timer::after_millis(1).await;
        self.reset.set_low();
        Timer::after_millis(15).await;
        self.reset.set_high();
        Timer::after_millis(1).await;
    }

    pub fn power_off(&mut self) {
        self.set_backlight_off();
        self.disable_3v3.set_high();
        self.enable_1v8.set_low();
    }

    pub fn power_on(&mut self) {
        self.set_backlight_on();
        self.disable_3v3.set_low();
        self.enable_1v8.set_high();
    }

    pub async fn init (
        &mut self
    ) -> Result<(), spi::Error> {
        // reference impl https://github.com/ghidraninja/game-and-watch-base/blob/main/Core/Src/lcd.c 
        // other reference impl that makes a bit more sense 
        // https://github.com/kbeckmann/game-and-watch-retro-go/blob/main/Core/Src/gw_lcd.c

        self.cs.set_high();
        self.power_off();
        self.power_on();
        Timer::after_millis(20).await;
        self.reset().await;

        self.spi_write(&[0x08, 0x80]).await?;
        self.spi_write(&[0x6e, 0x80]).await?;
        self.spi_write(&[0x80, 0x80]).await?;
        self.spi_write(&[0x68, 0x00]).await?;
        self.spi_write(&[0xd0, 0x00]).await?;
        self.spi_write(&[0x1b, 0x00]).await?;
        self.spi_write(&[0xe0, 0x00]).await?;
        self.spi_write(&[0x6a, 0x80]).await?;
        self.spi_write(&[0x80, 0x00]).await?;
        self.spi_write(&[0x14, 0x80]).await?;

        Ok(())
    }

    pub fn set_backlight_off(
        &mut self
    ) {
        self.backlight1.set_low();
        self.backlight2.set_low();
        self.backlight3.set_low();
        self.backlight_state = false;
    }

    pub fn set_backlight_on(
        &mut self
    ) {
        self.backlight1.set_high();
        self.backlight2.set_high();
        self.backlight3.set_high();
        self.backlight_state = true;
    }

    pub fn toggle_backlight(
        &mut self
    ) {
        if self.backlight_state {
            self.set_backlight_off();
        } else {
            self.set_backlight_on();
        }
    }
}


pub type TargetPixelType = u16;

// A simple double buffer
pub struct DoubleBuffer<'a> {
    buf0: &'a mut [TargetPixelType],
    buf1: &'a mut [TargetPixelType],
    is_buf0: bool,
    layer_config: LtdcLayerConfig,
}

impl<'a> DoubleBuffer<'a> {
    pub fn new(
        buf0: &'a mut [TargetPixelType],
        buf1: &'a mut [TargetPixelType],
        layer_config: LtdcLayerConfig,
    ) -> Self {
        Self {
            buf0,
            buf1,
            is_buf0: true,
            layer_config,
        }
    }

    pub fn current(&mut self) -> &mut [TargetPixelType] {
        if self.is_buf0 {
            self.buf0
        } else {
            self.buf1
        }
    }

    pub async fn swap<T: ltdc::Instance>(&mut self, ltdc: &mut Ltdc<'_, T>) -> Result<(), ltdc::Error> {
        let buf = self.current();
        let frame_buffer = buf.as_ptr();
        self.is_buf0 = !self.is_buf0;
        ltdc.set_buffer(self.layer_config.layer, frame_buffer as *const _).await
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        let buf = self.current();
        let black = Rgb565::new(0, 0, 0).into_storage();

        for a in buf.iter_mut() {
            *a = black; // solid black
        }
    }
}

impl<'a> DrawTarget for DoubleBuffer<'a> {
    type Color = Rgb565;
    type Error = ();

    /// Draw a pixel
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();
        let width = size.width as i32;
        let height = size.height as i32;
        let buf = self.current();

        for pixel in pixels {
            let Pixel(point, color) = pixel;

            if point.x >= 0 && point.y >= 0 && point.x < width && point.y < height {
                let index = point.y * width + point.x;
                let raw_color = color.into_storage();
                buf[index as usize] = raw_color;
            } else {
                // Ignore invalid points
                //defmt::error!("Invalid address");
            }
        }

        Ok(())
    }
}

impl<'a> OriginDimensions for DoubleBuffer<'a> {
    /// Return the size of the display
    fn size(&self) -> Size {
        Size::new(
            (self.layer_config.window_x1 - self.layer_config.window_x0) as _,
            (self.layer_config.window_y1 - self.layer_config.window_y0) as _,
        )
    }
}

pub static LTDC_CONFIG: LtdcConfiguration  = LtdcConfiguration {
    active_width: WIDTH as u16,
    active_height: HEIGHT as u16,
    h_back_porch: 200,
    h_front_porch: 431,
    v_back_porch: 3,
    v_front_porch: 3,
    h_sync: 10,
    v_sync: 2,
    h_sync_polarity: PolarityActive::ActiveLow,
    v_sync_polarity: PolarityActive::ActiveLow,
    data_enable_polarity: PolarityActive::ActiveLow,
    pixel_clock_polarity: PolarityEdge::FallingEdge,
};

pub static LTDC_LAYER_CONFIG: LtdcLayerConfig = LtdcLayerConfig {
    pixel_format: ltdc::PixelFormat::RGB565,
    layer: ltdc::LtdcLayer::Layer1,
    window_x0: 0,
    window_x1: WIDTH as u16,
    window_y0: 0,
    window_y1: HEIGHT as u16,
};
