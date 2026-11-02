use std::{env, fs, path::PathBuf};

// The MacDows NSIS package is embedded into the installer. scripts/build-installer.mjs puts it at
// payload/MacDows-nsis-setup.exe. Without it (dev builds, tests) a tiny stub is embedded instead and the
// installer refuses to install ("payload missing"), so a stub can never ship by accident.
fn main() {
    println!("cargo:rerun-if-changed=payload");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("payload.exe");
    let real = PathBuf::from("payload").join("MacDows-nsis-setup.exe");
    if real.exists() {
        fs::copy(&real, &out).unwrap();
    } else {
        fs::write(&out, b"MZ-macdows-payload-stub").unwrap();
    }
    tauri_build::build()
}
