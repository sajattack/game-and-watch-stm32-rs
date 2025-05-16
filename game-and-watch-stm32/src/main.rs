#![no_main]
#![no_std]

macro_rules! pins_alternate_high_speed {
    ($($func:ident:  $port:ident.$pin:ident:  $af:expr);*) => {
        (
            $(
                $port.$pin.into_alternate::<$af>()
                    .speed(Speed::High)
                    .internal_pull_up(true)
            ),*
        )
    };
}


mod lcd;
mod input;
mod spiflash;

#[macro_use]
mod utilities_display;

use panic_probe as _;
use core::mem::MaybeUninit;

const AUDIO_BUFFER_SIZE: usize = 256;
const AUDIO_DIVIDER: usize = 24;
const VIDEO_BUFFER_SIZE: usize = lcd::WIDTH * lcd::HEIGHT;

#[link_section = ".sram3"]
static mut FRONT_BUFFER: MaybeUninit<[u16; VIDEO_BUFFER_SIZE]> = MaybeUninit::uninit();
#[link_section = ".sram3"]
static mut BACK_BUFFER: MaybeUninit<[u16; VIDEO_BUFFER_SIZE]> = MaybeUninit::uninit();


//#[link_section = ".sram3"]
static AUDIO_BUFFER: [u16; AUDIO_BUFFER_SIZE] = [
    0x8000, 0x8324, 0x8647, 0x896a, 0x8c8b, 0x8fab, 0x92c7, 0x95e1, 0x98f8, 0x9c0b, 0x9f19, 0xa223,
0xa527, 0xa826, 0xab1f, 0xae10, 0xb0fb, 0xb3de, 0xb6b9, 0xb98c, 0xbc56, 0xbf17, 0xc1cd, 0xc47a,
0xc71c, 0xc9b3, 0xcc3f, 0xcebf, 0xd133, 0xd39a, 0xd5f5, 0xd842, 0xda82, 0xdcb3, 0xded7, 0xe0eb,
0xe2f1, 0xe4e8, 0xe6cf, 0xe8a6, 0xea6d, 0xec23, 0xedc9, 0xef5e, 0xf0e2, 0xf254, 0xf3b5, 0xf504,
0xf641, 0xf76b, 0xf884, 0xf989, 0xfa7c, 0xfb5c, 0xfc29, 0xfce3, 0xfd89, 0xfe1d, 0xfe9c, 0xff09,
0xff61, 0xffa6, 0xffd8, 0xfff5, 0xffff, 0xfff5, 0xffd8, 0xffa6, 0xff61, 0xff09, 0xfe9c, 0xfe1d,
0xfd89, 0xfce3, 0xfc29, 0xfb5c, 0xfa7c, 0xf989, 0xf884, 0xf76b, 0xf641, 0xf504, 0xf3b5, 0xf254,
0xf0e2, 0xef5e, 0xedc9, 0xec23, 0xea6d, 0xe8a6, 0xe6cf, 0xe4e8, 0xe2f1, 0xe0eb, 0xded7, 0xdcb3,
0xda82, 0xd842, 0xd5f5, 0xd39a, 0xd133, 0xcebf, 0xcc3f, 0xc9b3, 0xc71c, 0xc47a, 0xc1cd, 0xbf17,
0xbc56, 0xb98c, 0xb6b9, 0xb3de, 0xb0fb, 0xae10, 0xab1f, 0xa826, 0xa527, 0xa223, 0x9f19, 0x9c0b,
0x98f8, 0x95e1, 0x92c7, 0x8fab, 0x8c8b, 0x896a, 0x8647, 0x8324, 0x8000, 0x7cdb, 0x79b8, 0x7695,
0x7374, 0x7054, 0x6d38, 0x6a1e, 0x6707, 0x63f4, 0x60e6, 0x5ddc, 0x5ad8, 0x57d9, 0x54e0, 0x51ef,
0x4f04, 0x4c21, 0x4946, 0x4673, 0x43a9, 0x40e8, 0x3e32, 0x3b85, 0x38e3, 0x364c, 0x33c0, 0x3140,
0x2ecc, 0x2c65, 0x2a0a, 0x27bd, 0x257d, 0x234c, 0x2128, 0x1f14, 0x1d0e, 0x1b17, 0x1930, 0x1759,
0x1592, 0x13dc, 0x1236, 0x10a1, 0xf1d, 0xdab, 0xc4a, 0xafb, 0x9be, 0x894, 0x77b, 0x676,
0x583, 0x4a3, 0x3d6, 0x31c, 0x276, 0x1e2, 0x163, 0xf6, 0x9e, 0x59, 0x27, 0x0a,
0x00, 0x0a, 0x27, 0x59, 0x9e, 0xf6, 0x163, 0x1e2, 0x276, 0x31c, 0x3d6, 0x4a3,
0x583, 0x676, 0x77b, 0x894, 0x9be, 0xafb, 0xc4a, 0xdab, 0xf1d, 0x10a1, 0x1236, 0x13dc,
0x1592, 0x1759, 0x1930, 0x1b17, 0x1d0e, 0x1f14, 0x2128, 0x234c, 0x257d, 0x27bd, 0x2a0a, 0x2c65,
0x2ecc, 0x3140, 0x33c0, 0x364c, 0x38e3, 0x3b85, 0x3e32, 0x40e8, 0x43a9, 0x4673, 0x4946, 0x4c21,
0x4f04, 0x51ef, 0x54e0, 0x57d9, 0x5ad8, 0x5ddc, 0x60e6, 0x63f4, 0x6707, 0x6a1e, 0x6d38, 0x7054,
0x7374, 0x7695, 0x79b8, 0x7cdb
];


