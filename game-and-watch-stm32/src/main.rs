#![no_main]
#![no_std]

mod lcd;
use lcd::*;

use core::ptr::addr_of_mut;

use cortex_m_rt::entry;
use stm32h7xx_hal::{pac, prelude::*, spi::{self, Spi}, ltdc, gpio::Speed};
use embedded_display_controller::{DisplayController, DisplayControllerLayer, DisplayConfiguration};

use embedded_graphics::{image::Image, primitives::Rectangle, pixelcolor::Rgb565};
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

use tinybmp::Bmp;

use utilities_display::write::write_to::WriteTo;
use utilities_display::display_target::BufferedDisplay;

use defmt::{info, error};
use defmt_rtt as _;
use panic_probe as _;

#[macro_use]
mod utilities_display;

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

#[entry]
fn main() -> ! {
    info!("");
    info!("GAME & WATCH TEST");
    info!("");


    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();


    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.ldo().vos0(&dp.SYSCFG).freeze();


    // Constrain and Freeze clock
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(280.MHz())
        .pll1_q_ck(280.MHz())
        .pll1_r_ck(280.MHz())

        .pll2_p_ck(60.MHz())
        .pll2_q_ck(280.MHz())
        .pll2_r_ck(60.MHz())


        .pll3_p_ck(280.MHz())
        .pll3_q_ck(280.MHz())
        .pll3_r_ck(25.MHz())

        .freeze(pwrcfg, &dp.SYSCFG);


    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);
    let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
    let gpioh = dp.GPIOH.split(ccdr.peripheral.GPIOH);

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


    let sck = gpiob.pb13.into_alternate();
    let miso = gpioc.pc2.into_alternate();
    let mosi = gpiob.pb15.into_alternate();
    let cs = gpiob.pb12.into_push_pull_output();

    let pa4 = gpioa.pa4.into_push_pull_output();
    let pa5 = gpioa.pa5.into_push_pull_output();
    let pa6 = gpioa.pa6.into_push_pull_output();

    let disable_3v3 = gpiod.pd1.into_push_pull_output();
    let enable_1v8  = gpiod.pd4.into_push_pull_output();
    let reset = gpiod.pd8.into_push_pull_output();


    let spi = dp.SPI2.spi((sck, miso, mosi), spi::MODE_0, 25.MHz(), ccdr.peripheral.SPI2, &ccdr.clocks);

    let mut ltdc = ltdc::Ltdc::new(dp.LTDC, ccdr.peripheral.LTDC, &ccdr.clocks);
    ltdc.init(
        DisplayConfiguration {
            active_width: WIDTH as u16,
            active_height: HEIGHT as u16,
            h_back_porch: 200,
            h_front_porch: 430,
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


    info!("LCD Clock: {}", ltdc.clock());

    let mut layer = ltdc.split();

    let mut front_buffer = [0u16; lcd::WIDTH * lcd::HEIGHT];
    let mut back_buffer = [0u16; lcd::WIDTH * lcd::HEIGHT];

    let mut lcd = Lcd::new(pa4, pa5, pa6, disable_3v3, enable_1v8, reset, cs, spi, delay);
    lcd.init();

    let mut disp = BufferedDisplay::new(layer, &mut front_buffer, &mut back_buffer, WIDTH, HEIGHT);

    info!("Initialised Display...");
    

    loop { 
        disp.layer(|draw| {
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
        disp.swap_layer_wait();
    }
}

