use std::ffi::{c_char, c_void, CString};

use rtray_sys::tray::{tray, tray_menu};

fn tray_menu_null() -> tray_menu {
    tray_menu {
        text: std::ptr::null_mut(),
        disabled: 0,
        checked: 0,
        cb: None,
        context: std::ptr::null_mut(),
        submenu: std::ptr::null_mut(),
    }
}

fn tray_menu_vec_into_raw(menu: &[TrayMenu]) -> *mut tray_menu {
    let menu = menu.to_vec();
    let mut menu: Vec<tray_menu> = menu.iter().map(|x| x.inner.clone()).collect();
    if menu.len() > 0 {
        menu.push(tray_menu_null());
        let mut menu = std::mem::ManuallyDrop::new(menu);
        menu.as_mut_ptr()
    } else {
        std::ptr::null_mut()
    }
}

fn cstring_into_raw(s: &str) -> *mut c_char {
    let s = match CString::new(s) {
        Ok(v) => v,
        Err(r) => {
            let i = r.nul_position();
            CString::new(&r.into_vec()[0..i]).unwrap()
        }
    };
    s.into_raw()
}

/// A tray with an icon and a menu
///
/// Bindings to struct tray
#[derive(Debug, Clone)]
pub struct Tray {
    inner: tray,
}

impl Tray {
    /// Returns a tray with an icon and a menu.
    pub fn new(icon: &str, menu: &[TrayMenu]) -> Self {
        let inner = tray {
            icon: cstring_into_raw(icon),
            menu: tray_menu_vec_into_raw(menu),
        };
        Self { inner }
    }
}

/// A menu with menu text, menu checked and disabled (grayed) flags and a callback
///
/// Bindings to struct tray_menu
#[derive(Debug, Clone)]
pub struct TrayMenu {
    inner: tray_menu,
}

impl TrayMenu {
    /// Returns a menu with menu text, menu checked and disabled (grayed) flags and a callback.
    pub fn new_ex<F: FnMut(&mut Self) + 'static>(
        text: &str,
        disabled: bool,
        checked: bool,
        cb: F,
        submenu: &[TrayMenu],
    ) -> Self {
        let text = cstring_into_raw(text);
        let submenu = tray_menu_vec_into_raw(submenu);

        unsafe extern "C" fn shim(menu: *mut tray_menu) {
            let mut menu = TrayMenu { inner: *menu };

            let a = menu.inner.context as *mut Box<dyn FnMut(&mut TrayMenu)>;
            let f = &mut **a;
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut menu)));
        }

        let a: *mut Box<dyn FnMut(&mut Self)> = Box::into_raw(Box::new(Box::new(cb)));

        let inner = tray_menu {
            text,
            disabled: disabled as i32,
            checked: checked as i32,
            cb: Some(shim),
            context: a as *mut c_void,
            submenu,
        };

        return Self { inner };
    }

    /// Returns a menu with menu text, menu unchecked and enabled and a callback.
    pub fn new<F: FnMut(&mut Self) + 'static>(text: &str, cb: F) -> Self {
        Self::new_ex(text, false, false, cb, &[])
    }
}

/// Creates tray icon.
pub fn tray_init(tray: &mut Tray) -> Result<(), i32> {
    unsafe {
        match rtray_sys::tray::tray_init(&mut tray.inner) {
            -1 => Err(-1),
            _ => Ok(()),
        }
    }
}

/// Updates tray icon and menu.
pub fn tray_update(tray: &mut Tray) {
    unsafe {
        rtray_sys::tray::tray_update(&mut tray.inner);
    }
}

/// Runs one iteration of the UI loop. Returns false if `tray_exit()` has been called.
pub fn tray_loop(blocking: bool) -> bool {
    let blocking = if blocking { 1 } else { 0 };
    unsafe { rtray_sys::tray::tray_loop(blocking) != -1 }
}

/// Terminates UI loop.
pub fn tray_exit() {
    unsafe { rtray_sys::tray::tray_exit() }
}