#[rtic::app( device = stm32h7xx_hal::stm32, peripherals = true )]
mod app {
    use super::{AUDIO_BUFFER_SIZE, AUDIO_DIVIDER, VIDEO_BUFFER_SIZE};
    use crate::{FRONT_BUFFER, BACK_BUFFER, AUDIO_BUFFER};
    use core::mem::MaybeUninit;
    use ltdc::Ltdc;
    use stm32h7xx_hal::{dma::{mdma::StreamX, MasterTransfer, PeripheralToMemory, Transfer}, gpio::{Alternate, Pin, PinState, Speed}, ltdc::{self, LtdcLayer1}, pac::{self, rcc::cdccipr::FMCSEL_A, OCTOSPI1, SAI1}, prelude::*, rcc::rec::{Mdma, Octospi1, Sai1ClkSel, Spi123ClkSel}, sai::{
            self, I2SChanConfig, I2SDataSize, I2SDir, I2SSync, I2sUsers, Sai,
            SaiChannel, SaiI2sExt, I2S,
        }, spi::{self, Spi}, stm32::Interrupt, time::Hertz, timer::{Event, Timer}, traits::i2s::FullDuplex, xspi::{Octospi, OctospiWord}
    };
    use embedded_display_controller::{DisplayController, DisplayControllerLayer, DisplayConfiguration};

    use embedded_graphics::{image::Image, primitives::Rectangle, pixelcolor::Rgb565};
    use embedded_graphics::mono_font::{ascii, MonoTextStyle};
    use embedded_graphics::prelude::*;
    use embedded_graphics::text::Text;

    use fugit::Duration;

    use rtic_monotonics::systick::prelude::*;

    use tinybmp::Bmp;

    use defmt::{info, error, debug, trace};
    use defmt_rtt as _;

    use crate::utilities_display::write::write_to::WriteTo;
    use crate::utilities_display::display_target::BufferedDisplay;

    use crate::lcd::{self, *};
    use crate::spiflash::{self, *};
    use crate::input::{self, *};

    #[shared]
    struct SharedResources {
    }
    #[local]
    struct LocalResources {
        //audio_buffer: [u16; AUDIO_BUFFER_SIZE],
        audio: Sai<SAI1, I2S>,
        audio_pos: usize,
        audio_divider: usize,
        //transfer: Transfer<StreamX<pac::MDMA, 0>, Octospi<OCTOSPI1>, PeripheralToMemory, &'static mut [u32; crate::AUDIO_BUFFER_SIZE], MasterTransfer>,
        //spiflash_pos: usize,
        //display: BufferedDisplay<'static, LtdcLayer1>,
        //timer: Timer<stm32h7xx_hal::stm32::TIM2>,
        //ferris_pos: Point,
        //buttons: Buttons,
        //lcd: Lcd,
    }

    const AUDIO_SAMPLE_HZ: Hertz = Hertz::from_raw(48000);
    const PLL3_P_HZ: Hertz = Hertz::from_raw(AUDIO_SAMPLE_HZ.raw() * 257);
    //const AUDIO_SIZE: usize = 368542;

