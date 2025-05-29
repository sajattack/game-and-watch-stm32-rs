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

use stm32h7xx_hal::{dma::{self, mdma::StreamX, MasterTransfer, PeripheralToMemory, Transfer}, pac::{MDMA, OCTOSPI1},
xspi::Octospi,
sai, stm32};

const AUDIO_BUFFER_SIZE: usize = 192;
const VIDEO_BUFFER_SIZE: usize = lcd::WIDTH * lcd::HEIGHT;

#[link_section = ".sram3"]
static mut FRONT_BUFFER: MaybeUninit<[u16; VIDEO_BUFFER_SIZE]> = MaybeUninit::uninit();
#[link_section = ".sram3"]
static mut BACK_BUFFER: MaybeUninit<[u16; VIDEO_BUFFER_SIZE]> = MaybeUninit::uninit();
#[link_section = ".sram3"]
static mut AUDIO_BUFFER: MaybeUninit<[u32; AUDIO_BUFFER_SIZE]> = MaybeUninit::uninit();

type TransferDma1Str1 = dma::Transfer<
    dma::dma::Stream1<stm32::DMA1>,
    sai::dma::ChannelA<stm32::SAI1>,
    dma::MemoryToPeripheral,
    &'static mut [u32; AUDIO_BUFFER_SIZE],
    dma::DBTransfer,
>;

static mut TRANSFER_DMA1_STR1: MaybeUninit<Option<TransferDma1Str1>> = MaybeUninit::uninit();

static mut TRANSFER_SPIFLASH: MaybeUninit<Option<Transfer<StreamX<MDMA, 0>, Octospi<OCTOSPI1>, PeripheralToMemory, &'static mut MaybeUninit<[u32; crate::AUDIO_BUFFER_SIZE]>, MasterTransfer>>> = MaybeUninit::uninit();

//#[link_section = ".sram3"]
static SINE_WAVE: [u32; AUDIO_BUFFER_SIZE] = [
    0x8000, 0x8430, 0x885f, 0x8c8b, 0x90b5, 0x94d9, 0x98f8, 0x9d10,
    0xa120, 0xa527, 0xa924, 0xad16, 0xb0fb, 0xb4d3, 0xb89c, 0xbc56,
    0xbfff, 0xc397, 0xc71c, 0xca8e, 0xcdeb, 0xd133, 0xd465, 0xd77f,
    0xda82, 0xdd6b, 0xe03b, 0xe2f1, 0xe58c, 0xe80a, 0xea6d, 0xecb2,
    0xeed9, 0xf0e2, 0xf2cc, 0xf496, 0xf641, 0xf7cb, 0xf934, 0xfa7c,
    0xfba2, 0xfca7, 0xfd89, 0xfe49, 0xfee7, 0xff61, 0xffb9, 0xffed,
    0xffff, 0xffed, 0xffb9, 0xff61, 0xfee7, 0xfe49, 0xfd89, 0xfca7,
    0xfba2, 0xfa7c, 0xf934, 0xf7cb, 0xf641, 0xf496, 0xf2cc, 0xf0e2,
    0xeed9, 0xecb2, 0xea6d, 0xe80a, 0xe58c, 0xe2f1, 0xe03b, 0xdd6b,
    0xda82, 0xd77f, 0xd465, 0xd133, 0xcdeb, 0xca8e, 0xc71c, 0xc397,
    0xbfff, 0xbc56, 0xb89c, 0xb4d3, 0xb0fb, 0xad16, 0xa924, 0xa527,
    0xa120, 0x9d10, 0x98f8, 0x94d9, 0x90b5, 0x8c8b, 0x885f, 0x8430,
    0x8000, 0x7bcf, 0x77a0, 0x7374, 0x6f4a, 0x6b26, 0x6707, 0x62ef,
    0x5edf, 0x5ad8, 0x56db, 0x52e9, 0x4f04, 0x4b2c, 0x4763, 0x43a9,
    0x4000, 0x3c68, 0x38e3, 0x3571, 0x3214, 0x2ecc, 0x2b9a, 0x2880,
    0x257d, 0x2294, 0x1fc4, 0x1d0e, 0x1a73, 0x17f5, 0x1592, 0x134d,
    0x1126, 0xf1d, 0xd33, 0xb69, 0x9be, 0x834, 0x6cb, 0x583,
    0x45d, 0x358, 0x276, 0x1b6, 0x118, 0x9e, 0x46, 0x12,
    0x00, 0x12, 0x46, 0x9e, 0x118, 0x1b6, 0x276, 0x358,
    0x45d, 0x583, 0x6cb, 0x834, 0x9be, 0xb69, 0xd33, 0xf1d,
    0x1126, 0x134d, 0x1592, 0x17f5, 0x1a73, 0x1d0e, 0x1fc4, 0x2294,
    0x257d, 0x2880, 0x2b9a, 0x2ecc, 0x3214, 0x3571, 0x38e3, 0x3c68,
    0x4000, 0x43a9, 0x4763, 0x4b2c, 0x4f04, 0x52e9, 0x56db, 0x5ad8,
    0x5edf, 0x62ef, 0x6707, 0x6b26, 0x6f4a, 0x7374, 0x77a0, 0x7bcf
];



