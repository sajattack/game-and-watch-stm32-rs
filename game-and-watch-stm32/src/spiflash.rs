use stm32h7xx_hal::{
    delay::Delay, dma::{mdma::{MdmaConfig, MdmaIncrement, MdmaTransferRequest, MdmaTrigger, StreamX, StreamsTuple}, MasterTransfer, PeripheralToMemory, Transfer}, gpio::Speed, pac::MDMA, prelude::*, rcc::{rec::{ self, Mdma, OctospiClkSel}, CoreClocks}, time::U32Ext, xspi::{Config, Octospi, OctospiError, OctospiMode, OctospiModes, OctospiWord, SamplingEdge}
};
use stm32h7xx_hal::pac::OCTOSPI1;
use stm32h7xx_hal::gpio::{Pin, Alternate, PB2, PB1, PD12, PE2, PA1, PE11, AF9, AF11, PushPull};
use defmt::debug;
use core::mem::MaybeUninit;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    OspiError(OctospiError),
    IdMismatch,
}

#[repr(u8)]
pub enum FlashCommand {
    CMD_WRSR = 0x01,
    CMD_READ = 0x03,
    CMD_RDSR = 0x05,
    CMD_WREN = 0x06,
    CMD_RDCR = 0x15,
    CMD_PP = 0x38,
    CMD_RSTEN = 0x66,
    CMD_QREAD = 0x6B,
    CMD_RST = 0x99,
    CMD_RDID = 0x9f,
    CMD_4READ = 0xEB,
}

// FIXME support expanded flash as well
// (currently only supporting the stock chip)
pub const JEDEC_ID: [u8; 3] = [0xc2, 0x25, 0x34];


pub struct SpiFlash {
    _sck: Pin<'B', 2, Alternate<9, PushPull>>,
    _d0: Pin<'B', 1, Alternate<11, PushPull>>,
    _d1: Pin<'D', 12, Alternate<9, PushPull>>,
    _d2: Pin<'E', 2, Alternate<9, PushPull>>,
    _d3: Pin<'A', 1, Alternate<9, PushPull>>,
    _nss: Pin<'E', 11, Alternate<11, PushPull>>,
    ospi: Octospi<OCTOSPI1>,
    dp_mdma: MDMA,
    p_mdma: Mdma, 
}

