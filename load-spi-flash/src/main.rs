// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![no_main]

use core::sync::atomic::Ordering;
use defmt_rtt as _;
use panic_probe as _;
use rs_flash::flash_interface;

use spi_memory::prelude::*;
use spi_memory::series25::Flash;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{pac, spi};

/// The size of the flash in bytes. CHANGE ME!
const FLASH_SIZE: usize = 16 * 1024 * 1024;
/// The size of the RAM buffer in bytes. CHANGE ME!
///
/// The RAM buffer size must divide the flash size without remainder.
const BUFFER_SIZE: usize = 32 * 1024;

flash_interface!(FLASH_SIZE, BUFFER_SIZE, load);

/// Load example.
#[cortex_m_rt::entry]
fn main() -> ! {
    // --- Initialize peripherals.
    // This code is inline due to the horrible typing.

    defmt::info!("init");
    let dp = pac::Peripherals::take().unwrap();
    // Internal flash memory.
    let mut flash = dp.FLASH.constrain();
    // Reset & Clock Control.
    let rcc = dp.RCC.constrain();
    // Initialize the device to run at 48Mhz using the 8Mhz crystal on
    // the PCB instead of the internal oscillator.
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .freeze(&mut flash.acr);

    // Use GPIO B for external flash SPI access.
    let mut gpiob = dp.GPIOB.split();

    let cs = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14.into_floating_input(&mut gpiob.crh);
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

    // Configure SPI for external flash access.
    let spi = spi::Spi::spi2(
        dp.SPI2,
        (sck, miso, mosi),
        spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        },
        clocks.pclk1(), // Run as fast as we can. The flash chip can go up to 133Mhz.
        clocks,
    );
    let mut ex_flash = Flash::init(spi, cs).unwrap();

    // --- Erase the entire flash.
    defmt::info!("erasing chip...");
    ex_flash.erase_all().unwrap();

    // --- Load the entire flash.
    defmt::info!("loading...");

    for (offset, chunk) in (0..FLASH_SIZE).step_by(BUFFER_SIZE).zip(1..) {
        defmt::info!("chunk {} / {} (at 0x{:08x})", chunk, _CHUNKS, offset);
        // Spin until the host has written the buffer.
        while unsafe { RS_FLASH_CONTROL.load(Ordering::SeqCst) } == 0 {
            cortex_m::asm::nop();
        }
        // Write the next chunk into the flash.
        {
            let buf = unsafe { &mut *RS_FLASH_BUFFER.as_mut_ptr() };
            let addr = offset as _;
            ex_flash.write_bytes(addr, buf).unwrap();
        }
        // Signal buffer is ready to be written.
        unsafe { RS_FLASH_CONTROL.store(0, Ordering::SeqCst) };
    }

    // --- Done.
    defmt::info!("done.");
    cortex_m::asm::bkpt();
    cortex_m::asm::udf()
}