    #[init]
    fn init(mut ctx: init::Context) -> (SharedResources, LocalResources) {
        info!("");
        info!("GAME & WATCH TEST");
        info!("");


        // Constrain and Freeze power
        let pwr = ctx.device.PWR.constrain();
        let pwrcfg = pwr.ldo().vos0(&ctx.device.SYSCFG).freeze();

        // Constrain and Freeze clock

        let rcc = ctx.device.RCC.constrain();
        
        let mut ccdr = rcc.sys_ck(280.MHz())
            .pll2_p_ck(18.MHz())
            .pll2_q_ck(144.MHz())
            .pll2_r_ck(6.MHz())


            .pll3_p_ck(PLL3_P_HZ)
            .pll3_q_ck(150.MHz())
            .pll3_r_ck(24.MHz())
            .per_ck(64.MHz())

            .freeze(pwrcfg, &ctx.device.SYSCFG);

        ccdr.peripheral.kernel_octospi_clk_mux(FMCSEL_A::Per);
        ccdr.peripheral.kernel_spi123_clk_mux(Spi123ClkSel::Pll2P);

        let mut delay = stm32h7xx_hal::delay::Delay::new(ctx.core.SYST, ccdr.clocks);

        let gpioa = ctx.device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = ctx.device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = ctx.device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = ctx.device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = ctx.device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = ctx.device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = ctx.device.GPIOG.split(ccdr.peripheral.GPIOG);
        let gpioh = ctx.device.GPIOH.split(ccdr.peripheral.GPIOH);

        //let lcd_pins = pins_alternate_high_speed! {
            //clk:   gpiob.pb14: 14;
            //vsync: gpioa.pa7: 14;
            //hsync: gpioc.pc6: 14;
            //de: gpioe.pe13: 14;
            //r7: gpioe.pe15: 14;
            //r6: gpioa.pa8: 14;
            //r5: gpioa.pa9: 14;
            //r4: gpioa.pa11: 14;
            //r3: gpiob.pb0: 14;
            //r2: gpioc.pc10: 14;
            //g7: gpiod.pd3: 14;
            //g6: gpioc.pc7: 14;
            //g5: gpiob.pb11: 14;
            //g4: gpiob.pb10: 14;
            //g3: gpioc.pc9: 14;
            //g2: gpioc.pc0: 14;
            //b7: gpiod.pd2: 14;
            //b6: gpiob.pb8: 14;
            //b5: gpiob.pb5: 14;
            //b4: gpioa.pa10: 14;
            //b3: gpiod.pd10: 14;
            //b2: gpiod.pd6: 14
        //};

        //let left = gpiod.pd11;
        //let right = gpiod.pd15;
        //let up = gpiod.pd0;
        //let down = gpiod.pd14;
        //let a = gpiod.pd9;
        //let b = gpiod.pd5;
        //let game = gpioc.pc1;
        //let time = gpioc.pc4;
        //let pause = gpioc.pc13;
        //let power = gpioa.pa0;

        //let buttons: input::Buttons = input::ButtonPins::new(
            //left.into(),
            //right.into(),
            //up.into(),
            //down.into(), 
            //a.into(),
            //b.into(),
            //game.into(),
            //time.into(),
            //pause.into(),
            //power.into()
        //).into();

        //let sck = gpiob.pb13.into_alternate();
        //let mosi = gpiob.pb15.into_alternate();
        //let cs = gpiob.pb12.into_push_pull_output();

        //let pa4 = gpioa.pa4.into_push_pull_output();
        //let pa5 = gpioa.pa5.into_push_pull_output();
        //let pa6 = gpioa.pa6.into_push_pull_output();

        //let mut disable_3v3 = gpiod.pd1.into_push_pull_output();
        //let mut enable_1v8  = gpiod.pd4.into_push_pull_output();
        //let reset = gpiod.pd8.into_push_pull_output();

        //let spi = ctx.device.SPI2.spi((sck, spi::NoMiso, mosi), spi::MODE_0, 18.MHz(), ccdr.peripheral.SPI2, &ccdr.clocks);

        //debug!("SPI clock {}", Spi::<stm32h7xx_hal::stm32::SPI2, spi::Enabled, u8>::kernel_clk(&ccdr.clocks).unwrap().raw());

        //let mut ltdc = ltdc::Ltdc::new(ctx.device.LTDC, ccdr.peripheral.LTDC, &ccdr.clocks);
        //ltdc.init(
            //DisplayConfiguration {
                //active_width: WIDTH as u16,
                //active_height: HEIGHT as u16,
                //h_back_porch: 200,
                //h_front_porch: 431,
                //v_back_porch: 21,
                //v_front_porch: 1,
                //h_sync: 10,
                //v_sync: 2,
                //h_sync_pol: false,
                //v_sync_pol: false,
                //not_data_enable_pol: false,
                //pixel_clock_pol: true,
            //}
        //);

        //ltdc.inner_mut().srcr.modify(|_, w| w.vbr().set_bit());

        ////ltdc.listen();

        //info!("LCD Clock: {}", ltdc.clock());

        //let mut layer = ltdc.split();

        //let mut lcd = Lcd::new(pa4, pa5, pa6, disable_3v3, enable_1v8, reset, cs, spi);
        //lcd.init(&mut delay).unwrap();

        //#[allow(static_mut_refs)]
        //let front_buffer: &'static mut [u16; VIDEO_BUFFER_SIZE] = unsafe { FRONT_BUFFER.assume_init_mut() };
        //let back_buffer: &'static mut [u16; VIDEO_BUFFER_SIZE] = unsafe { BACK_BUFFER.assume_init_mut() };

        //let mut disp = BufferedDisplay::new(layer, front_buffer, back_buffer, WIDTH, HEIGHT);

        //info!("Initialised Display...");

        //let mut timer = ctx.device.TIM2.timer(Hertz::from_duration(Duration::<u32, 1, 1000>::millis(2)[>input::TIMER_PERIOD.into()<]), ccdr.peripheral.TIM2, &ccdr.clocks);
        //// Generate an interrupt when the timer expires
        //timer.listen(Event::TimeOut);

        //let spiflash = SpiFlash::new(
            //gpiob.pb2.into(),
            //gpiob.pb1.into(),
            //gpiod.pd12.into(),
            //gpioe.pe2.into(),
            //gpioa.pa1.into(),
            //gpioe.pe11.into(),
            //ctx.device.OCTOSPI1,
            //&ccdr.clocks,
            //ccdr.peripheral.OCTOSPI1,
            //&mut delay,
            //ctx.device.MDMA,
            //ccdr.peripheral.MDMA,
        //);

        //let mut spiflash_pos: usize = 0;

        //let mut transfer = spiflash.begin_transfer(unsafe { &mut AUDIO_BUFFER }, &mut spiflash_pos);
        //transfer.start(|_|{});

        let mut audio_enable = gpioe.pe3.into_push_pull_output_in_state(PinState::High);

        // Use PLL3_P for the SAI1 clock
        let sai1_rec = ccdr.peripheral.SAI1.kernel_clk_mux(Sai1ClkSel::Pll3P);
        let master_config =
            I2SChanConfig::new(I2SDir::Tx)
            //.set_frame_sync_active_high(true)
            .set_mono_mode(true);

        let slave_config = I2SChanConfig::new(I2SDir::Rx)
            .set_sync_type(I2SSync::Internal)
            //.set_frame_sync_active_high(true)
            .set_mono_mode(true);

        let sai1_pins = (
            // pg7 doesn't exist afaik but the hal needs something here
            gpiog.pg7.into_alternate(),
            gpioe.pe5.into_alternate(),
            gpioe.pe4.into_alternate(),
            gpioe.pe6.into_alternate(),
            None::<Pin<'E', 3, Alternate<6>>>
        );

        let mut audio = ctx.device.SAI1.i2s_ch_a(
            sai1_pins,
            AUDIO_SAMPLE_HZ,
            I2SDataSize::BITS_16,
            sai1_rec,
            &ccdr.clocks,
            I2sUsers::new(master_config).add_slave(slave_config),
        );

        // Setup cache
        // Sound breaks up without this enabled
        ctx.core.SCB.enable_icache();

        audio.listen(SaiChannel::ChannelA, sai::Event::Data);
        audio.enable();
        nb::block!(audio.try_send(0, 0)).unwrap();

        let ferris_pos = Point::new(120, 125);
        
        info!("Startup complete!");
        (
            SharedResources {
                //audio_buffer: unsafe { AUDIO_BUFFER },
            },
            LocalResources {
                //audio_buffer: unsafe { AUDIO_BUFFER },
                audio,
                audio_pos: 0, 
                audio_divider: AUDIO_DIVIDER,
                //transfer,
                //spiflash_pos,
                //display: disp,
                //timer,
                //ferris_pos,
                //buttons,
                //lcd,
            },
        )
    }

