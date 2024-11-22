#![no_main]
#![no_std]

mod lcd;
use grounded::uninit::GroundedArrayCell;
use lcd::*;

mod input;
use input::*;

mod spiflash;
use spiflash::*;

use embedded_graphics::{
    prelude::*,
    image::Image, primitives::Rectangle, pixelcolor::Rgb565,
    mono_font::{ascii, MonoTextStyle},
    text::Text,
};

use mux::{Fmcsel, Persel};
use tinybmp::Bmp;

use embassy_stm32::{
    bind_interrupts, dma::Channel, flash::{self, Bank1Region, Flash}, gpio::{AfType, Flex, Input, Level, Output, OutputType, Pull, Speed}, interrupt::{self, InterruptExt, Priority}, ltdc::{self, Ltdc}, mode::Blocking, pac, peripherals::{self, RNG, SAI1}, rcc::{self, mux::Saisel, SupplyConfig, *}, rng::{self, Rng}, sai::{self, Dma, FsPin, MasterClockDivider, MuteValue, Sai, SckPin, SdPin, SubBlock}, spi::{Config as SpiConfig, Spi}, time::mhz, Config, Peripheral, PeripheralRef,
};

use embassy_time::Timer;
use embassy_executor::{Spawner, InterruptExecutor};
use embassy_sync::{mutex::Mutex, blocking_mutex::raw::CriticalSectionRawMutex};

use defmt::{info, error, debug, trace};
use defmt_rtt as _;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

static mut FRONT_BUFFER: [TargetPixelType; WIDTH * HEIGHT] = [0u16; WIDTH * HEIGHT];
static mut BACK_BUFFER: [TargetPixelType; WIDTH * HEIGHT] = [0u16; WIDTH * HEIGHT];
static BUTTONS: Mutex<CriticalSectionRawMutex, Option<Buttons>> = Mutex::new(None);

// this doesn't really need a mutex because it's only modified once but
// it's better than making it static mut I suppose
static FERRIS: Mutex<CriticalSectionRawMutex, Option<Bmp<Rgb565>>> = Mutex::new(None);

static SAI: Mutex<CriticalSectionRawMutex, Option<Sai<'_, SAI1, u32>>> = Mutex::new(None);

static RNG: Mutex<CriticalSectionRawMutex, Option<Rng<'static, RNG>>> = Mutex::new(None);

static LTDC_OBJ: Mutex<CriticalSectionRawMutex, Option<Ltdc<'static, peripherals::LTDC>>> = Mutex::new(None);

static DISPLAY: Mutex<CriticalSectionRawMutex, Option<DoubleBuffer>> = Mutex::new(None);

static LCD: Mutex<CriticalSectionRawMutex, Option<Lcd>> = Mutex::new(None);

static GS: Mutex<CriticalSectionRawMutex, Option<GameState>> = Mutex::new(None);

static IE1: InterruptExecutor = InterruptExecutor::new();
static IE2: InterruptExecutor = InterruptExecutor::new();

const BLOCK_LENGTH: usize = 128;
const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2;

const QUARTER_DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH / 2;
const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2;
const SAMPLE_RATE: u32 = 48000;

#[link_section = ".sram1_bss"]
static mut TX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

// Probe-rs fails to flash the extflash if I try this :(
//#[used]
//#[unsafe(link_section = "._extflash")]
//static FLASH_DATA: [u8; 368542] = *include_bytes!("../assets/crab_rave.raw_s16le_pcm_48k");


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
        Timer::after_millis(1).await;
    }
}

