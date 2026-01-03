use std::fs::File;
use std::io::copy;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let suffix = match (target_os.as_str(), target_arch.as_str()) {
        ("windows", "x86_64") => "windows-x64.exe",
        ("windows", "aarch64") => "windows-arm64.exe",
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("macos", "aarch64") => "macos-arm64",
        ("macos", "x86_64") => "macos-x64",
        _ => panic!(
            "Unsupported target: {} {}. Please report this as an issue!",
            target_os, target_arch
        ),
    };

    let url = format!(
        "https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-{}",
        suffix
    );

    let dest_path = Path::new(&out_dir).join("tailwindcss_bin");

    if !dest_path.exists() {
        println!(
            "cargo:warning=Downloading Tailwind CLI for {}...",
            target_os
        );
        let mut response = reqwest::blocking::get(url).expect("Failed to download Tailwind");
        let mut dest = File::create(&dest_path).expect("Failed to create bin file");
        copy(&mut response, &mut dest).expect("Failed to copy binary");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
