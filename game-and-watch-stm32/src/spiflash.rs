use defmt::debug;
use embassy_time::Timer;
use embassy_stm32::{
    mode::Blocking, ospi::{AddressSize, ChipSelectHighTime, D0Pin, D1Pin, D2Pin, D3Pin, DummyCycles, FIFOThresholdLevel, Instance, MemorySize, MemoryType, NSSPin, Ospi, OspiWidth, SckPin, TransferConfig, WrapSize}, pac::{self, octospi::vals::FunctionalMode}, peripherals::{self, OCTOSPI1, PA1, PB1, PB2, PD12, PE11, PE2}, rcc::frequency, Peripheral, PeripheralRef
};

use pac::common::Write;

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


pub struct SpiFlash<'a, T: embassy_stm32::ospi::Instance> {
    ospi: Ospi<'a, T, Blocking>,
}

impl<'a, T: embassy_stm32::ospi::Instance> SpiFlash<'a, T> 
    where PB2: SckPin<T>,
        PB1: D0Pin<T>,
        PD12: D1Pin<T>,
        PE2: D2Pin<T>,
        PA1: D3Pin<T>,
        PE11: NSSPin<T>,
{
    pub fn new(
        sck: PeripheralRef<'a, PB2>,
        d0:  PeripheralRef<'a, PB1>,
        d1: PeripheralRef<'a, PD12>,
        d2: PeripheralRef<'a, PE2>, 
        d3: PeripheralRef<'a, PA1>,
        nss: PeripheralRef<'a, PE11>,
        ospi: PeripheralRef<'a, T>,
    )
    -> Self
    {

        let qspi_config = embassy_stm32::ospi::Config {
            fifo_threshold: FIFOThresholdLevel::_32Bytes,
            memory_type: MemoryType::Macronix,
            device_size: MemorySize::_1MiB,
            chip_select_high_time: ChipSelectHighTime::_4Cycle,
            free_running_clock: false,
            clock_mode: false,
            wrap_size: WrapSize::None,
            clock_prescaler: 1,
            sample_shifting: false,
            delay_hold_quarter_cycle: false,
            chip_select_boundary: 0,
            delay_block_bypass: true,
            max_transfer: 0,
            refresh: 0
        };

        debug!("OSPI1 freq {}", frequency::<peripherals::OCTOSPI1>());

        let ospi = embassy_stm32::ospi::Ospi::new_blocking_quadspi(
            ospi,
            sck,
            d0,
            d1,
            d2,
            d3,
            nss,
            qspi_config,
        );

        Self {
            ospi,
        }
    }

    pub async fn init(&mut self)
    {
        // Reset
        let transaction: TransferConfig = TransferConfig {
            instruction: Some(FlashCommand::CMD_RSTEN as u32),
            iwidth: OspiWidth::SING,
            adwidth: OspiWidth::NONE,
            adsize: AddressSize::_24bit,
            dwidth: OspiWidth::NONE,
            dummy: DummyCycles::_0,
            ..Default::default()
        };

        self.ospi.command(&transaction).await.unwrap();

        Timer::after_millis(2).await;

        let transaction: TransferConfig = TransferConfig {
            instruction: Some(FlashCommand::CMD_RST as u32),
            iwidth: OspiWidth::SING,
            adwidth: OspiWidth::NONE,
            adsize: AddressSize::_24bit,
            dwidth: OspiWidth::NONE,
            dummy: DummyCycles::_0,
            ..Default::default()
        };

        self.ospi.command(&transaction).await.unwrap();

        Timer::after_millis(20).await;

        // READ JEDEC ID
        let transaction: TransferConfig = TransferConfig {
            instruction: Some(FlashCommand::CMD_RDID as u32),
            iwidth: OspiWidth::SING,
            adwidth: OspiWidth::NONE,
            adsize: AddressSize::_24bit,
            dwidth: OspiWidth::SING,
            dummy: DummyCycles::_0,
            ..Default::default()
        };

        let mut buffer = [0u8; 3];
        self.ospi.blocking_read(&mut buffer, transaction).unwrap();

        debug!("FLASH JEDEC ID: {=[u8]:x}", buffer);

        // Read status register
        let mut status_reg_val = [0u16; 1];
        let transaction: TransferConfig = TransferConfig {
            instruction: Some(FlashCommand::CMD_RDSR as u32),
            iwidth: OspiWidth::SING,
            adwidth: OspiWidth::NONE,
            adsize: AddressSize::_24bit,
            dwidth: OspiWidth::SING,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.ospi.blocking_read(&mut status_reg_val, transaction).unwrap();

        // Enable Quad mode
        status_reg_val[0] |= 1 << 6;

        // Write it back
        let transaction: TransferConfig = TransferConfig {
            instruction: Some(FlashCommand::CMD_WRSR as u32),
            iwidth: OspiWidth::SING,
            adwidth: OspiWidth::NONE,
            adsize: AddressSize::_24bit,
            dwidth: OspiWidth::SING,
            dummy: DummyCycles::_0,
            ..Default::default()
        };
        self.ospi.blocking_write(&mut status_reg_val, transaction).unwrap();

        while pac::OCTOSPI1.sr().read().busy() {
            core::hint::spin_loop();
        }

        let read_config: TransferConfig = TransferConfig {
            instruction: Some(FlashCommand::CMD_READ as u32),
            iwidth: OspiWidth::SING,
            adwidth: OspiWidth::QUAD,
            adsize: AddressSize::_24bit,
            dwidth: OspiWidth::QUAD,
            dummy: DummyCycles::_6,
            ..Default::default()
        };

        let write_config: TransferConfig = TransferConfig {
            instruction: Some(FlashCommand::CMD_PP as u32),
            iwidth: OspiWidth::SING,
            adwidth: OspiWidth::QUAD,
            adsize: AddressSize::_24bit,
            dwidth: OspiWidth::QUAD,
            dummy: DummyCycles::_0,
            ..Default::default()
        };

        self.ospi.enable_memory_mapped_mode(read_config, write_config).unwrap();

        while pac::OCTOSPI1.sr().read().busy() {
            core::hint::spin_loop();
        }

        Timer::after_millis(20).await;
    }
}
