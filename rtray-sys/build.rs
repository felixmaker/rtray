use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=tray/tray.h");

    let define = match target_os.as_str() {
        "windows" => "TRAY_WINAPI",
        "linux" => "TRAY_APPINDICATOR",
        "macos" => "TRAY_APPKIT",
        _ => panic!("Target {target_os} are not supported!"),
    };

    cc::Build::new()
        .define(define, "1")
        .file("wrapper.c")
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
