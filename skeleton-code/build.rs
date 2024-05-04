// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

macro_rules! copy {
    ($out_dir:ident, $name:literal) => {{
        // Copy file to the output directory.
        let path = $out_dir.join($name);
        std::fs::write(&path, include_bytes!($name))
            .unwrap_or_else(|e| panic!("Failed to write `{}`: {:?}", path.display(), e));
        // Ensure the build script is only re-run if the file is changed.
        println!("cargo:rerun-if-changed={}", path.display());
    }};
    ($out_dir:ident, $name:literal, link) => {{
        copy!($out_dir, $name);
        println!("cargo:rustc-link-arg=-T{}", $name);
    }};
}

fn main() {
    // Get the output directory.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);
    // Ensure the output directory is in the linker search path.
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Copy `memory.x` to the output directory.
    // `memory.x` is included by `link_ram.x`.
    copy!(out_dir, "memory.x");

    // Copy `link_ram.x` to the output directory.
    copy!(out_dir, "link_ram.x", link);

    // Add the rs_flash linker script.
    println!("cargo:rustc-link-arg=-Trs_flash.x");
    // Add the defmt linker script.
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // Set the defmt log level.
    println!("cargo:rustc-env=DEFMT_LOG=trace");
}
