#![no_main]
#![no_std]

mod lcd;
use lcd::*;

use core::ptr::addr_of_mut;

use cortex_m_rt::entry;

use embedded_graphics::{image::Image, primitives::Rectangle, pixelcolor::Rgb565};
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

use tinybmp::Bmp;

use embassy_stm32::{
    Config, rcc::{self, *, SupplyConfig, mux::Saisel},
    gpio::{self, Pull, Input, Output, Flex, Speed, Level, AfType, OutputType},
    Peripherals,
    ltdc::{self, Ltdc, PolarityEdge, PolarityActive, LtdcConfiguration, LtdcLayerConfig},
    mode::Blocking,
    peripherals,
    time::{Hertz, hz, mhz},
    bind_interrupts,
    interrupt::{self, typelevel::LTDC},
    pac,
};

use embassy_stm32::spi::{self, Config as SpiConfig, Spi};
use embassy_time::Timer;
use embassy_executor::Spawner;

use defmt::{info, error};
use defmt_rtt as _;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

static mut FRONT_BUFFER: [TargetPixelType; WIDTH * HEIGHT] = [0u16; WIDTH * HEIGHT];
static mut BACK_BUFFER: [TargetPixelType; WIDTH * HEIGHT] = [0u16; WIDTH * HEIGHT];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("GAME & WATCH TEST");

    let mut config = Config::default();

    config.rcc.hsi = Some(HSIPrescaler::DIV1);
    config.rcc.sys = Sysclk::PLL1_P;

    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV32,
        mul: PllMul::MUL200,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV2),
        divr: Some(PllDiv::DIV2),
    });

    config.rcc.pll2 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV32,
        mul: PllMul::MUL210,
        divp: Some(PllDiv::DIV7),
        divq: Some(PllDiv::DIV2),
        divr: Some(PllDiv::DIV7), 
    });
    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV32,
        mul: PllMul::MUL200,
        divp: Some(PllDiv::DIV8),
        divq: Some(PllDiv::DIV2), 
        divr: Some(PllDiv::DIV2),
    });

    config.rcc.supply_config = SupplyConfig::LDO;
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV2;
    config.rcc.apb3_pre = APBPrescaler::DIV2;
    config.rcc.apb4_pre = APBPrescaler::DIV2;
    config.rcc.voltage_scale = VoltageScale::Scale0;


    config.rcc.ls = LsConfig {
        rtc: RtcClockSource::LSE,
        lsi: true,
        lse: Some(LseConfig {
            frequency: hz(32768),
            mode: LseMode::Oscillator(LseDrive::High)
        })
    };

    config.rcc.mux.spi123sel = Saisel::PLL2_P;
    
    pac::RCC.apb3enr().modify(|w| { w.set_ltdcen(true) });

    let cp = embassy_stm32::init(config);


    let cs = Output::new(cp.PB12, Level::High, Speed::Low);

    let pa4 = Output::new(cp.PA4, Level::Low, Speed::Low);
    let pa5 = Output::new(cp.PA5, Level::Low, Speed::Low);
    let pa6 = Output::new(cp.PA6, Level::Low, Speed::Low);

    let disable_3v3 = Output::new(cp.PD1, Level::High, Speed::Low);
    let enable_1v8  = Output::new(cp.PD4, Level::Low, Speed::Low);
    let reset = Output::new(cp.PD8, Level::High, Speed::Low);

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = mhz(15);

    let spi = Spi::<Blocking>::new_blocking_txonly(
        cp.SPI2,
        cp.PB13,
        cp.PB15,
        spi_config 
    );

    //let buttons = input::ButtonPins::new(
       //gpiod.pd11.into_input(),
       //gpiod.pd15.into_input(),
       //gpiod.pd0.into_input(),
       //gpiod.pd14.into_input(),
       //gpiod.pd9.into_input(),
       //gpiod.pd5.into_input(),
       //gpioc.pc1.into_input(),
       //gpioc.pc4.into_input(),
       //gpioc.pc13.into_input(),
    //);
    let pd11 = Input::new(cp.PD11, Pull::None);
    let pd15 = Input::new(cp.PD15, Pull::None);
    let pd0 = Input::new(cp.PD0, Pull::None);
    let pd14 = Input::new(cp.PD14, Pull::None);
    let pd9 = Input::new(cp.PD9, Pull::None);
    let pd5 = Input::new(cp.PD5, Pull::None);
    let pc1 = Input::new(cp.PC1, Pull::None);
    let pc4 = Input::new(cp.PC4, Pull::None);
    let pc13 = Input::new(cp.PC13, Pull::None);


    let _ltdc_clk = Flex::new(cp.PB14).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_vsync = Flex::new(cp.PA7).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_hsync = Flex::new(cp.PC6).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_de = Flex::new(cp.PE13).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_r7 = Flex::new(cp.PE15).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_r6 = Flex::new(cp.PA8).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_r5 = Flex::new(cp.PA9).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_r4 = Flex::new(cp.PA11).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_r3 = Flex::new(cp.PB0).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_r2 = Flex::new(cp.PC10).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_g7 = Flex::new(cp.PD3).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_g6 = Flex::new(cp.PC7).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_g5 = Flex::new(cp.PB11).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_g4 = Flex::new(cp.PB10).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_g3 = Flex::new(cp.PC9).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_g2 = Flex::new(cp.PC0).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_b7 = Flex::new(cp.PD2).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_b6 = Flex::new(cp.PB8).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_b5 = Flex::new(cp.PB5).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_b4 = Flex::new(cp.PA10).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_b3 = Flex::new(cp.PD10).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));
    let _ltdc_b2 = Flex::new(cp.PD6).set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::VeryHigh));



    defmt::info!("Clocks: 
        SPI2: {}
        LTDC: {}",
        rcc::frequency::<peripherals::SPI2>(),
        rcc::frequency::<peripherals::LTDC>()
        );

    defmt::info!("SPI2 Config: {:?}", spi.get_current_config().frequency);


    let mut lcd = Lcd::new(pa4, pa5, pa6, disable_3v3, enable_1v8, reset, cs, spi);

    lcd.init().await;

    Timer::after_millis(200).await;

    let mut ltdc = Ltdc::new(
        cp.LTDC
    );

    let ltdc_config = LtdcConfiguration {
        active_width: WIDTH as u16,
        active_height: HEIGHT as u16,
        h_back_porch: 200,
        h_front_porch: 430,
        v_back_porch: 3,
        v_front_porch: 3,
        h_sync: 10,
        v_sync: 2,
        h_sync_polarity: PolarityActive::ActiveLow,
        v_sync_polarity: PolarityActive::ActiveLow,
        data_enable_polarity: PolarityActive::ActiveLow,
        pixel_clock_polarity: PolarityEdge::FallingEdge,
    };

    ltdc.init(&ltdc_config);

    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::RGB565,
        layer: ltdc::LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: WIDTH as u16,
        window_y0: 0,
        window_y1: HEIGHT as u16,
    };

    let mut layer = ltdc.init_layer(&layer_config, None);

    let mut disp = DoubleBuffer::new(
        unsafe { FRONT_BUFFER.as_mut() } ,
        unsafe { BACK_BUFFER.as_mut() } ,
        layer_config
    );

    info!("Initialised Display...");
    

    let mut ferris_pos = Point::new(120, 125);

    loop { 
        //let button_state = buttons.read_debounced(&mut delay, 4);
        //if button_state.left {
            //ferris_pos.x -= 1;
        //}
        //if button_state.right {
            //ferris_pos.x += 1;
        //}
        //if button_state.up {
            //ferris_pos.y -= 1;
        //}
        //if button_state.down {
            //ferris_pos.y += 1;
        //}
        disp.clear();
        disp.fill_solid(&Rectangle::new(Point::new(0, 0), Size::new(320, 240)), RgbColor::RED).unwrap();
        let text_style =
            MonoTextStyle::new(&ascii::FONT_9X18, RgbColor::WHITE);
        Text::new("Hello Rust!", Point::new(120, 100), text_style)
            .draw(&mut disp)
            .unwrap();

        let ferris: Bmp<Rgb565> =
            Bmp::from_slice(include_bytes!("../assets/ferris.bmp")).unwrap();
        let ferris = Image::new(&ferris, ferris_pos);


        ferris.draw(&mut disp).unwrap();
        disp.swap(&mut ltdc).await.unwrap();
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

// Implement DrawTarget for
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
                defmt::error!("Invalid address");
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

