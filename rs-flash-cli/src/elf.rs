// SPDX-License-Identifier: MIT OR Apache-2.0

use color_eyre::eyre::{bail, eyre, OptionExt as _, Result};
use ram_probe_rs::defmt::DefmtInfo;
use ram_probe_rs::elf::{parse_vector_table, Parser, Segments, VectorTable};
use ram_probe_rs::probe_rs::Target;
use rs_flash::Direction;

#[derive(Debug, Clone)]
pub(crate) struct FlashTable {
    pub(crate) direction: Direction,
    pub(crate) flash_size: usize,
    pub(crate) buffer_size: usize,
    pub(crate) buffer_addr: u64,
    pub(crate) control_addr: u64,
}

pub(crate) fn parse_elf<'data>(
    data: &'data [u8],
    target: &Target,
) -> Result<(Segments<'data>, u32, VectorTable, FlashTable, DefmtInfo)> {
    let elf = Parser::new(&data)?;

    let segments = elf.ram_loadable_segments(&target)?;

    let mut rtt_addr = None;
    let mut buffer_addr = None;
    let mut control_addr = None;

    for (name, addr) in elf.named_symbols() {
        log::trace!("ELF symbol `{}` at 0x{:08x}", name, addr);
        match name {
            "_SEGGER_RTT" => rtt_addr = Some(addr),
            "_RS_FLASH_BUFFER" => buffer_addr = Some(addr),
            "_RS_FLASH_CONTROL" => control_addr = Some(addr),
            _ => {}
        }
    }
    let rtt_addr = rtt_addr.ok_or_eyre("RTT symbol not found")?;
    log::debug!("RTT address 0x{:08x}", rtt_addr);
    let buffer_addr = buffer_addr.ok_or_eyre("Flash buffer symbol not found")?;
    log::debug!("Buffer address 0x{:08x}", buffer_addr);
    let control_addr = control_addr.ok_or_eyre("Flash control symbol not found")?;
    log::debug!("Control address 0x{:08x}", control_addr);

    let mut vector_table = None;
    let mut flash_table = None;
    for (name, section) in elf.named_sections() {
        use ram_probe_rs::elf::ObjectSection as _;
        log::trace!(
            "ELF section `{}` at 0x{:08x} ({} bytes)",
            name,
            section.address(),
            section.size()
        );
        match name {
            ".vector_table" => {
                vector_table = Some(parse_vector_table(section)?);
            }
            ".rs-flash" => {
                flash_table = Some(parse_flash_table(&section, buffer_addr, control_addr)?);
            }
            _ => {}
        }
    }

    let vector_table = vector_table.ok_or_eyre("vector table section not found")?;
    log::debug!("{:?}", vector_table);
    let flash_table = flash_table.ok_or_eyre("flash table section not found")?;
    log::debug!("{:?}", flash_table);

    let defmt = DefmtInfo::new(&data)?.ok_or_eyre("defmt info not found")?;
    if defmt.is_missing_debug() {
        log::warn!("defmt locations empty, is the ELF compiled with `debug = 2`?");
    }
    Ok((segments, rtt_addr, vector_table, flash_table, defmt))
}

/// Parse the flash table section.
fn parse_flash_table(
    section: &ram_probe_rs::elf::ElfSection<'_, '_>,
    buffer_addr: u32,
    control_addr: u32,
) -> Result<FlashTable> {
    use ram_probe_rs::elf::object::ObjectSection as _;

    let size = section.size() as u32;
    if size != 3 * 4 {
        bail!("flash table is wrong size");
    }

    let data = section.data()?;
    assert_eq!(
        data.len(),
        3 * 4,
        "data length {} is different than size {}",
        data.len(),
        size
    );
    let flash_size = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let buffer_size = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
    let direction = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let direction = Direction::from_u32(direction)
        .ok_or_else(|| eyre!("Invalid flash table direction 0x{:08x}", direction))?;

    Ok(FlashTable {
        direction,
        flash_size,
        buffer_size,
        buffer_addr: buffer_addr as _,
        control_addr: control_addr as _,
    })
}
