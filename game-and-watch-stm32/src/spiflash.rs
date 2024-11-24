use stm32h7xx_hal::{
    xspi::{Octospi, Config, OctospiMode, OctospiWord, OctospiError},
    time::U32Ext,
    rcc::{CoreClocks, rec},
    prelude::*,
};
use stm32h7xx_hal::pac::OCTOSPI1;
use stm32h7xx_hal::gpio::{PB2, PB1, PD12, PE2, PA1, PE11};


pub enum Error {
    OspiError(OctospiError),
    IdMismatch,
}

#[repr(u8)]
enum FlashCommand {
    CMD_WRSR = 0x01,
    CMD_RDSR = 0x05,
    CMD_RDCR = 0x15,
    CMD_PP = 0x38,
    CMD_RSTEN = 0x66,
    CMD_RST = 0x99,
    CMD_RDID = 0x9f,
    CMD_READ = 0xEB,
}

pub const JEDEC_ID: u16 = 


pub struct SpiFlash {
    ospi: Octospi<OCTOSPI1>,
}

impl SpiFlash {
    pub fn new<'a>(
        sck: PB2,
        d0: PB1,
        d1: PD12,
        d2: PE2,
        d3: PA1,
        nss: PE11,
        ospi_periph: OCTOSPI1,
        clocks: &'a CoreClocks, 
        peripheral: rec::Octospi1,
    ) -> Self {
        let config = Config::new(64.MHz()).mode(OctospiMode::FourBit).dummy_cycles(0);
        let mut ospi = ospi_periph.octospi_unchecked(config, clocks, peripheral);

        Self {
            ospi
        }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        
        Ok(())
    }


    /// buf must be 32 bytes or less!
    pub fn read_bytes(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Error> {
        while self.ospi.is_busy() {
            // not super clear if this is right or if it should be core::hint::spin_loop
            cortex_m::asm::wfe();
        }
        self.ospi.read_extended(OctospiWord::U8(FlashCommand::CMD_READ), OctospiWord::U24(addr), OctospiWord::None, 6, buf)
            .map_err(|e| Error::OspiError(e))?;
        Ok(())
    }

    /// buf must be 32 bytes or less!
    pub fn write_bytes(&mut self, addr: u32, buf: &[u8]) -> Result<(), Error> {
        while self.ospi.is_busy() {
            // not super clear if this is right or if it should be core::hint::spin_loop
            cortex_m::asm::wfe();
        }
        self.ospi.write_extended(OctospiWord::U8(FlashCommand::CMD_PP), OctospiWord::U24(addr), 0, buf)
            .map_err(|e| Error::OspiError(e))?;
        Ok(())
    }



    // TODO: DMA!!!!
}

