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

const AUDIO_BUFFER_SIZE: usize = 960;
const VIDEO_BUFFER_SIZE: usize = lcd::WIDTH * lcd::HEIGHT;

#[link_section = ".sram3"]
static mut FRONT_BUFFER: MaybeUninit<[u16; VIDEO_BUFFER_SIZE]> = MaybeUninit::uninit();
#[link_section = ".sram3"]
static mut BACK_BUFFER: MaybeUninit<[u16; VIDEO_BUFFER_SIZE]> = MaybeUninit::uninit();


//#[link_section = ".sram3"]
static AUDIO_BUFFER: [u16; AUDIO_BUFFER_SIZE] = [
0x8000, 0x80d6, 0x81ad, 0x8283, 0x835a, 0x8430, 0x8506, 0x85dd, 0x86b3, 0x8789, 0x885f, 0x8935,
0x8a0b, 0x8ae1, 0x8bb6, 0x8c8c, 0x8d61, 0x8e36, 0x8f0b, 0x8fe0, 0x90b5, 0x918a, 0x925e, 0x9332,
0x9406, 0x94da, 0x95ad, 0x9680, 0x9753, 0x9826, 0x98f9, 0x99cb, 0x9a9d, 0x9b6e, 0x9c40, 0x9d11,
0x9de2, 0x9eb2, 0x9f82, 0xa052, 0xa121, 0xa1f0, 0xa2bf, 0xa38d, 0xa45b, 0xa528, 0xa5f5, 0xa6c2,
0xa78e, 0xa85a, 0xa925, 0xa9f0, 0xaaba, 0xab84, 0xac4e, 0xad17, 0xaddf, 0xaea7, 0xaf6e, 0xb035,
0xb0fc, 0xb1c2, 0xb287, 0xb34c, 0xb410, 0xb4d4, 0xb597, 0xb659, 0xb71b, 0xb7dc, 0xb89d, 0xb95d,
0xba1c, 0xbadb, 0xbb99, 0xbc57, 0xbd14, 0xbdd0, 0xbe8b, 0xbf46, 0xc000, 0xc0b9, 0xc172, 0xc22a,
0xc2e1, 0xc398, 0xc44d, 0xc502, 0xc5b7, 0xc66a, 0xc71d, 0xc7cf, 0xc880, 0xc930, 0xc9e0, 0xca8f,
0xcb3d, 0xcbea, 0xcc96, 0xcd41, 0xcdec, 0xce96, 0xcf3e, 0xcfe6, 0xd08e, 0xd134, 0xd1d9, 0xd27e,
0xd321, 0xd3c4, 0xd465, 0xd506, 0xd5a6, 0xd645, 0xd6e3, 0xd780, 0xd81c, 0xd8b7, 0xd951, 0xd9ea,
0xda82, 0xdb1a, 0xdbb0, 0xdc45, 0xdcd9, 0xdd6c, 0xddfe, 0xde8f, 0xdf1f, 0xdfae, 0xe03c, 0xe0c9,
0xe155, 0xe1e0, 0xe269, 0xe2f2, 0xe37a, 0xe400, 0xe485, 0xe509, 0xe58d, 0xe60f, 0xe68f, 0xe70f,
0xe78e, 0xe80b, 0xe888, 0xe903, 0xe97d, 0xe9f6, 0xea6e, 0xeae4, 0xeb5a, 0xebce, 0xec41, 0xecb3,
0xed23, 0xed93, 0xee01, 0xee6e, 0xeeda, 0xef45, 0xefae, 0xf016, 0xf07d, 0xf0e3, 0xf147, 0xf1ab,
0xf20d, 0xf26d, 0xf2cd, 0xf32b, 0xf388, 0xf3e4, 0xf43e, 0xf497, 0xf4ef, 0xf546, 0xf59b, 0xf5ef,
0xf642, 0xf693, 0xf6e3, 0xf732, 0xf780, 0xf7cc, 0xf817, 0xf860, 0xf8a8, 0xf8ef, 0xf935, 0xf979,
0xf9bc, 0xf9fe, 0xfa3e, 0xfa7d, 0xfabb, 0xfaf7, 0xfb32, 0xfb6b, 0xfba3, 0xfbda, 0xfc10, 0xfc44,
0xfc77, 0xfca8, 0xfcd8, 0xfd07, 0xfd34, 0xfd60, 0xfd8a, 0xfdb4, 0xfddb, 0xfe02, 0xfe27, 0xfe4a,
0xfe6d, 0xfe8d, 0xfead, 0xfecb, 0xfee8, 0xff03, 0xff1d, 0xff35, 0xff4c, 0xff62, 0xff77, 0xff89,
0xff9b, 0xffab, 0xffba, 0xffc7, 0xffd3, 0xffde, 0xffe7, 0xffee, 0xfff5, 0xfffa, 0xfffd, 0xffff,
0xffff, 0xffff, 0xfffd, 0xfffa, 0xfff5, 0xffee, 0xffe7, 0xffde, 0xffd3, 0xffc7, 0xffba, 0xffab,
0xff9b, 0xff89, 0xff77, 0xff62, 0xff4c, 0xff35, 0xff1d, 0xff03, 0xfee8, 0xfecb, 0xfead, 0xfe8d,
0xfe6d, 0xfe4a, 0xfe27, 0xfe02, 0xfddb, 0xfdb4, 0xfd8a, 0xfd60, 0xfd34, 0xfd07, 0xfcd8, 0xfca8,
0xfc77, 0xfc44, 0xfc10, 0xfbda, 0xfba3, 0xfb6b, 0xfb32, 0xfaf7, 0xfabb, 0xfa7d, 0xfa3e, 0xf9fe,
0xf9bc, 0xf979, 0xf935, 0xf8ef, 0xf8a8, 0xf860, 0xf817, 0xf7cc, 0xf780, 0xf732, 0xf6e3, 0xf693,
0xf642, 0xf5ef, 0xf59b, 0xf546, 0xf4ef, 0xf497, 0xf43e, 0xf3e4, 0xf388, 0xf32b, 0xf2cd, 0xf26d,
0xf20d, 0xf1ab, 0xf147, 0xf0e3, 0xf07d, 0xf016, 0xefae, 0xef45, 0xeeda, 0xee6e, 0xee01, 0xed93,
0xed23, 0xecb3, 0xec41, 0xebce, 0xeb5a, 0xeae4, 0xea6e, 0xe9f6, 0xe97d, 0xe903, 0xe888, 0xe80b,
0xe78e, 0xe70f, 0xe68f, 0xe60f, 0xe58d, 0xe509, 0xe485, 0xe400, 0xe37a, 0xe2f2, 0xe269, 0xe1e0,
0xe155, 0xe0c9, 0xe03c, 0xdfae, 0xdf1f, 0xde8f, 0xddfe, 0xdd6c, 0xdcd9, 0xdc45, 0xdbb0, 0xdb1a,
0xda82, 0xd9ea, 0xd951, 0xd8b7, 0xd81c, 0xd780, 0xd6e3, 0xd645, 0xd5a6, 0xd506, 0xd465, 0xd3c4,
0xd321, 0xd27e, 0xd1d9, 0xd134, 0xd08e, 0xcfe6, 0xcf3e, 0xce96, 0xcdec, 0xcd41, 0xcc96, 0xcbea,
0xcb3d, 0xca8f, 0xc9e0, 0xc930, 0xc880, 0xc7cf, 0xc71d, 0xc66a, 0xc5b7, 0xc502, 0xc44d, 0xc398,
0xc2e1, 0xc22a, 0xc172, 0xc0b9, 0xc000, 0xbf46, 0xbe8b, 0xbdd0, 0xbd14, 0xbc57, 0xbb99, 0xbadb,
0xba1c, 0xb95d, 0xb89d, 0xb7dc, 0xb71b, 0xb659, 0xb597, 0xb4d4, 0xb410, 0xb34c, 0xb287, 0xb1c2,
0xb0fc, 0xb035, 0xaf6e, 0xaea7, 0xaddf, 0xad17, 0xac4e, 0xab84, 0xaaba, 0xa9f0, 0xa925, 0xa85a,
0xa78e, 0xa6c2, 0xa5f5, 0xa528, 0xa45b, 0xa38d, 0xa2bf, 0xa1f0, 0xa121, 0xa052, 0x9f82, 0x9eb2,
0x9de2, 0x9d11, 0x9c40, 0x9b6e, 0x9a9d, 0x99cb, 0x98f9, 0x9826, 0x9753, 0x9680, 0x95ad, 0x94da,
0x9406, 0x9332, 0x925e, 0x918a, 0x90b5, 0x8fe0, 0x8f0b, 0x8e36, 0x8d61, 0x8c8c, 0x8bb6, 0x8ae1,
0x8a0b, 0x8935, 0x885f, 0x8789, 0x86b3, 0x85dd, 0x8506, 0x8430, 0x835a, 0x8283, 0x81ad, 0x80d6,
0x8000, 0x7f2a, 0x7e53, 0x7d7d, 0x7ca6, 0x7bd0, 0x7afa, 0x7a23, 0x794d, 0x7877, 0x77a1, 0x76cb,
0x75f5, 0x751f, 0x744a, 0x7374, 0x729f, 0x71ca, 0x70f5, 0x7020, 0x6f4b, 0x6e76, 0x6da2, 0x6cce,
0x6bfa, 0x6b26, 0x6a53, 0x6980, 0x68ad, 0x67da, 0x6707, 0x6635, 0x6563, 0x6492, 0x63c0, 0x62ef,
0x621e, 0x614e, 0x607e, 0x5fae, 0x5edf, 0x5e10, 0x5d41, 0x5c73, 0x5ba5, 0x5ad8, 0x5a0b, 0x593e,
0x5872, 0x57a6, 0x56db, 0x5610, 0x5546, 0x547c, 0x53b2, 0x52e9, 0x5221, 0x5159, 0x5092, 0x4fcb,
0x4f04, 0x4e3e, 0x4d79, 0x4cb4, 0x4bf0, 0x4b2c, 0x4a69, 0x49a7, 0x48e5, 0x4824, 0x4763, 0x46a3,
0x45e4, 0x4525, 0x4467, 0x43a9, 0x42ec, 0x4230, 0x4175, 0x40ba, 0x4000, 0x3f47, 0x3e8e, 0x3dd6,
0x3d1f, 0x3c68, 0x3bb3, 0x3afe, 0x3a49, 0x3996, 0x38e3, 0x3831, 0x3780, 0x36d0, 0x3620, 0x3571,
0x34c3, 0x3416, 0x336a, 0x32bf, 0x3214, 0x316a, 0x30c2, 0x301a, 0x2f72, 0x2ecc, 0x2e27, 0x2d82,
0x2cdf, 0x2c3c, 0x2b9b, 0x2afa, 0x2a5a, 0x29bb, 0x291d, 0x2880, 0x27e4, 0x2749, 0x26af, 0x2616,
0x257e, 0x24e6, 0x2450, 0x23bb, 0x2327, 0x2294, 0x2202, 0x2171, 0x20e1, 0x2052, 0x1fc4, 0x1f37,
0x1eab, 0x1e20, 0x1d97, 0x1d0e, 0x1c86, 0x1c00, 0x1b7b, 0x1af7, 0x1a73, 0x19f1, 0x1971, 0x18f1,
0x1872, 0x17f5, 0x1778, 0x16fd, 0x1683, 0x160a, 0x1592, 0x151c, 0x14a6, 0x1432, 0x13bf, 0x134d,
0x12dd, 0x126d, 0x11ff, 0x1192, 0x1126, 0x10bb, 0x1052, 0xfea, 0xf83, 0xf1d, 0xeb9, 0xe55,
0xdf3, 0xd93, 0xd33, 0xcd5, 0xc78, 0xc1c, 0xbc2, 0xb69, 0xb11, 0xaba, 0xa65, 0xa11,
0x9be, 0x96d, 0x91d, 0x8ce, 0x880, 0x834, 0x7e9, 0x7a0, 0x758, 0x711, 0x6cb, 0x687,
0x644, 0x602, 0x5c2, 0x583, 0x545, 0x509, 0x4ce, 0x495, 0x45d, 0x426, 0x3f0, 0x3bc,
0x389, 0x358, 0x328, 0x2f9, 0x2cc, 0x2a0, 0x276, 0x24c, 0x225, 0x1fe, 0x1d9, 0x1b6,
0x193, 0x173, 0x153, 0x135, 0x118, 0xfd, 0xe3, 0xcb, 0xb4, 0x9e, 0x89, 0x77,
0x65, 0x55, 0x46, 0x39, 0x2d, 0x22, 0x19, 0x12, 0x0b, 0x06, 0x03, 0x01,
0x00, 0x01, 0x03, 0x06, 0x0b, 0x12, 0x19, 0x22, 0x2d, 0x39, 0x46, 0x55,
0x65, 0x77, 0x89, 0x9e, 0xb4, 0xcb, 0xe3, 0xfd, 0x118, 0x135, 0x153, 0x173,
0x193, 0x1b6, 0x1d9, 0x1fe, 0x225, 0x24c, 0x276, 0x2a0, 0x2cc, 0x2f9, 0x328, 0x358,
0x389, 0x3bc, 0x3f0, 0x426, 0x45d, 0x495, 0x4ce, 0x509, 0x545, 0x583, 0x5c2, 0x602,
0x644, 0x687, 0x6cb, 0x711, 0x758, 0x7a0, 0x7e9, 0x834, 0x880, 0x8ce, 0x91d, 0x96d,
0x9be, 0xa11, 0xa65, 0xaba, 0xb11, 0xb69, 0xbc2, 0xc1c, 0xc78, 0xcd5, 0xd33, 0xd93,
0xdf3, 0xe55, 0xeb9, 0xf1d, 0xf83, 0xfea, 0x1052, 0x10bb, 0x1126, 0x1192, 0x11ff, 0x126d,
0x12dd, 0x134d, 0x13bf, 0x1432, 0x14a6, 0x151c, 0x1592, 0x160a, 0x1683, 0x16fd, 0x1778, 0x17f5,
0x1872, 0x18f1, 0x1971, 0x19f1, 0x1a73, 0x1af7, 0x1b7b, 0x1c00, 0x1c86, 0x1d0e, 0x1d97, 0x1e20,
0x1eab, 0x1f37, 0x1fc4, 0x2052, 0x20e1, 0x2171, 0x2202, 0x2294, 0x2327, 0x23bb, 0x2450, 0x24e6,
0x257e, 0x2616, 0x26af, 0x2749, 0x27e4, 0x2880, 0x291d, 0x29bb, 0x2a5a, 0x2afa, 0x2b9b, 0x2c3c,
0x2cdf, 0x2d82, 0x2e27, 0x2ecc, 0x2f72, 0x301a, 0x30c2, 0x316a, 0x3214, 0x32bf, 0x336a, 0x3416,
0x34c3, 0x3571, 0x3620, 0x36d0, 0x3780, 0x3831, 0x38e3, 0x3996, 0x3a49, 0x3afe, 0x3bb3, 0x3c68,
0x3d1f, 0x3dd6, 0x3e8e, 0x3f47, 0x4000, 0x40ba, 0x4175, 0x4230, 0x42ec, 0x43a9, 0x4467, 0x4525,
0x45e4, 0x46a3, 0x4763, 0x4824, 0x48e5, 0x49a7, 0x4a69, 0x4b2c, 0x4bf0, 0x4cb4, 0x4d79, 0x4e3e,
0x4f04, 0x4fcb, 0x5092, 0x5159, 0x5221, 0x52e9, 0x53b2, 0x547c, 0x5546, 0x5610, 0x56db, 0x57a6,
0x5872, 0x593e, 0x5a0b, 0x5ad8, 0x5ba5, 0x5c73, 0x5d41, 0x5e10, 0x5edf, 0x5fae, 0x607e, 0x614e,
0x621e, 0x62ef, 0x63c0, 0x6492, 0x6563, 0x6635, 0x6707, 0x67da, 0x68ad, 0x6980, 0x6a53, 0x6b26,
0x6bfa, 0x6cce, 0x6da2, 0x6e76, 0x6f4b, 0x7020, 0x70f5, 0x71ca, 0x729f, 0x7374, 0x744a, 0x751f,
0x75f5, 0x76cb, 0x77a1, 0x7877, 0x794d, 0x7a23, 0x7afa, 0x7bd0, 0x7ca6, 0x7d7d, 0x7e53, 0x7f2a
];


#[rtic::app( device = stm32h7xx_hal::stm32, peripherals = true )]
mod app {
    use super::{AUDIO_BUFFER_SIZE, VIDEO_BUFFER_SIZE};
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
        //nb::block!(audio.try_send(0, 0)).unwrap();

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
                audio_divider: 23,
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
                let newval = (ctx.local.audio_divider).checked_sub(1);
                if newval.is_none()
                {
                    *ctx.local.audio_pos += 1;
                    *ctx.local.audio_divider = 23;
                }
                else
                {
                    (*ctx.local.audio_divider) = newval.unwrap();
                }

                // we set the i2s data size as 16bit but the dumb library still wants a u32
                if value.is_some() {
                    nb::block!(ctx.local.audio.try_send(0, *value.unwrap() as u32));
                }
                //trace!("audio pos: {} audio_val: 0x{:04x}", ctx.local.audio_pos, value);

            }
            else
            {
                *ctx.local.audio_pos = 0;
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
