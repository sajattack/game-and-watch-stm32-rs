use stm32h7xx_hal::{
    prelude::*, rcc::{rec::{self, OctospiClkSel}, CoreClocks}, time::U32Ext, xspi::{Config, Octospi, OctospiError, OctospiMode, OctospiWord, SamplingEdge, OctospiModes},
    delay::Delay,
};
use stm32h7xx_hal::pac::OCTOSPI1;
use stm32h7xx_hal::gpio::{Pin, Alternate, PB2, PB1, PD12, PE2, PA1, PE11, AF9, AF11, PushPull};
use defmt::debug;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    OspiError(OctospiError),
    IdMismatch,
}

#[repr(u8)]
enum FlashCommand {
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
}

impl SpiFlash {
    pub fn new<'b>(
        _sck: Pin<'B', 2, Alternate<9, PushPull>>,
        _d0: Pin<'B', 1, Alternate<11, PushPull>>,
        _d1: Pin<'D', 12, Alternate<9, PushPull>>,
        _d2: Pin<'E', 2, Alternate<9, PushPull>>,
        _d3: Pin<'A', 1, Alternate<9, PushPull>>,
        _nss: Pin<'E', 11, Alternate<11, PushPull>>,
        ospi_periph: OCTOSPI1,
        clocks: &'b CoreClocks, 
        peripheral: rec::Octospi1,
    ) -> Self {
        let config = Config::new(16.MHz()).mode(OctospiMode::OneBit).sampling_edge(SamplingEdge::Falling).fifo_threshold(4).dummy_cycles(6);
        let mut ospi = ospi_periph.octospi_unchecked(config, clocks, peripheral);

        Self {
            _sck,
            _d0,
            _d1,
            _d2,
            _d3,
            _nss,
            ospi,
        }
    }

    pub fn init<'a>(&mut self, delay: &'a mut Delay) -> Result<(), Error> {
        // reset
        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_RSTEN as u8), OctospiWord::None, OctospiWord::None, &[])
            .map_err(|e| Error::OspiError(e))?;

        delay.delay_ms(2u8);

        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_RST as u8), OctospiWord::None, OctospiWord::None, &[])
            .map_err(|e| Error::OspiError(e))?;

        delay.delay_ms(20u8);

        // read jedec id
        let mut buf = [0u8; 3];
        self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDID as u8), OctospiWord::None, OctospiWord::None, 0, &mut buf)
           .map_err(|e| Error::OspiError(e))?;
        if buf != JEDEC_ID {
            return Err(Error::IdMismatch);
        }

        // enable writes
        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_WREN as u8), OctospiWord::None, OctospiWord::None, &[])
            .map_err(|e| Error::OspiError(e))?;

        let mut status_reg = [0u8; 1];


        self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDSR as u8), OctospiWord::None, OctospiWord::None, 0, &mut status_reg)
            .map_err(|e| Error::OspiError(e))?;

        debug!("status_reg original {}", status_reg);

        while status_reg[0] & (1 << 6) == 0  || status_reg[0] & (1 << 1) !=0 {
            // Enable quad mode
            status_reg[0] |= (1 << 6);

            debug!("status_reg modified {}", status_reg);

            self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_WRSR as u8), OctospiWord::None, OctospiWord::None, &status_reg)
                .map_err(|e| Error::OspiError(e))?;

            delay.delay_ms(20u8);

            self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDSR as u8), OctospiWord::None, OctospiWord::None, 0, &mut status_reg)
                .map_err(|e| Error::OspiError(e))?;

            debug!("status_reg readback {}", status_reg);
        }

        self.ospi.configure_modes(
            OctospiModes {
               instruction: OctospiMode::OneBit,
               address: OctospiMode::FourBit,
               alt_byte: OctospiMode::FourBit,
               data: OctospiMode::FourBit,
            }
        )
            .map_err(|e| Error::OspiError(e))?;

        self.ospi.inner_mut().ccr.modify(|_, w| unsafe { 
            w.sioo().set_bit()
        });

        self.ospi.inner_mut().dcr1.modify(|_, w| unsafe { 
            w.csht().bits(4); 
            w.mtyp().bits(1);
            w.dlybyp().set_bit();
            w.devsize().bits(19)
        });

        while self.ospi.is_busy().is_err() {
            core::hint::spin_loop();
        }

        Ok(())
    }


    /// buf must be 32 bytes or less!
    pub fn read_bytes(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Error> {
        self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_4READ as u8), OctospiWord::U24(addr), OctospiWord::None, 6, buf)
            .map_err(|e| Error::OspiError(e))?;
        Ok(())
    }

    /// buf must be 32 bytes or less!
    pub fn write_bytes(&mut self, addr: u32, buf: &[u8]) -> Result<(), Error> {
        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_PP as u8), OctospiWord::U24(addr), OctospiWord::None, buf)
            .map_err(|e| Error::OspiError(e))?;
        Ok(())
    }

    // TODO: DMA!!!!
}

