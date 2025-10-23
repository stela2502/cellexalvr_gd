//build.rs
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let godot_dir = out_dir.join("../godot/extensions/printforge3d");

    fs::create_dir_all(&godot_dir).unwrap();

    #[cfg(target_os = "linux")]
    let lib_name = "libprintforge3d.so";
    #[cfg(target_os = "windows")]
    let lib_name = "printforge3d.dll";
    #[cfg(target_os = "macos")]
    let lib_name = "libprintforge3d.dylib";

    let src = out_dir.join("target/release").join(lib_name);
    let dst = godot_dir.join(lib_name);

    println!("cargo:rerun-if-changed=src/lib.rs");

    if src.exists() {
        println!("Copying {:?} → {:?}", src, dst);
        fs::copy(&src, &dst).expect("Failed to copy library to Godot extension dir");
    } else {
        println!("⚠️ Built library not found at {:?}", src);
    }
}