#[rtic::app(device=stm32h7xx_hal::stm32, peripherals=true)]
mod app {
    use super::{AUDIO_BUFFER_SIZE, VIDEO_BUFFER_SIZE};
    use crate::{FRONT_BUFFER, BACK_BUFFER, AUDIO_BUFFER, SINE_WAVE, TRANSFER_DMA1_STR1, TRANSFER_SPIFLASH};
    use core::mem::MaybeUninit;
    use ltdc::Ltdc;
    use stm32h7xx_hal::{dma::{mdma::StreamX, MasterTransfer, PeripheralToMemory, Transfer, self}, gpio::{Alternate, Pin, PinState, Speed}, ltdc::{self, LtdcLayer1}, pac::{self, rcc::cdccipr::FMCSEL_A, OCTOSPI1, SAI1}, prelude::*, rcc::rec::{Mdma, Octospi1, Sai1ClkSel, Spi123ClkSel}, sai::{
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
        sai1: Sai<SAI1, I2S>,
        audio_pos: usize,
        spiflash_pos: usize,
        display: BufferedDisplay<'static, LtdcLayer1>,
        timer: Timer<stm32h7xx_hal::stm32::TIM2>,
        ferris_pos: Point,
        ferris: Bmp<'static, Rgb565>,
        text_style: MonoTextStyle<'static, Rgb565>,
        buttons: Buttons,
        lcd: Lcd,
    }

    const AUDIO_SAMPLE_HZ: Hertz = Hertz::from_raw(48_000);
    const PLL3_P_HZ: Hertz = Hertz::from_raw(AUDIO_SAMPLE_HZ.raw() * 257);
    const AUDIO_SIZE: usize = 368542;

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
            .pll2_q_ck(160.MHz())
            .pll2_r_ck(6.MHz())


            .pll3_p_ck(PLL3_P_HZ)
            .pll3_q_ck(150.MHz())
            .pll3_r_ck(28.MHz())
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

        let lcd_pins = pins_alternate_high_speed! {
            clk:   gpiob.pb14: 14;
            vsync: gpioa.pa7: 14;
            hsync: gpioc.pc6: 14;
            de: gpioe.pe13: 14;
            r7: gpioe.pe15: 14;
            r6: gpioa.pa8: 14;
            r5: gpioa.pa9: 14;
            r4: gpioa.pa11: 14;
            r3: gpiob.pb0: 14;
            r2: gpioc.pc10: 14;
            g7: gpiod.pd3: 14;
            g6: gpioc.pc7: 14;
            g5: gpiob.pb11: 14;
            g4: gpiob.pb10: 14;
            g3: gpioc.pc9: 14;
            g2: gpioc.pc0: 14;
            b7: gpiod.pd2: 14;
            b6: gpiob.pb8: 14;
            b5: gpiob.pb5: 14;
            b4: gpioa.pa10: 14;
            b3: gpiod.pd10: 14;
            b2: gpiod.pd6: 14
        };

        let left = gpiod.pd11;
        let right = gpiod.pd15;
        let up = gpiod.pd0;
        let down = gpiod.pd14;
        let a = gpiod.pd9;
        let b = gpiod.pd5;
        let game = gpioc.pc1;
        let time = gpioc.pc4;
        let pause = gpioc.pc13;
        let power = gpioa.pa0;

        let buttons: input::Buttons = input::ButtonPins::new(
            left.into(),
            right.into(),
            up.into(),
            down.into(), 
            a.into(),
            b.into(),
            game.into(),
            time.into(),
            pause.into(),
            power.into()
        ).into();

        let sck = gpiob.pb13.into_alternate();
        let mosi = gpiob.pb15.into_alternate();
        let cs = gpiob.pb12.into_push_pull_output();

        let pa4 = gpioa.pa4.into_push_pull_output();
        let pa5 = gpioa.pa5.into_push_pull_output();
        let pa6 = gpioa.pa6.into_push_pull_output();

        let mut disable_3v3 = gpiod.pd1.into_push_pull_output();
        let mut enable_1v8  = gpiod.pd4.into_push_pull_output();
        let reset = gpiod.pd8.into_push_pull_output();

        let spi = ctx.device.SPI2.spi((sck, spi::NoMiso, mosi), spi::MODE_0, 18.MHz(), ccdr.peripheral.SPI2, &ccdr.clocks);

        debug!("SPI clock {}", Spi::<stm32h7xx_hal::stm32::SPI2, spi::Enabled, u8>::kernel_clk(&ccdr.clocks).unwrap().raw());

        let mut ltdc = ltdc::Ltdc::new(ctx.device.LTDC, ccdr.peripheral.LTDC, &ccdr.clocks);
        ltdc.init(
            DisplayConfiguration {
                active_width: WIDTH as u16,
                active_height: HEIGHT as u16,
                h_back_porch: 200,
                h_front_porch: 431,
                v_back_porch: 21,
                v_front_porch: 1,
                h_sync: 10,
                v_sync: 2,
                h_sync_pol: false,
                v_sync_pol: false,
                not_data_enable_pol: false,
                pixel_clock_pol: true,
            }
        );

        ltdc.inner_mut().bccr.write(|w| w.bcred().bits(255));
        ltdc.inner_mut().srcr.modify(|_, w| w.vbr().set_bit());
        ltdc.inner_mut().ier.write(|w| w.rrie().set_bit() );

        ltdc.listen();

        info!("LCD Clock: {}", ltdc.clock());

        let mut layer = ltdc.split();

        let mut lcd = Lcd::new(pa4, pa5, pa6, disable_3v3, enable_1v8, reset, cs, spi);
        lcd.init(&mut delay).unwrap();

        #[allow(static_mut_refs)]
        let front_buffer: &'static mut [u16; VIDEO_BUFFER_SIZE] = unsafe { FRONT_BUFFER.assume_init_mut() };
        let back_buffer: &'static mut [u16; VIDEO_BUFFER_SIZE] = unsafe { BACK_BUFFER.assume_init_mut() };

        let mut disp = BufferedDisplay::new(layer, front_buffer, back_buffer, WIDTH, HEIGHT);

        info!("Initialised Display...");

        let ferris_pos = Point::new(120, 125);
        let ferris: Bmp<Rgb565> =
            Bmp::from_slice(include_bytes!("../assets/ferris.bmp")).unwrap();


        let text_style =
            MonoTextStyle::new(&ascii::FONT_9X18, RgbColor::WHITE);

        let mut timer = ctx.device.TIM2.timer(Hertz::from_duration(Duration::<u32, 1, 1000>::millis(2)/*input::TIMER_PERIOD.into()*/), ccdr.peripheral.TIM2, &ccdr.clocks);
        // Generate an interrupt when the timer expires
        timer.listen(Event::TimeOut);

        let spiflash = SpiFlash::new(
            gpiob.pb2.into(),
            gpiob.pb1.into(),
            gpiod.pd12.into(),
            gpioe.pe2.into(),
            gpioa.pa1.into(),
            gpioe.pe11.into(),
            ctx.device.OCTOSPI1,
            &ccdr.clocks,
            ccdr.peripheral.OCTOSPI1,
            &mut delay,
            ctx.device.MDMA,
            ccdr.peripheral.MDMA,
        );

        let mut spiflash_pos: usize = 0x0;

        unsafe { TRANSFER_SPIFLASH.write(Some(spiflash.begin_transfer(&mut AUDIO_BUFFER, &mut spiflash_pos))) };

        // configure dma1
        let dma1_streams = dma::dma::StreamsTuple::new(ctx.device.DMA1, ccdr.peripheral.DMA1);

        #[allow(static_mut_refs)]
        let tx_buffer: &'static mut [u32; AUDIO_BUFFER_SIZE] = unsafe { AUDIO_BUFFER.assume_init_mut() };
                                                       
        let dma_config = dma::dma::DmaConfig::default()
            .priority(dma::config::Priority::VeryHigh)
            .memory_increment(true)
            .peripheral_increment(false)
            .circular_buffer(true)
            .transfer_complete_interrupt(true);
        let mut dma1_str1: dma::Transfer<_, _, dma::MemoryToPeripheral, _, _> =
            dma::Transfer::init(
                dma1_streams.1,
                unsafe { pac::Peripherals::steal().SAI1.dma_ch_a() }, // Channel A
                tx_buffer,
                None,
                dma_config,
            );


        let mut audio_enable = gpioe.pe3.into_push_pull_output_in_state(PinState::High);

        // Use PLL3_P for the SAI1 clock
        let sai1_rec = ccdr.peripheral.SAI1.kernel_clk_mux(Sai1ClkSel::Pll3P);
        let master_config =
            I2SChanConfig::new(I2SDir::Tx).set_mono_mode(true);

        let slave_config = I2SChanConfig::new(I2SDir::Rx)
            .set_sync_type(I2SSync::Internal)
            .set_mono_mode(true);

        let sai1_pins = (
            // pg7 doesn't exist afaik but the hal needs something here
            gpiog.pg7.into_alternate(),
            gpioe.pe5.into_alternate(),
            gpioe.pe4.into_alternate(),
            gpioe.pe6.into_alternate(),
            None::<Pin<'E', 3, Alternate<6>>>
        );

        let mut sai1 = ctx.device.SAI1.i2s_ch_a(
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


        // unmask interrupt handler for dma 1, stream 1
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::DMA1_STR1);
        }

