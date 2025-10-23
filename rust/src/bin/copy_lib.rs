//copy_lib.rs
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Determine current working directory (the Rust project root)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Path to compiled library
    let lib_name = if cfg!(target_os = "windows") {
        "rust_print_forge_3d.dll"
    } else if cfg!(target_os = "linux") {
        "librust_print_forge_3d.so"
    } else {
        "librust_print_forge_3d.dylib"
    };

    let src = manifest_dir.join("target/release").join(lib_name);

    // Destination in the Godot project
    let dst = manifest_dir
        .join("../godot/extensions/printforge3d/bin")
        .join(lib_name);

    println!("📦 Copying built library:");
    println!("    from: {:?}", src);
    println!("      to: {:?}", dst);

    // Create destination folder if missing
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    // Copy the file (with error handling)
    match fs::copy(&src, &dst) {
        Ok(_) => println!("✅ Copied successfully."),
        Err(e) => eprintln!("❌ Failed to copy: {}", e),
    }
}