#[embassy_executor::task]
async fn audio_tx_task(
) 
{
    let mut sai_lock = SAI.lock().await;
    //let mut rng_lock = RNG.lock().await;
    //if let Some(r) = rng_lock.as_mut() {
        if let Some(sai_transmitter) = sai_lock.as_mut() {
            let mut audio_data = unsafe { core::slice::from_raw_parts(0x90000000 as *const u8, 368542) };
            let mut audio_pos: usize = 0;
            let mut audio_buffer = [0u8; HALF_DMA_BUFFER_LENGTH*4];
            loop {
                audio_buffer.copy_from_slice(&audio_data[audio_pos..audio_pos+HALF_DMA_BUFFER_LENGTH*4]);
                let mut audio_data = unsafe { core::mem::transmute::<[u8; HALF_DMA_BUFFER_LENGTH*4], [u32; HALF_DMA_BUFFER_LENGTH]>(audio_buffer) };
                trace!("sample data: {}", audio_data[0..10]);
                let result = sai_transmitter.write(&audio_data).await;
                if let Err(e) = result {
                    error!("{}", e);
                }
                if audio_pos < (368542 - HALF_DMA_BUFFER_LENGTH*8) {
                    audio_pos += HALF_DMA_BUFFER_LENGTH*4;
                }
                else {
                    audio_pos = 0;
                }
            }
        }
    //}
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("GAME & WATCH TEST");

    // initialize clocks
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

    config.rcc.mux.persel = Persel::HSI;
    config.rcc.mux.octospisel = Fmcsel::PER;
    config.rcc.mux.sai1sel = Saisel::PLL2_P;
    config.rcc.mux.spi123sel = Saisel::PLL3_P;

    let cp = embassy_stm32::init(config);

    // initialize lcd pins + spi
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

    // initialize ltdc pins
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

    {
        *(LCD.lock().await) = Some(lcd);
    }


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

    {
        *(DISPLAY.lock().await) = Some(disp);
    }

    {
        *(LTDC_OBJ.lock().await) = Some(ltdc);
    }

    info!("Initialised Display...");

    //Timer::after_millis(200).await;

    // initialize buttons
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

    {
        *(BUTTONS.lock().await) = Some(buttons);
    }

    // Initialize static ferris struct
    {
        *(FERRIS.lock().await) = Some(Bmp::from_slice(include_bytes!("../assets/ferris.bmp")).unwrap());
    }

    // Initialize spi flash
    // FIXME is there a way to avoid calling PeripheralRef::new on all of these?
    let mut spiflash = SpiFlash::new(
        PeripheralRef::new(cp.PB2),
        PeripheralRef::new(cp.PB1),
        PeripheralRef::new(cp.PD12),
        PeripheralRef::new(cp.PE2),
        PeripheralRef::new(cp.PA1),
        PeripheralRef::new(cp.PE11),
        PeripheralRef::new(cp.OCTOSPI1)
    );
    spiflash.init().await;

    unsafe {
        debug!("First word of spiflash: {=u32:x}", core::ptr::read_volatile(0x90000000 as *const u32));
    }

    //debug!("SAI FREQ: {}", embassy_stm32::rcc::frequency::<peripherals::SAI1>());

    let audio_enable = Output::new(cp.PE3, Level::High, Speed::Low);

    let mut rng = Rng::new(cp.RNG, Irqs); 

    {
        *(RNG.lock().await) = Some(rng);
    }

    // Initialize state
    let mut gs = GameState::new();

    {
        *(GS.lock().await) = Some(gs);
    }

    // start polling for input asynchronously
    spawner.spawn(input_task()).unwrap();

    let kernel_clock = rcc::frequency::<peripherals::SAI1>().0;
    debug!("SAI clock: {}", kernel_clock);

    let mclk_div = sai::MasterClockDivider::Div2;

    let mut tx_config = sai::Config::default();
    tx_config.mode = sai::Mode::Master;
    tx_config.tx_rx = sai::TxRx::Transmitter;
    tx_config.sync_output = true;
    tx_config.clock_strobe = sai::ClockStrobe::Falling;
    tx_config.master_clock_divider = mclk_div;
    tx_config.stereo_mono = sai::StereoMono::Stereo;
    tx_config.data_size = sai::DataSize::Data24;
    tx_config.bit_order = sai::BitOrder::MsbFirst;
    tx_config.frame_sync_polarity = sai::FrameSyncPolarity::ActiveLow;
    tx_config.frame_sync_offset = sai::FrameSyncOffset::OnFirstBit;
    tx_config.frame_length = HALF_DMA_BUFFER_LENGTH as u8;
    tx_config.frame_sync_active_level_length = sai::word::U7(BLOCK_LENGTH as u8);
    tx_config.fifo_threshold = sai::FifoThreshold::Quarter;

    let tx_buffer: &mut [u32] = unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(TX_BUFFER);
        buf.initialize_all_copied(0);
        let (ptr, len) = buf.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let mut sai_transmitter = Sai::new_asynchronous(
        sai::split_subblocks(cp.SAI1).0,
        cp.PE5,
        cp.PE6,
        cp.PE4,
        cp.DMA1_CH0,
        tx_buffer,
        tx_config
    );

    sai_transmitter.set_mute(false);

    { 
    *(SAI.lock().await) = Some(sai_transmitter);
    }

    pac::interrupt::UART7.set_priority(Priority::P0);
    pac::interrupt::UART8.set_priority(Priority::P7);

   let sai_spawner = IE1.start(pac::interrupt::UART7);
   let ltdc_spawner = IE2.start(pac::interrupt::UART8);

   sai_spawner.spawn(audio_tx_task()).unwrap();
   ltdc_spawner.spawn(display_task()).unwrap();

   loop {
       Timer::after_secs(1000).await;
   }
}