    #[task(priority=16, binds=SAI1, shared=[], local=[audio, audio_pos, audio_divider])]
    fn audio_tx(mut ctx: audio_tx::Context) {
            let value = unsafe { AUDIO_BUFFER.get(*ctx.local.audio_pos) };
                
            if (*ctx.local.audio_pos) < (AUDIO_BUFFER_SIZE - 1)
            {
                //let newval = (ctx.local.audio_divider).checked_sub(1);
                //if newval.is_none()
                //{
                    *ctx.local.audio_pos += 20;
                    //*ctx.local.audio_divider = AUDIO_DIVIDER;
                //}
                //else
                //{
                    //(*ctx.local.audio_divider) = newval.unwrap();
                //}

                // we set the i2s data size as 16bit but the dumb library still wants a u32
                if value.is_some() {
                    nb::block!(ctx.local.audio.try_send(0, *value.unwrap() as u32));
                }
                //trace!("audio pos: {} audio_val: 0x{:04x}", ctx.local.audio_pos, value);

            }
            else
            {
                *ctx.local.audio_pos = 0;
                //*ctx.local.audio_divider = AUDIO_DIVIDER;
            }

    }

    //#[task(priority=15, binds = LTDC, local = [display, ferris_pos, lcd, buttons])]
    //fn draw(mut ctx: draw::Context) {
        //trace!("FRAME");

