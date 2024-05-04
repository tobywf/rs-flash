// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![no_main]

use core::sync::atomic::Ordering;
use defmt_rtt as _;
use panic_probe as _;
use rs_flash::flash_interface;

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
    defmt::info!("init");

    todo!("Initialize peripherals");
    let flash = ();

    // --- Erase the entire flash.
    defmt::info!("erasing chip...");

    todo!("Erase the entire flash.");
    flash.erase_all().unwrap();

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

            todo!("Write the next chunk into the flash");
            flash.write(offset, buf).unwrap();
        }
        // Signal buffer is ready to be written.
        unsafe { RS_FLASH_CONTROL.store(0, Ordering::SeqCst) };
    }

    // --- Done.
    defmt::info!("done.");
    cortex_m::asm::bkpt();
    cortex_m::asm::udf()
}
