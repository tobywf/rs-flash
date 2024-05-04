// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
    // Get the output directory.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);
    // Ensure the output directory is in the linker search path.
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Copy `rs_flash.x` to the output directory.
    let path = out_dir.join("rs_flash.x");
    std::fs::write(&path, include_bytes!("rs_flash.x"))
        .unwrap_or_else(|e| panic!("Failed to write `{}`: {:?}", path.display(), e));
    // Ensure the build script is only re-run if the file is changed.
    println!("cargo:rerun-if-changed={}", path.display());
}
