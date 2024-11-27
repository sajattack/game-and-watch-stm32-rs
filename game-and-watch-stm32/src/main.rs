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
//mod input;
mod spiflash;

#[macro_use]
mod utilities_display;

use panic_probe as _;

static mut FRONT_BUFFER: [u16; lcd::WIDTH * lcd::HEIGHT] = [0u16; lcd::WIDTH * lcd::HEIGHT];
static mut BACK_BUFFER: [u16; lcd::WIDTH * lcd::HEIGHT] = [0u16; lcd::WIDTH * lcd::HEIGHT];

#[rtic::app( device = stm32h7xx_hal::stm32, peripherals = true )]
mod app {
    use ltdc::Ltdc;
    use stm32h7xx_hal::{gpio::{Alternate, Pin, PinState, Speed}, stm32::Interrupt, ltdc::{self, LtdcLayer1}, pac::{self, rcc::cdccipr::FMCSEL_A, SAI1}, prelude::*, rcc::rec::{Sai1ClkSel, Spi123ClkSel}, 
        sai::{
            self, I2SChanConfig, I2SDataSize, I2SDir, I2SSync, I2sUsers, Sai,
            SaiChannel, SaiI2sExt, I2S,
        }, spi::{self, Spi}, time::Hertz, traits::i2s::FullDuplex
    };
    use embedded_display_controller::{DisplayController, DisplayControllerLayer, DisplayConfiguration};

    use embedded_graphics::{image::Image, primitives::Rectangle, pixelcolor::Rgb565};
    use embedded_graphics::mono_font::{ascii, MonoTextStyle};
    use embedded_graphics::prelude::*;
    use embedded_graphics::text::Text;

    use rtic_monotonics::systick::prelude::*;

    use tinybmp::Bmp;

    use defmt::{info, error, debug, trace};
    use defmt_rtt as _;

    use crate::utilities_display::write::write_to::WriteTo;
    use crate::utilities_display::display_target::BufferedDisplay;

    use crate::lcd::{self, *};
    use crate::spiflash::{self, *};
    //use crate::input::*;

    #[shared]
    struct SharedResources {
        audio: Sai<SAI1, I2S>,
        spiflash: SpiFlash,
    }
    #[local]
    struct LocalResources {
        audio_pos: usize,
        display: BufferedDisplay<'static, LtdcLayer1>,
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
            .pll2_q_ck(144.MHz())
            .pll2_r_ck(6.MHz())


            .pll3_p_ck(150.MHz())
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

        //let buttons: Buttons = ButtonPins::new(
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

        ltdc.inner_mut().srcr.modify(|_, w| w.vbr().set_bit());

        ltdc.listen();

        info!("LCD Clock: {}", ltdc.clock());

        let mut layer = ltdc.split();

        let mut lcd = Lcd::new(pa4, pa5, pa6, disable_3v3, enable_1v8, reset, cs, spi);
        lcd.init(&mut delay).unwrap();

        let mut disp = BufferedDisplay::new(layer, unsafe{ crate::FRONT_BUFFER.as_mut() }, unsafe { crate::BACK_BUFFER.as_mut() }, WIDTH, HEIGHT);

        info!("Initialised Display...");


        let mut spiflash = SpiFlash::new(
            gpiob.pb2.into(),
            gpiob.pb1.into(),
            gpiod.pd12.into(),
            gpioe.pe2.into(),
            gpioa.pa1.into(),
            gpioe.pe11.into(),
            ctx.device.OCTOSPI1, &ccdr.clocks, ccdr.peripheral.OCTOSPI1);

        spiflash.init(&mut delay).unwrap();

        let mut audio_enable = gpioe.pe3.into_push_pull_output_in_state(PinState::High);

        // Use PLL3_P for the SAI1 clock
        let sai1_rec = ccdr.peripheral.SAI1.kernel_clk_mux(Sai1ClkSel::Pll2P);
        let master_config =
            I2SChanConfig::new(I2SDir::Tx).set_frame_sync_active_high(true);

        let slave_config = I2SChanConfig::new(I2SDir::Rx)
            .set_sync_type(I2SSync::Internal)
            .set_frame_sync_active_high(true);

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

        
        info!("Startup complete!");
        (
            SharedResources {
                audio,
                spiflash,
            },
            LocalResources {
                audio_pos: 0,
                display: disp,
            },
        )
    }

    //#[task(binds=SAI1, shared=[audio, spiflash], local=[audio_pos])]
    //fn audio_tx(mut ctx: audio_tx::Context) {

        //ctx.shared.audio.lock(|audio| {
            //for _ in 0..48_000
            //{
                //let mut buf = [0u8; 2];
                //ctx.shared.spiflash.lock(|spiflash| {
                    //spiflash.read_bytes(*ctx.local.audio_pos as u32, &mut buf).unwrap();
                //});

                //let value = u16::from_le_bytes(buf);

                //nb::block!(audio.try_send(0, value as u32)).unwrap();

                //if *ctx.local.audio_pos < AUDIO_SIZE -2 {
                    //*ctx.local.audio_pos += 2;
                //}
                //else {
                    //*ctx.local.audio_pos = 0;
                //}
            //}
        //});
        //trace!("audio pos: {}", ctx.local.audio_pos);
    //}

    #[task(binds = LTDC, local = [display])]
    fn draw(ctx: draw::Context) {
        trace!("FRAME");
        ctx.local.display.layer(|draw| {
            draw.clear();
            draw.fill_solid(&Rectangle::new(Point::new(0, 0), Size::new(320, 240)), RgbColor::RED).unwrap();

            let text_style =
                MonoTextStyle::new(&ascii::FONT_9X18, RgbColor::WHITE);
            Text::new("Hello Rust!", Point::new(120, 100), text_style)
                .draw(draw)
                .unwrap();

            let ferris: Bmp<Rgb565> =
                Bmp::from_slice(include_bytes!("../assets/ferris.bmp")).unwrap();
            let ferris = Image::new(&ferris, Point::new(120, 125));
            ferris.draw(draw).unwrap();
        });
        ctx.local.display.swap_layer_wait();
    }

    #[idle]
    fn idle(cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }
}
