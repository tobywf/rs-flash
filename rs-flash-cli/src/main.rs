// SPDX-License-Identifier: MIT OR Apache-2.0

mod elf;
mod run;

use color_eyre::eyre::{bail, Context as _, Result};
use color_eyre::{Section as _, SectionExt as _};
use ram_probe_rs::probe_rs::config::get_target_by_name;
use ram_probe_rs::run::DefmtOpts;
use ram_probe_rs::session::{connect, ProbeArgs};
use rs_flash::Direction;
use run::{FlashData, FlashRunner};
use std::time::Duration;

#[derive(Debug, Clone, clap::Parser)]
#[command(version = "1.0", about = "Flash and run an ELF program from RAM")]
struct Args {
    /// The path to the ELF file to flash and run from RAM
    path: String,

    #[clap(flatten)]
    probe: ProbeArgs,

    /// When running a loader program, the data to load
    #[clap(long)]
    data: Option<String>,

    /// The timeout for the erase step, in seconds
    #[clap(long, default_value_t = 60 * 5)]
    erase_timeout: u64,

    /// The timeout for the dump/load steps, in seconds
    #[clap(long, default_value_t = 10)]
    timeout: u64,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    try_init_logging()?;

    use clap::Parser as _;
    let args = Args::parse();

    let erase_timeout = Duration::from_secs(args.erase_timeout);
    let timeout = Duration::from_secs(args.timeout);

    log::debug!("target `{}`", args.probe.chip);
    let target = get_target_by_name(&args.probe.chip)?;

    log::debug!("reading `{}`", args.path);
    let data = std::fs::read(&args.path)
        .wrap_err("failed to read ELF file")
        .with_section(|| args.path.clone().header("Path"))?;

    let (segments, rtt_addr, vector_table, flash_table, defmt) = elf::parse_elf(&data, &target)?;

    let opts = DefmtOpts::with_defaults(&segments, rtt_addr, &vector_table, &defmt);

    let flash_data = match flash_table.direction {
        Direction::Dump => {
            if args.data.is_some() {
                bail!("`--data` is specified, but ELF file dumps data");
            }
            let file = std::fs::File::create("dump.bin").wrap_err("failed to open dump file")?;
            FlashData::Dump(file)
        }
        Direction::Load => match args.data.as_deref() {
            Some(path) => {
                let file = std::fs::File::open(path).wrap_err("failed to open load file")?;
                FlashData::Load(file)
            }
            None => bail!("`--data` not specified, but ELF file loads data"),
        },
    };

    let mut session = connect(&args.probe, target)?;
    let mut runner = FlashRunner::new(
        &mut session,
        &opts,
        flash_table,
        flash_data,
        timeout,
        erase_timeout,
    )?;
    runner.run(&mut session)?;
    Ok(())
}

fn try_init_logging() -> Result<()> {
    let mut builder = pretty_env_logger::formatted_builder();
    match std::env::var("RUST_LOG") {
        Ok(filters) => {
            builder.parse_filters(&filters);
        }
        Err(std::env::VarError::NotPresent) => {
            builder.filter_level(log::LevelFilter::Warn);
            // app output
            builder.filter_module("rs_flash", log::LevelFilter::Info);
            // ram-probe-rs
            builder.filter_module("ram_probe_rs", log::LevelFilter::Info);
            // target output
            builder.filter_module("target", log::LevelFilter::Info);
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            bail!("`RUST_LOG` is not unicode");
        }
    }
    Ok(builder.try_init()?)
}
