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

use stm32h7xx_hal::{sai, stm32, dma::{mdma::StreamX, MasterTransfer, PeripheralToMemory, Transfer, self}};

const AUDIO_BUFFER_SIZE: usize = 192;
static mut AUDIO_BUFFER: MaybeUninit<[u32; AUDIO_BUFFER_SIZE]> = MaybeUninit::uninit();

type TransferDma1Str1 = dma::Transfer<
    dma::dma::Stream1<stm32::DMA1>,
    sai::dma::ChannelA<stm32::SAI1>,
    dma::MemoryToPeripheral,
    &'static mut [u32; AUDIO_BUFFER_SIZE],
    dma::DBTransfer,
>;

static mut TRANSFER_DMA1_STR1: MaybeUninit<Option<TransferDma1Str1>> = MaybeUninit::uninit();

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


#[rtic::app( device = stm32h7xx_hal::stm32, peripherals = true )]
mod app {
    use super::AUDIO_BUFFER_SIZE;
    use crate::{AUDIO_BUFFER, SINE_WAVE, TRANSFER_DMA1_STR1};
    use core::mem::MaybeUninit;
    use ltdc::Ltdc;
    use stm32h7xx_hal::{stm32, dma::{mdma::StreamX, MasterTransfer, PeripheralToMemory, Transfer, self}, gpio::{Alternate, Pin, PinState, Speed}, ltdc::{self, LtdcLayer1}, pac::{self, rcc::cdccipr::FMCSEL_A, OCTOSPI1, SAI1}, prelude::*, rcc::rec::{Mdma, Octospi1, Sai1ClkSel, Spi123ClkSel}, sai::{
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
        delay: stm32h7xx_hal::delay::Delay,
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


        let gpioa = ctx.device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = ctx.device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = ctx.device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = ctx.device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = ctx.device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = ctx.device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = ctx.device.GPIOG.split(ccdr.peripheral.GPIOG);
        let gpioh = ctx.device.GPIOH.split(ccdr.peripheral.GPIOH);

        let mut delay = stm32h7xx_hal::delay::Delay::new(ctx.core.SYST, ccdr.clocks);


        // configure dma1
        let dma1_streams = dma::dma::StreamsTuple::new(ctx.device.DMA1, ccdr.peripheral.DMA1);

        #[allow(static_mut_refs)] // TODO: Fix this
        let tx_buffer: &'static mut [u32; AUDIO_BUFFER_SIZE] =
            unsafe { AUDIO_BUFFER.assume_init_mut() }; // uninitialised memory
        let dma_config = dma::dma::DmaConfig::default()
            .priority(dma::config::Priority::High)
            .memory_increment(true)
            .peripheral_increment(false)
            .circular_buffer(true)
            .transfer_complete_interrupt(true)
            .fifo_enable(false);
        let mut dma1_str1: dma::Transfer<_, _, dma::MemoryToPeripheral, _, _> =
            dma::Transfer::init(
                dma1_streams.1,
                unsafe { pac::Peripherals::steal().SAI1.dma_ch_a() }, // Channel A
                tx_buffer,
                None,
                dma_config,
            );

        // configure sai
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
        #[allow(static_mut_refs)] // TODO: Fix this
        TRANSFER_DMA1_STR1.write(Some(dma1_str1)); // drops previous None
    }

        let ferris_pos = Point::new(120, 125);
        
        info!("Startup complete!");
        (
            SharedResources {

            },
            LocalResources {
                sai1,
                delay,
            },
        )
    }

    #[task(priority=16, binds=DMA1_STR1)]
    fn audio_tx(mut ctx: audio_tx::Context) {

        #[allow(static_mut_refs)] // TODO: Fix this
        let tx_buffer: &'static mut [u32; AUDIO_BUFFER_SIZE] =
            unsafe { AUDIO_BUFFER.assume_init_mut() };

        #[allow(static_mut_refs)] // TODO: Fix this
        if let Some(transfer) = unsafe { TRANSFER_DMA1_STR1.assume_init_mut() }
        {
            if transfer.get_transfer_complete_flag() {
                transfer.clear_transfer_complete_interrupt();
            }

            let mut index = 0;
            while index < AUDIO_BUFFER_SIZE {
                tx_buffer[index] = SINE_WAVE[index] as u32;
                index += 1;
            }
        }
    }
}
