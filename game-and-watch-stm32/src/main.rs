#![no_main]
#![no_std]

mod lcd;
use lcd::*;

mod input;
use input::*;

use embedded_graphics::{
    prelude::*,
    image::Image, primitives::Rectangle, pixelcolor::Rgb565,
    mono_font::{ascii, MonoTextStyle},
    text::Text,
};

use tinybmp::Bmp;

use embassy_stm32::{
    Config, rcc::{*, SupplyConfig, mux::Saisel},
    gpio::{Pull, Input, Output, Flex, Speed, Level, AfType, OutputType},
    spi::{Config as SpiConfig, Spi},
    ltdc::{self, Ltdc},
    mode::Blocking,
    peripherals,
    time::mhz,
    bind_interrupts,
    pac,
};

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

// this doesn't really need a mutex because it's only modified once but
// it's better than making it static mut I suppose
static FERRIS: Mutex<CriticalSectionRawMutex, Option<Bmp<Rgb565>>> = Mutex::new(None);

struct GameState {
    pub ferris_pos: Point,
    pub button_reading: Option<ButtonReading>,
    pub button_clicks: Option<ButtonClick>,
}

impl GameState {
    pub fn new() -> Self
    {
        Self {
            ferris_pos: Point::new(120, 125),
            button_reading: None,
            button_clicks: None,
        }
    }
}

async fn draw(gs: &GameState, display: &mut DoubleBuffer<'_>)
{
    display.clear();
    display.fill_solid(&Rectangle::new(Point::new(0, 0), Size::new(320, 240)), RgbColor::RED).unwrap();
    let text_style =
        MonoTextStyle::new(&ascii::FONT_9X18, RgbColor::WHITE);
    Text::new("Hello Rust!", Point::new(120, 100), text_style)
        .draw(display)
        .unwrap();

    {
        let ferris = FERRIS.lock().await;
        if let Some(f) = *ferris {
            let ferris_img = Image::new(&f, gs.ferris_pos);
            ferris_img.draw(display).unwrap();
        }
    }
}

async fn update(gs: &mut GameState, lcd: &mut Lcd<'_>) {
    {
        // read input state
        // MUTEX HELD!
        let mut buttons = BUTTONS.lock().await;
        if let Some(b) = buttons.as_mut() {
            gs.button_reading = Some(b.raw_read_all());
            gs.button_clicks = Some(b.read_clicks());
            b.reset_all();
        }
    }

    if let Some(button_state) = gs.button_reading {
        if button_state.left.is_held() {
            gs.ferris_pos.x -= 1;
        }
        if button_state.right.is_held() {
            gs.ferris_pos.x += 1;
        }
        if button_state.up.is_held() {
            gs.ferris_pos.y -= 1;
        }
        if button_state.down.is_held() {
            gs.ferris_pos.y +=1;
        }
    }

    if let Some(clicks) = gs.button_clicks {
        if clicks.power {
            lcd.toggle_backlight();
        }
    }
}

#[embassy_executor::task]
async fn input_task() -> ! {
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

    let mut lcd = Lcd::new(pa4, pa5, pa6, disable_3v3, enable_1v8, reset, cs, spi);

    lcd.init().await.unwrap();

    let mut ltdc = Ltdc::new(
        cp.LTDC
    );

    ltdc.init(&LTDC_CONFIG);

    ltdc.init_layer(&LTDC_LAYER_CONFIG, None);

    let mut disp = DoubleBuffer::new(
        unsafe { FRONT_BUFFER.as_mut() } ,
        unsafe { BACK_BUFFER.as_mut() } ,
        LTDC_LAYER_CONFIG
    );

    info!("Initialised Display...");

    Timer::after_millis(200).await;

    let buttons: Buttons = ButtonPins::new(
        Input::new(cp.PD11, Pull::None), // I think these have hardware pullups already
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

    // Initialize static button struct
    {
        *(BUTTONS.lock().await) = Some(buttons);
    }

    // Initialize static ferris struct
    {
        *(FERRIS.lock().await) = Some(Bmp::from_slice(include_bytes!("../assets/ferris.bmp")).unwrap());
    }

    // Initialize state
    let mut gs = GameState::new();

    // start polling for input asynchronously
    spawner.spawn(input_task()).unwrap();

    // main loop
    loop { 
        update(&mut gs, &mut lcd).await; 
        draw(&gs, &mut disp).await;
        disp.swap(&mut ltdc).await.unwrap();
   }
}


