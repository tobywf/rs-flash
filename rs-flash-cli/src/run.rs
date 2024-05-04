// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::elf::FlashTable;
use color_eyre::eyre::{bail, eyre, Result};
use ram_probe_rs::defmt::DefmtDecoder;
use ram_probe_rs::probe_rs::rtt::UpChannel;
use ram_probe_rs::probe_rs::{MemoryInterface as _, Session};
use ram_probe_rs::run::{init_cpu, setup_rtt, DefmtOpts};
use std::io::{Read as _, Write as _};
use std::time::{Duration, Instant};

pub(crate) enum FlashData {
    Dump(std::fs::File),
    Load(std::fs::File),
}

pub(crate) struct FlashRunner<'opts> {
    decoder: DefmtDecoder<'opts>,
    defmt: UpChannel,
    flash_table: FlashTable,
    flash_data: FlashData,
    count: usize,
    timeout: Duration,
    erase_timeout: Option<Duration>,
}

impl<'opts> FlashRunner<'opts> {
    pub(crate) fn new(
        session: &mut Session,
        opts: &'opts DefmtOpts<'_>,
        flash_table: FlashTable,
        flash_data: FlashData,
        timeout: Duration,
        erase_timeout: Duration,
    ) -> Result<Self> {
        init_cpu(session, &opts.segments, &opts.vector_table, opts.timeout)?;

        let mut rtt = setup_rtt(session, opts.rtt_addr, opts.retries)?;

        let defmt = rtt
            .up_channels()
            .take(0)
            .ok_or_else(|| eyre!("RTT up channel 0 not found"))?;

        let decoder = DefmtDecoder::new(&opts.defmt, "target");

        Ok(Self {
            decoder,
            defmt,
            flash_table,
            flash_data,
            count: 0,
            timeout,
            erase_timeout: Some(erase_timeout),
        })
    }

    pub(crate) fn run(&mut self, session: &mut Session) -> Result<()> {
        let mut was_halted = false;

        loop {
            self.poll(session)?;

            let mut core = session.core(0)?;
            let is_halted = core.core_halted()?;

            if is_halted && was_halted {
                return Ok(());
            }
            was_halted = is_halted;
        }
    }

    pub(crate) fn poll(&mut self, session: &mut Session) -> Result<()> {
        let mut read_buf = [0; 1024];

        if self.count < self.flash_table.flash_size {
            // Display progress.
            let ft = &self.flash_table;
            let chunks = ft.flash_size / ft.buffer_size;
            let chunk = (self.count / ft.buffer_size) + 1;
            log::info!("chunk {} / {} (at 0x{:08x})", chunk, chunks, self.count);

            let mut core = session.core(0)?;

            match &mut self.flash_data {
                FlashData::Dump(file) => {
                    log::debug!("waiting for chunk to become available");
                    let deadline = Instant::now() + self.timeout;
                    loop {
                        // Wait for signal that the buffer is ready to be read.
                        let control = core.read_word_32(ft.control_addr)?;
                        log::trace!("control: {}", control);
                        if control == 1 {
                            break;
                        }
                        // In the meantime, pump the defmt output.
                        let n = self.defmt.read(&mut core, &mut read_buf)?;
                        log::trace!("defmt bytes: {}", n);
                        if n > 0 {
                            self.decoder.decode(&read_buf[..n])?;
                        }
                        // Or time out.
                        if Instant::now() > deadline {
                            bail!("Time out");
                        }
                    }

                    log::debug!("reading chunk from target (offset 0x{:08x})", self.count);
                    // Read chunk from target.
                    let mut buf = vec![0; ft.buffer_size];
                    core.read(ft.buffer_addr, &mut buf)?;
                    // Write chunk to file.
                    file.write_all(&buf)?;
                    // Signal target to read the next chunk.
                    core.write_word_32(ft.control_addr, 0)?;
                    self.count += buf.len();
                }
                FlashData::Load(file) => {
                    log::debug!("writing chunk to target (offset 0x{:08x})", self.count);
                    // Read chunk from file.
                    let mut buf = vec![0; ft.buffer_size];
                    file.read_exact(&mut buf)?;
                    // Write chunk to target.
                    core.write(ft.buffer_addr, &buf)?;
                    // Signal target to write the current chunk.
                    core.write_word_32(ft.control_addr, 1)?;
                    self.count += buf.len();

                    log::debug!("waiting for chunk to become committed");
                    let timeout = self.erase_timeout.take().unwrap_or(self.timeout);
                    let deadline = Instant::now() + timeout;
                    loop {
                        // Wait for signal that the buffer is ready to be written again.
                        let control = core.read_word_32(ft.control_addr)?;
                        log::trace!("control: {}", control);
                        if control == 0 {
                            break;
                        }
                        // In the meantime, pump the defmt output.
                        let n = self.defmt.read(&mut core, &mut read_buf)?;
                        log::trace!("defmt: {}", n);
                        if n > 0 {
                            self.decoder.decode(&read_buf[..n])?;
                        }
                        // Or time out.
                        if Instant::now() > deadline {
                            bail!("Time out");
                        }
                    }
                }
            }
        } else {
            let mut core = session.core(0)?;
            let n = self.defmt.read(&mut core, &mut read_buf)?;
            if n > 0 {
                self.decoder.decode(&read_buf[..n])?;
            }
        }

        Ok(())
    }
}
