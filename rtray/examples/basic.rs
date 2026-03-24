use rtray::{Tray, TrayMenu};

fn main() {
    let _tray = Tray::new(
        "icon.ico",
        &[
            TrayMenu::new("Set checked!", |tray, menu| {
                menu.set_checked(!menu.is_checked());
                tray.update();
            }),
            TrayMenu::new("Set disabled!", |tray, menu| {
                menu.set_disabled(!menu.is_disabled());
                tray.update();
            }),
            TrayMenu::new("Set text!", |tray, menu| {
                menu.set_text("New Text");
                tray.update();
            }),
            TrayMenu::new("Exit", |_, _| rtray::tray_exit()),
        ],
    );

    while rtray::tray_loop(true) {}
}
