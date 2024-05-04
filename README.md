# rs-flash

`rs-flash` is a proof of concept for in-circuit dumping or loading external flash using RAM-only programs on the target platform, and a CLI on the host.

**WARNING**: `rs-flash` is unsupported and unmaintained. I don't have the time or knowledge to support a project like this. I'm only releasing this because I hope the Rust embedded eco-system will eventually have this capability. There is no guarantee any of this works; it was mainly a learning experience.

`rs-flash` builds on my other proof of concept [`ram-probe-rs`](https://github.com/tobywf/ram-probe-rs) that provides "flashing"/downloading RAM-only programs similar to [`probe-rs`](https://github.com/probe-rs/probe-rs). Currently, only ARM Cortex-M targets are supported.

## How it works

### RAM-only program

This is a small program that - via a linker script - is configured to fit into RAM only. By using the `rs_flash::flash_interface!()` macro, a host/target interface is set up, and flashing information (the total flash size, the transfer buffer size, and the operation mode/direction) is exported (as ELF symbols/sections).

This program can almost do whatever it wants (as long as the host/target interface is maintained), which enables it to interface with basically any peripheral. Currently, there is another restriction that the program must use [`defmt`](https://github.com/knurling-rs/defmt) for logging. This is required, even if no log messages are emitted.

### CLI

The examples are very specific. The dumping operation looks like this:

```bash
cd dump-spi-flash/
cargo build  # --release is also possible
cd ../rs-flash-cli/
cargo run -- --chip 'STM32F103ZE' ../dump-spi-flash/target/thumbv7em-none-eabihf/debug/dump
```

The loading operation looks like this:

```bash
cd load-spi-flash/
cargo build  # --release is also possible
cd ../rs-flash-cli/
cargo run -- --chip 'STM32F103ZE' ../load-spi-flash/target/thumbv7em-none-eabihf/debug/load --data ../firmware/mod.bin
```

The host/CLI and target/defmt logging is output to stdout, and can be configured via `RUST_LOG`. For dumping, the data is automatically written to `dump.bin`. For loading, the data is read from the file specified with `--data`.

### Dump (read)

Example run:

```shell
$ cargo run -- --chip 'STM32F103ZE' ../dump-spi-flash/target/thumbv7em-none-eabihf/debug/dump
[...]
 INFO  ram_probe_rs::elf > segment at 0x20003920 is empty, skipping
 INFO  ram_probe_rs::run > writing ram
 INFO  ram_probe_rs::run > wrote ram
 INFO  rs_flash::run     > chunk 1 / 512 (at 0x00000000)
 INFO  target            > init
 INFO  target            > dumping...
 INFO  target            > chunk 1 / 512 (at 0x00000000)
 INFO  target            > chunk 2 / 512 (at 0x00008000)
 INFO  rs_flash::run     > chunk 2 / 512 (at 0x00008000)
 INFO  rs_flash::run     > chunk 511 / 512 (at 0x00ff0000)
 INFO  target            > chunk 511 / 512 (at 0x00ff0000)
 INFO  rs_flash::run     > chunk 512 / 512 (at 0x00ff8000)
 INFO  target            > chunk 512 / 512 (at 0x00ff8000)
 INFO  target            > done.
```

### Load (write)

Example run:

```shell
$ cargo run -- --chip 'STM32F103ZE' ../load-spi-flash/target/thumbv7em-none-eabihf/debug/load --data ../firmware/mod.bin
[...]
 INFO  ram_probe_rs::elf > segment at 0x20003920 is empty, skipping
 INFO  ram_probe_rs::run > writing ram
 INFO  ram_probe_rs::run > wrote ram
 INFO  rs_flash::run     > chunk 1 / 512 (at 0x00000000)
 INFO  target            > init
 INFO  target            > erasing chip...
 INFO  target            > loading...
 INFO  target            > chunk 1 / 512 (at 0x00000000)
 INFO  target            > chunk 2 / 512 (at 0x00008000)
 INFO  rs_flash::run     > chunk 2 / 512 (at 0x00008000)
[...]
 INFO  rs_flash::run     > chunk 511 / 512 (at 0x00ff0000)
 INFO  target            > chunk 511 / 512 (at 0x00ff0000)
 INFO  rs_flash::run     > chunk 512 / 512 (at 0x00ff8000)
 INFO  target            > chunk 512 / 512 (at 0x00ff8000)
 INFO  target            > done.
```

### Verify

A verify implementation is not supported. Just dump the flash and compare it on the host.

## Components

* The `rs-flash` crate contains a to set up the host/target interface and export the necessary information for the CLI to automatically detect the flash and buffer sizes, as well as the operation mode/direction (dump i.e. target to host, or load i.e. host to target). RAM-only dumping or loading programs should use this.
* The `rs-flash-cli` crate implements a CLI for "flashing"/downloading RAM-only dumping or loading programs to a target, and automatic data transfer based on the exported information in the programs.
* The `skeleton-code` directory provides incomplete code as a starting point to implementing RAM-only dumping or loading programs.
* The `dump-spi-flash` contains an example implementation of a RAM-only dumping program for a specific target (GD32F307VE), and a specifically set up external SPI flash. It is mainly for reference unless you have the exact target platform.
* The `load-spi-flash` contains an example implementation of a RAM-only loading program for a specific target (GD32F307VE), and a specifically set up external SPI flash. It is mainly for reference unless you have the exact target platform.

## License

### rs-flash-cli and rs-flash

As the `rs-flash-cli` and `rs-flash` crates are heavily inspired by [`teleprobe`](https://github.com/embassy-rs/teleprobe) and based on [`probe-rs`](https://github.com/probe-rs/probe-rs), they are licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](rs-flash-cli/LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](rs-flash-cli/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### dump-spi-flash and load-spi-flash

As a large part of the `dump-spi-flash` and `load-spi-flash` crates are based on [`turbo-resin`](https://github.com/nviennot/turbo-resin) and [reverse engineering the Anycubic Photon Mono 4K](https://github.com/nviennot/reversing-mono4k), they are licensed as:

- GPL-3.0-or-later ([LICENSE](dump-spi-flash/LICENSE), [LICENSE](load-spi-flash/LICENSE) or <https://opensource.org/license/gpl-3-0>)

### skeleton-code

The `skeleton-code` directory contains code for implementing new dumping or loading RAM-only programs for new targets, and is my own work. That work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](skeleton-code/LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](skeleton-code/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
