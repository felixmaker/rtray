use rtray::{Tray, TrayMenu};

enum Massage {
    SayHello,
    Exit,
}

fn main() {
    let (s, r) = std::sync::mpsc::channel();

    let say_hello = TrayMenu::new("Say Hello", {
        let s = s.clone();
        move |_| s.send(Massage::SayHello).unwrap()
    });

    let quit = TrayMenu::new("Exit", {
        let s = s.clone();
        move |_| s.send(Massage::Exit).unwrap()
    });

    let mut tray = Tray::new("icon.ico", &[say_hello, quit]);
    // let mut tray_2 = Tray::new("icon.ico", &[quit]);

    rtray::tray_init(&mut tray).expect("Failed to create tray icon!");

    while rtray::tray_loop(false) {
        if let Ok(msg) = r.try_recv() {
            match msg {
                Massage::SayHello => println!("Hello, tray"),
                Massage::Exit => rtray::tray_exit()
            }
        }
    }
}