#[embassy_executor::task]
async fn display_task() {
    let mut ltdc_lock = LTDC_OBJ.lock().await;
    let mut disp_lock = DISPLAY.lock().await;
    let mut gs_lock = GS.lock().await;
    let mut lcd_lock = LCD.lock().await;
    if let Some(ltdc) = ltdc_lock.as_mut() {
        if let Some(disp) = disp_lock.as_mut() {
            if let Some(gs) = gs_lock.as_mut() {
                if let Some(lcd) = lcd_lock.as_mut() {

                    loop {
                        update(gs, lcd).await; 
                        draw(gs,  disp).await;
                        disp.swap(ltdc).await.unwrap();
                        //Timer::after_millis(8).await;
                    }
                }
            }
        }
    }
}

#[cortex_m_rt::interrupt]
unsafe fn UART7() {
    IE1.on_interrupt()
}


#[cortex_m_rt::interrupt]
unsafe fn UART8() {
    IE2.on_interrupt()
}

const fn mclk_div_from_u8(v: u8) -> MasterClockDivider {
    match v {
        1 => MasterClockDivider::Div1,
        2 => MasterClockDivider::Div2,
        3 => MasterClockDivider::Div3,
        4 => MasterClockDivider::Div4,
        5 => MasterClockDivider::Div5,
        6 => MasterClockDivider::Div6,
        7 => MasterClockDivider::Div7,
        8 => MasterClockDivider::Div8,
        9 => MasterClockDivider::Div9,
        10 => MasterClockDivider::Div10,
        11 => MasterClockDivider::Div11,
        12 => MasterClockDivider::Div12,
        13 => MasterClockDivider::Div13,
        14 => MasterClockDivider::Div14,
        15 => MasterClockDivider::Div15,
        16 => MasterClockDivider::Div16,
        17 => MasterClockDivider::Div17,
        18 => MasterClockDivider::Div18,
        19 => MasterClockDivider::Div19,
        20 => MasterClockDivider::Div20,
        21 => MasterClockDivider::Div21,
        22 => MasterClockDivider::Div22,
        23 => MasterClockDivider::Div23,
        24 => MasterClockDivider::Div24,
        25 => MasterClockDivider::Div25,
        26 => MasterClockDivider::Div26,
        27 => MasterClockDivider::Div27,
        28 => MasterClockDivider::Div28,
        29 => MasterClockDivider::Div29,
        30 => MasterClockDivider::Div30,
        31 => MasterClockDivider::Div31,
        32 => MasterClockDivider::Div32,
        33 => MasterClockDivider::Div33,
        34 => MasterClockDivider::Div34,
        35 => MasterClockDivider::Div35,
        36 => MasterClockDivider::Div36,
        37 => MasterClockDivider::Div37,
        38 => MasterClockDivider::Div38,
        39 => MasterClockDivider::Div39,
        40 => MasterClockDivider::Div40,
        41 => MasterClockDivider::Div41,
        42 => MasterClockDivider::Div42,
        43 => MasterClockDivider::Div43,
        44 => MasterClockDivider::Div44,
        45 => MasterClockDivider::Div45,
        46 => MasterClockDivider::Div46,
        47 => MasterClockDivider::Div47,
        48 => MasterClockDivider::Div48,
        49 => MasterClockDivider::Div49,
        50 => MasterClockDivider::Div50,
        51 => MasterClockDivider::Div51,
        52 => MasterClockDivider::Div52,
        53 => MasterClockDivider::Div53,
        54 => MasterClockDivider::Div54,
        55 => MasterClockDivider::Div55,
        56 => MasterClockDivider::Div56,
        57 => MasterClockDivider::Div57,
        58 => MasterClockDivider::Div58,
        59 => MasterClockDivider::Div59,
        60 => MasterClockDivider::Div60,
        61 => MasterClockDivider::Div61,
        62 => MasterClockDivider::Div62,
        63 => MasterClockDivider::Div63,
        _ => panic!(),
    }
}
