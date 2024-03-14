use rtray::{Tray, TrayMenu};

fn main() {
    let mut tray = Tray::new(
        "icon.ico",
        &[
            TrayMenu::new("Say Hello", |_| println!("Hello, rtray!")),
            TrayMenu::new("Exit", |_| rtray::tray_exit()),
        ],
    );

    rtray::tray_init(&mut tray).expect("Failed to create tray icon!");
    while rtray::tray_loop(true) {}
}
