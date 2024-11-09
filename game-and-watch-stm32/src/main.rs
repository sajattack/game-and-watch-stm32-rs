#![no_main]
#![no_std]

mod lcd;
use lcd::*;

mod input;
use input::*;


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
use embassy_sync::{mutex::Mutex, blocking_mutex::raw::CriticalSectionRawMutex};

use defmt::{info, error, debug};
use defmt_rtt as _;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

static mut FRONT_BUFFER: [TargetPixelType; WIDTH * HEIGHT] = [0u16; WIDTH * HEIGHT];
static mut BACK_BUFFER: [TargetPixelType; WIDTH * HEIGHT] = [0u16; WIDTH * HEIGHT];

static BUTTONS: Mutex<CriticalSectionRawMutex, Option<Buttons>> = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("GAME & WATCH TEST");

    let mut config = Config::default();

    config.rcc.hsi = Some(HSIPrescaler::DIV1);
    config.rcc.sys = Sysclk::PLL1_P;

    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV16,
        mul: PllMul::MUL140,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV2),
        divr: Some(PllDiv::DIV2),
    });

    config.rcc.pll2 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV25,
        mul: PllMul::MUL192,
        divp: Some(PllDiv::DIV5),
        divq: Some(PllDiv::DIV2),
        divr: Some(PllDiv::DIV5), 
    });
    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL9,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV2), 
        divr: Some(PllDiv::DIV24),
    });

    config.rcc.supply_config = SupplyConfig::LDO;
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV2;
    config.rcc.apb3_pre = APBPrescaler::DIV2;
    config.rcc.apb4_pre = APBPrescaler::DIV2;
    config.rcc.voltage_scale = VoltageScale::Scale0;

    config.rcc.mux.sai1sel = Saisel::PLL2_P;
    config.rcc.mux.spi123sel = Saisel::PLL3_P;

    pac::RCC.ahb1enr().modify(|w| { w.set_dma1en(true) });
    
    let cp = embassy_stm32::init(config);

    let cs = Output::new(cp.PB12, Level::High, Speed::Low);

    let pa4 = Output::new(cp.PA4, Level::Low, Speed::Low);
    let pa5 = Output::new(cp.PA5, Level::Low, Speed::Low);
    let pa6 = Output::new(cp.PA6, Level::Low, Speed::Low);

    let disable_3v3 = Output::new(cp.PD1, Level::High, Speed::Low);
    let enable_1v8  = Output::new(cp.PD4, Level::Low, Speed::Low);
    let reset = Output::new(cp.PD8, Level::High, Speed::Low);

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = mhz(18);

    let spi = Spi::<Blocking>::new_blocking_txonly(
        cp.SPI2,
        cp.PB13,
        cp.PB15,
        spi_config 
    );

    let mut ltdc_clk = Flex::new(cp.PB14);
        ltdc_clk.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_vsync = Flex::new(cp.PA7);
        ltdc_vsync.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_hsync = Flex::new(cp.PC6);
        ltdc_hsync.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_de = Flex::new(cp.PE13);
        ltdc_de.set_high();
        ltdc_de.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_r7 = Flex::new(cp.PE15);
        ltdc_r7.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_r6 = Flex::new(cp.PA8);
        ltdc_r6.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_r5 = Flex::new(cp.PA9);
        ltdc_r5.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_r4 = Flex::new(cp.PA11);
        ltdc_r4.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_r3 = Flex::new(cp.PB0);
        ltdc_r3.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_r2 = Flex::new(cp.PC10);
        ltdc_r2.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_g7 = Flex::new(cp.PD3);
        ltdc_g7.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_g6 = Flex::new(cp.PC7);
        ltdc_g6.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_g5 = Flex::new(cp.PB11);
        ltdc_g5.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_g4 = Flex::new(cp.PB10);
        ltdc_g4.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_g3 = Flex::new(cp.PC9);
        ltdc_g3.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_g2 = Flex::new(cp.PC0);
        ltdc_g2.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_b7 = Flex::new(cp.PD2);
        ltdc_b7.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_b6 = Flex::new(cp.PB8);
        ltdc_b6.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_b5 = Flex::new(cp.PB5);
        ltdc_b5.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_b4 = Flex::new(cp.PA10);
        ltdc_b4.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_b3 = Flex::new(cp.PD10);
        ltdc_b3.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));
    let mut ltdc_b2 = Flex::new(cp.PD6);
        ltdc_b2.set_as_af_unchecked(14, AfType::output(OutputType::PushPull, Speed::High));

    defmt::info!("Clocks: 
        SPI2: {}
        LTDC: {}",
        rcc::frequency::<peripherals::SPI2>(),
        rcc::frequency::<peripherals::LTDC>()
    );

    defmt::info!("SPI2 Config: {:?}", spi.get_current_config().frequency);


    let mut lcd = Lcd::new(pa4, pa5, pa6, disable_3v3, enable_1v8, reset, cs, spi);

    lcd.init().await;

    let mut ltdc = Ltdc::new(
        cp.LTDC
    );

    let ltdc_config = LtdcConfiguration {
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

    ltdc.init(&ltdc_config);

    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::RGB565,
        layer: ltdc::LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: WIDTH as u16,
        window_y0: 0,
        window_y1: HEIGHT as u16,
    };

    ltdc.init_layer(&layer_config, None);

    let mut disp = DoubleBuffer::new(
        unsafe { FRONT_BUFFER.as_mut() } ,
        unsafe { BACK_BUFFER.as_mut() } ,
        layer_config
    );

    info!("Initialised Display...");
    
    Timer::after_millis(200).await;

    let buttons: Buttons = ButtonPins::new(
        Input::new(cp.PD11, Pull::None),
        Input::new(cp.PD15, Pull::None),
        Input::new(cp.PD0,  Pull::None),
        Input::new(cp.PD14,  Pull::None),
        Input::new(cp.PD9,  Pull::None),
        Input::new(cp.PD5,  Pull::None),
        Input::new(cp.PC1,  Pull::None),
        Input::new(cp.PC4,  Pull::None),
        Input::new(cp.PC13,  Pull::None),
        Input::new(cp.PA0,  Pull::None)
    ).into();

    {
        *(BUTTONS.lock().await) = Some(buttons);
    }

    spawner.spawn(input_task());

    let mut ferris_pos = Point::new(120, 125);
    let mut br = ButtonReading::default();

    loop { 
        {
            let mut buttons = BUTTONS.lock().await;
            if let Some(b) = buttons.as_mut() {
                br = b.read_all();
            }
        }

        if br.left {
            ferris_pos.x -= 1;
        }
        if br.right {
            ferris_pos.x += 1;
        }
        if br.up {
            ferris_pos.y -= 1;
        }
        if br.down {
            ferris_pos.y +=1;
        }

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
        Timer::after_micros(16667).await;
   }
}

#[embassy_executor::task]
pub async fn input_task() -> ! {
    loop {
        {
            let mut buttons = BUTTONS.lock().await;
            if let Some(b) = buttons.as_mut()
            {
                b.tick_all();
            }
        }
        Timer::after_micros(500).await;
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

