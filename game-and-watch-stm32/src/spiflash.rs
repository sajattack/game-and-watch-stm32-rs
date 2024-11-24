use stm32h7xx_hal::{
    delay::Delay, prelude::*, rcc::{rec::{self, OctospiClkSel}, CoreClocks}, time::U32Ext, xspi::{Config, Octospi, OctospiError, OctospiMode, OctospiWord, SamplingEdge}
};
use stm32h7xx_hal::pac::OCTOSPI1;
use stm32h7xx_hal::gpio::{Pin, Alternate, PB2, PB1, PD12, PE2, PA1, PE11, AF9, AF11, PushPull};
use embedded_hal::blocking::delay::DelayMs;
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


pub struct SpiFlash<'a> {
    _sck: Pin<'B', 2, Alternate<9, PushPull>>,
    _d0: Pin<'B', 1, Alternate<11, PushPull>>,
    _d1: Pin<'D', 12, Alternate<9, PushPull>>,
    _d2: Pin<'E', 2, Alternate<9, PushPull>>,
    _d3: Pin<'A', 1, Alternate<9, PushPull>>,
    _nss: Pin<'E', 11, Alternate<11, PushPull>>,
    ospi: Octospi<OCTOSPI1>,
    delay: &'a mut Delay,
}

impl<'a> SpiFlash<'a> {
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
        delay: &'a mut Delay,
    ) -> Self {

        ospi_periph.dcr1.modify(|_, w| unsafe { w.csht().bits(2) });
        ospi_periph.dcr1.modify(|_, w| unsafe { w.mtyp().bits(1) });
        ospi_periph.dcr1.modify(|_, w| unsafe { w.devsize().bits(28) });

        debug!("Peripheral clock: {}", clocks.per_ck());
        let config = Config::new(32.MHz()).mode(OctospiMode::OneBit).sampling_edge(SamplingEdge::Falling).fifo_threshold(5);
        let mut ospi = ospi_periph.octospi_unchecked(config, clocks, peripheral);
        debug!("Ospi clock: {}", Octospi::<OCTOSPI1>::kernel_clk_unwrap(clocks));

        Self {
            _sck,
            _d0,
            _d1,
            _d2,
            _d3,
            _nss,
            ospi,
            delay,
        }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_RSTEN as u8), OctospiWord::None, OctospiWord::None, &[])
            .map_err(|e| Error::OspiError(e))?;

        // FIXME is there a better rtic way for this?
        self.delay.delay_ms(2u8);

        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_RST as u8), OctospiWord::None, OctospiWord::None, &[])
            .map_err(|e| Error::OspiError(e))?;

        // FIXME ditto above
        self.delay.delay_ms(20u8);

        while self.ospi.is_busy().err() == Some(OctospiError::Busy) {
            cortex_m::asm::nop();
        }

        let mut buf = [0u8; 3];
        self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDID as u8), OctospiWord::None, OctospiWord::None, 0, &mut buf)
           .map_err(|e| Error::OspiError(e))?;
        if buf != JEDEC_ID {
            return Err(Error::IdMismatch);
        }


        let mut status_reg = [0u8; 1];
            self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDSR as u8), OctospiWord::None, OctospiWord::None, 0, &mut status_reg)
            .map_err(|e| Error::OspiError(e))?;

        debug!("status_reg original {}", status_reg);


        self.delay.delay_ms(20u8);

        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_WREN as u8), OctospiWord::None, OctospiWord::None, &[])
            .map_err(|e| Error::OspiError(e))?;

        // FIXME ditto above
        self.delay.delay_ms(20u8);


        // Enable quad mode
        status_reg[0] |= (1 << 6);

        debug!("status_reg modified {}", status_reg);

        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_WRSR as u8), OctospiWord::None, OctospiWord::None, &status_reg)
            .map_err(|e| Error::OspiError(e))?;

        // FIXME ditto above
        self.delay.delay_ms(20u8);

        let mut status_reg = [0u8; 1];
            self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_RDSR as u8), OctospiWord::None, OctospiWord::None, 0, &mut status_reg)
            .map_err(|e| Error::OspiError(e))?;

        debug!("status_reg readback {}", status_reg);



        self.ospi.configure_mode(OctospiMode::FourBit)
            .map_err(|e| Error::OspiError(e))?;

        Ok(())
    }


    /// buf must be 32 bytes or less!
    pub fn read_bytes(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Error> {
        self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_4READ as u8), OctospiWord::U24(addr), OctospiWord::None, 6, buf);
        //self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_READ as u8), OctospiWord::U24(addr), OctospiWord::None, 6, buf)
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

