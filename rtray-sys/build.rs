use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/tray.h");

    let target_macro = match target_os.as_str() {
        "windows" => "TRAY_WINAPI",
        "linux" => "TRAY_APPINDICATOR",
        "macos" => "TRAY_APPKIT",
        _ => panic!("Target {target_os} are not supported!"),
    };

    cc::Build::new()
        .define(target_macro, "1")
        .file(concat!("src/wrapper.c"))
        .compile("tray");

    match target_os.as_str() {
        "windows" => {
            println!("cargo:rustc-link-lib=User32");
            println!("cargo:rustc-link-lib=Shell32");
        }
        "linux" => {}
        "macos" => {}
        _ => panic!("Target {target_os} are not supported!"),
    }
}