        // unmask interrupt handler for MDMA
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::MDMA);
        }

        // unmask interrupt handler for OCTOSPI
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::OCTOSPI1);
        }


        
        dma1_str1.start(|sai1_rb| {
            sai1.enable_dma(SaiChannel::ChannelA);
            info!("sai1 fifo waiting to receive data");
            while sai1_rb.cha().sr.read().flvl().is_empty() {}
            info!("audio started");
        });

        sai1.listen(SaiChannel::ChannelA, sai::Event::Data);
        sai1.enable();

        sai1.try_send(0, 0).unwrap();
        
        unsafe {
            #[allow(static_mut_refs)]
            TRANSFER_DMA1_STR1.write(Some(dma1_str1)); // drops previous None
        }

        let audio_pos = 0;
        
        info!("Startup complete!");
        (
            SharedResources {
            },
            LocalResources {
                sai1,
                audio_pos,
                //transfer,
                spiflash_pos,
                display: disp,
                timer,
                ferris_pos,
                buttons,
                lcd,
                ferris,
                text_style,
            },
        )
    }
    #[task(priority=16, binds=DMA1_STR1, local=[audio_pos])]
    fn audio_tx(mut ctx: audio_tx::Context) {
        #[allow(static_mut_refs)]
        let tx_buffer: &'static mut [u32; AUDIO_BUFFER_SIZE] =
            unsafe { AUDIO_BUFFER.assume_init_mut() };

        #[allow(static_mut_refs)]
        if let Some(transfer) = unsafe { TRANSFER_DMA1_STR1.assume_init_mut() }
        {
            //tx_buffer.copy_from_slice(&SINE_WAVE);
            /*let audio_pos = *ctx.local.audio_pos;
            if (audio_pos < AUDIO_BUFFER_SIZE - 8)
            {
                tx_buffer[0..8].copy_from_slice(&SINE_WAVE[audio_pos..(audio_pos+8)]);
                *ctx.local.audio_pos += 8;
            }
            else 
            {
                *ctx.local.audio_pos = 0;
                if transfer.get_transfer_complete_flag() {
                    transfer.clear_transfer_complete_interrupt();
                }
            }*/

            debug!("{}: {}", ctx.local.audio_pos, tx_buffer[*ctx.local.audio_pos as usize .. *ctx.local.audio_pos as usize+8]);

            if transfer.get_transfer_complete_flag() {
                transfer.clear_transfer_complete_interrupt();
            }
        }

    }

    #[task(binds=MDMA, local=[spiflash_pos])]
    fn spiflash_rx(mut ctx: spiflash_rx::Context)
    {
        #[allow(static_mut_refs)]
        if let Some(transfer) = unsafe {  TRANSFER_SPIFLASH.assume_init_mut() }
        {
            if transfer.get_transfer_complete_flag() {
                transfer.clear_transfer_complete_interrupt();
            }
        }
    }

    #[task(priority=15, binds=LTDC, local=[display, text_style, ferris, ferris_pos, lcd, buttons])]
    fn draw(mut ctx: draw::Context) {

        unsafe { pac::Peripherals::steal().LTDC.icr.write(|w| w.crrif().set_bit()) };
        trace!("FRAME");
        update(ctx.local.ferris_pos, ctx.local.buttons, ctx.local.lcd);
        ctx.local.display.layer(|draw| {
            draw.fill_solid(&Rectangle::new(Point::new(0, 0), Size::new(320, 240)), RgbColor::RED).unwrap();

            Text::new("Hello Rust!", Point::new(120, 100), *ctx.local.text_style)
                .draw(draw)
                .unwrap();

            let ferris = Image::new(ctx.local.ferris, *ctx.local.ferris_pos);
            ferris.draw(draw).unwrap();
        });
        //unsafe { ctx.local.display.swap_layer() };
        ctx.local.display.swap_layer_wait();
    }

    // This is gross and needs a refactor
    #[task(binds = TIM2, local=[timer])]
    fn timer(mut ctx: timer::Context) {    
        unsafe { input::GLOBAL_TIMER_COUNTER += input::TIMER_PERIOD }
        ctx.local.timer.clear_irq();
    }

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

    
    fn update(ferris_pos: &mut Point, buttons: &mut Buttons, lcd: &mut Lcd) {
        buttons.tick_all();

        let button_reading = buttons.raw_read_all();
        let button_clicks = buttons.read_clicks();

        if button_reading.left.is_held() {
            ferris_pos.x -= 1;
        }
        if button_reading.right.is_held() {
            ferris_pos.x += 1;
        }
        if button_reading.up.is_held() {
            ferris_pos.y -= 1;
        }
        if button_reading.down.is_held() {
            ferris_pos.y +=1;
        }

        if button_clicks.power {
            lcd.toggle_backlight();
        }

        //buttons.reset_all();
    }
}