impl SpiFlash {
    pub fn new<'b>(
        mut _sck: Pin<'B', 2, Alternate<9, PushPull>>,
        mut _d0: Pin<'B', 1, Alternate<11, PushPull>>,
        mut _d1: Pin<'D', 12, Alternate<9, PushPull>>,
        mut _d2: Pin<'E', 2, Alternate<9, PushPull>>,
        mut _d3: Pin<'A', 1, Alternate<9, PushPull>>,
        mut _nss: Pin<'E', 11, Alternate<11, PushPull>>,
        ospi_periph: OCTOSPI1,
        clocks: &'b CoreClocks, 
        peripheral: rec::Octospi1,
        delay: &mut stm32h7xx_hal::delay::Delay,
        dp_mdma: MDMA,
        p_mdma: Mdma,
    ) -> Self {
        let config = Config::new(
            16.MHz()
        )
            .mode(OctospiMode::OneBit)
            .sampling_edge(SamplingEdge::Falling)
            .fifo_threshold(8*4)
            .dummy_cycles(6);

        let mut ospi = ospi_periph.octospi_unchecked(config, clocks, peripheral);

        ospi.configure_mode(OctospiMode::OneBit);

        // reset
        ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_RSTEN as u8), OctospiWord::None, OctospiWord::None, &[])
            .unwrap(); // FIXME
            //.map_err(|e| Error::OspiError(e))?;

        delay.delay_ms(2u8);

        ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_RST as u8), OctospiWord::None, OctospiWord::None, &[])
            .unwrap(); // FIXME
            //.map_err(|e| Error::OspiError(e))?;

        delay.delay_ms(20u8);

        // read jedec id
        let mut buf = [0u8; 3];
        ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDID as u8), OctospiWord::None, OctospiWord::None, 0, &mut buf)
            .unwrap(); // FIXME
           //.map_err(|e| Error::OspiError(e))?;
        if buf != JEDEC_ID {
            panic!("JEDEC ID Mismatch"); // FIXME
            //return Err(Error::IdMismatch);
        }

        // enable writes
        //ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_WREN as u8), OctospiWord::None, OctospiWord::None, &[])
            //.unwrap(); // FIXME
            ////.map_err(|e| Error::OspiError(e))?;

        let mut status_reg = [0u8; 1];


        ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDSR as u8), OctospiWord::None, OctospiWord::None, 0, &mut status_reg)
            .unwrap(); // FIXME
            //.map_err(|e| Error::OspiError(e))?;

        debug!("status_reg original {}", status_reg);

        while status_reg[0] & (1 << 6) == 0  || status_reg[0] & (1 << 1) !=0 {
            // Enable quad mode
            status_reg[0] |= (1 << 6);

            debug!("status_reg modified {}", status_reg);

            ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_WRSR as u8), OctospiWord::None, OctospiWord::None, &status_reg)
                .unwrap(); // FIXME
                //.map_err(|e| Error::OspiError(e))?;

            delay.delay_ms(20u8);

            ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDSR as u8), OctospiWord::None, OctospiWord::None, 0, &mut status_reg)
                .unwrap(); // FIXME
                //.map_err(|e| Error::OspiError(e))?;

            debug!("status_reg readback {}", status_reg);
        }

        ospi.configure_modes(
            OctospiModes {
               instruction: OctospiMode::OneBit,
               address: OctospiMode::FourBit,
               alt_byte: OctospiMode::FourBit,
               data: OctospiMode::FourBit,
            }
        )
            .unwrap(); // FIXME
            //.map_err(|e| Error::OspiError(e))?;

        //ospi.inner_mut().ccr.modify(|_, w| unsafe { 
            //w.sioo().set_bit()
        //});

        ospi.inner_mut().cr.modify(|_, w| unsafe {
            w.fmode().bits(3);
            w.en().set_bit()
        });

        ospi.inner_mut().dcr1.modify(|_, w| unsafe { 
            w.csht().bits(2); 
            w.mtyp().bits(1);
            w.dlybyp().set_bit();
            w.devsize().bits(0x1b)
        });

        ospi.inner_mut().ccr.modify(|_, w| unsafe {
            w.dmode().bits(1);
            w.abmode().bits(0);
            w.adsize().bits(3);
            w.admode().bits(1);
            w.isize().bits(0);
            w.imode().bits(1)
        });

        while ospi.is_busy().is_err() {
            core::hint::spin_loop();
        }

        _sck.set_speed(Speed::VeryHigh);
        _d0.set_speed(Speed::VeryHigh);
        _d1.set_speed(Speed::VeryHigh);
        _d2.set_speed(Speed::VeryHigh);
        _d3.set_speed(Speed::VeryHigh);
        _nss.set_speed(Speed::VeryHigh);

        Self {
            _sck,
            _d0,
            _d1,
            _d2,
            _d3,
            _nss,
            ospi,
            dp_mdma,
            p_mdma,
        }
    }

    pub fn begin_transfer<'a>(mut self, buffer: &'static mut MaybeUninit<[u32; crate::AUDIO_BUFFER_SIZE]>, transfer_pos: &mut usize) -> Transfer<StreamX<MDMA, 0>, Octospi<OCTOSPI1>, PeripheralToMemory, &'static mut MaybeUninit<[u32; crate::AUDIO_BUFFER_SIZE]>, MasterTransfer>
{
        self.ospi.begin_read_extended(OctospiWord::U8(FlashCommand::CMD_4READ as u8), OctospiWord::U24(*transfer_pos as u32), OctospiWord::None, 6, 8*4);


        let dmaconfig = MdmaConfig::default()
            .transfer_complete_interrupt(true)
            .source_increment(MdmaIncrement::Increment)
            .destination_increment(MdmaIncrement::Increment)
            .hardware_transfer_request(MdmaTransferRequest::Octospi1TcTrg)
            .trigger_mode(MdmaTrigger::Buffer)
            .buffer_length(8*4);

        let streams = StreamsTuple::new(self.dp_mdma, self.p_mdma);

        let mut transfer: stm32h7xx_hal::dma::Transfer<
            _,
            _,
            stm32h7xx_hal::dma::PeripheralToMemory,
            _,
            _,
        > = stm32h7xx_hal::dma::Transfer::init_master(
            streams.0,
            self.ospi,
            buffer,
            None,
            dmaconfig,
        );

        transfer.start(|_|{});
        transfer_pos.wrapping_add(8*4);
        transfer
    }
}
