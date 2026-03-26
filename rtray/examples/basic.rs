use rtray::{tray_exit, tray_loop, Tray, TrayMenu};

fn main() {
    let _tray = Tray::new(
        "icon.ico",
        &[
            TrayMenu::new("Hello", |_| {
                println!("Hello, rtray!");
            }),
            TrayMenu::new("Checked", |menu| {
                menu.set_checked(!menu.is_checked());
            }),
            TrayMenu::new_ex("Disabled", true, false, |_| {}, &[]),
            TrayMenu::new("-", |_| {}),
            TrayMenu::new_ex("Submenu", false, false, |_| {}, &[
                TrayMenu::new("First", |menu| {
                    println!("{} submenu clicked", menu.text());
                }),
                TrayMenu::new("Second", |menu| {
                    println!("{} submenu clicked", menu.text());
                }),
            ]),
            TrayMenu::new("Exit", |_| tray_exit()),
        ],
    );

    while tray_loop(true) {}
}