        //update(ctx.local.ferris_pos, ctx.local.buttons, ctx.local.lcd);
        //ctx.local.display.layer(|draw| {
            //draw.clear();
            //draw.fill_solid(&Rectangle::new(Point::new(0, 0), Size::new(320, 240)), RgbColor::RED).unwrap();

            //let text_style =
                //MonoTextStyle::new(&ascii::FONT_9X18, RgbColor::WHITE);
            //Text::new("Hello Rust!", Point::new(120, 100), text_style)
                //.draw(draw)
                //.unwrap();

            //let ferris: Bmp<Rgb565> =
                //Bmp::from_slice(include_bytes!("../assets/ferris.bmp")).unwrap();
            //let ferris = Image::new(&ferris, *ctx.local.ferris_pos);
            //ferris.draw(draw).unwrap();
        //});

        //if ctx.local.display.is_swap_pending() {
            //unsafe { ctx.local.display.swap_layer(); } 
        //}
    //}

    //// This is gross and needs a refactor
    //#[task(binds = TIM2, local=[timer])]
    //fn timer(mut ctx: timer::Context) {    
        //unsafe { input::GLOBAL_TIMER_COUNTER += input::TIMER_PERIOD }
        //ctx.local.timer.clear_irq();
    //}

    //#[task(binds = MDMA,  local=[transfer, spiflash_pos])]
    //fn spiflash_rx_complete(mut ctx: spiflash_rx_complete::Context) {
        //ctx.local.transfer.pause(|p| {
            //if *ctx.local.spiflash_pos >= [>0x9000_0000 +<] AUDIO_SIZE -1
            //{
                //*ctx.local.spiflash_pos = 0;//0x9000_0000
            //}
            //else 
            //{
                //*ctx.local.spiflash_pos += AUDIO_BUFFER_SIZE * 4;
            //}
            //p.begin_read_extended(OctospiWord::U8(spiflash::FlashCommand::CMD_4READ as u8), OctospiWord::U24(*ctx.local.spiflash_pos as u32), OctospiWord::None, 6, crate::AUDIO_BUFFER_SIZE*4).unwrap();
        //});
        //ctx.local.transfer.clear_transfer_complete_interrupt();
        //ctx.local.transfer.start(|_|{});
    //}

    //#[idle]
    //fn idle(cx: idle::Context) -> ! {
        //loop {
            //cortex_m::asm::wfi();
        //}
    //}

    
    //fn update(ferris_pos: &mut Point, buttons: &mut Buttons, lcd: &mut Lcd) {
        //buttons.tick_all();

        //let button_reading = buttons.raw_read_all();
        //let button_clicks = buttons.read_clicks();

        //if button_reading.left.is_held() {
            //ferris_pos.x -= 1;
        //}
        //if button_reading.right.is_held() {
            //ferris_pos.x += 1;
        //}
        //if button_reading.up.is_held() {
            //ferris_pos.y -= 1;
        //}
        //if button_reading.down.is_held() {
            //ferris_pos.y +=1;
        //}

        //if button_clicks.power {
            //lcd.toggle_backlight();
        //}

        ////buttons.reset_all();
    //}
}
