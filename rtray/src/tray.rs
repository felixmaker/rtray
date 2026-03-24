use std::{
    cell::RefCell,
    ffi::{c_char, c_void, CStr, CString},
    rc::Rc,
    sync::atomic::{AtomicPtr, Ordering},
};

use rtray_sys::{CTray, CTrayMenu};

static TRAY: AtomicPtr<TrayInner> = AtomicPtr::new(std::ptr::null_mut());

/// A tray with an icon and a menu
///
/// Bindings to struct tray
pub struct Tray {
    inner: *mut TrayInner,
}

impl Tray {
    /// Returns a tray with an icon and a menu.
    ///
    /// # Panics
    ///
    /// If one tray is already created.
    pub fn new<T>(icon: &str, menus: T) -> Self
    where
        T: Into<Vec<TrayMenu>>,
    {
        if TRAY.load(Ordering::Relaxed) != std::ptr::null_mut() {
            panic!("tray already created");
        }
        Self {
            inner: TrayInner::new(icon, menus) as *mut _,
        }
    }

    /// Updates tray icon and menu.
    pub fn update(&mut self) {
        unsafe {
            let tray = &mut *self.inner;
            rtray_sys::tray::tray_update(&mut tray.inner as *mut CTray); //TODO
        }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            if let Ok(_) = TRAY.compare_exchange(
                self.inner,
                std::ptr::null_mut(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                let _ = Box::from_raw(self.inner);
            }
        }
    }
}

#[repr(C)]
pub(crate) struct TrayInner {
    inner: CTray,
    menu: Box<[TrayMenu]>,
}

impl TrayInner {
    pub fn new<T>(icon: &str, menus: T) -> &'static mut Self
    where
        T: Into<Vec<TrayMenu>>,
    {
        let icon = CString::new(icon).unwrap();
        let mut menu: Vec<TrayMenu> = menus.into();
        menu.push(TrayMenu::null()); // Tray menu is null terminated.

        let mut menu = menu.into_boxed_slice();

        let inner = CTray {
            icon: icon.into_raw(),
            menu: menu.as_mut_ptr() as *mut CTrayMenu,
        };

        let tray = Self { inner, menu };
        let tray = Box::into_raw(Box::new(tray));
        TRAY.store(tray, Ordering::Relaxed);
        let tray = unsafe { &mut *TRAY.load(Ordering::Relaxed) };
        unsafe {
            rtray_sys::tray_init(&mut tray.inner);
        }

        tray
    }
}

impl Drop for TrayInner {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.inner.icon);
        }
    }
}

#[repr(C)]
// #[derive(Clone)]
struct TrayMenuContext {
    callback: Option<Box<dyn FnMut(&mut Tray, &mut TrayMenu)>>,
    submenu: Box<[TrayMenu]>,
    text: CString,
}

impl TrayMenuContext {
    fn new<I, T>(text: &str, callback: I, submenu: T) -> Self
    where
        I: FnMut(&mut Tray, &mut TrayMenu) + 'static,
        T: Into<Vec<TrayMenu>>,
    {
        let mut submenu: Vec<TrayMenu> = submenu.into();
        submenu.push(TrayMenu::null()); // Tray submenu is null terminated.
        Self {
            callback: Some(Box::new(callback)),
            submenu: submenu.into_boxed_slice(),
            text: CString::new(text).unwrap(),
        }
    }
}

/// A menu with menu text, menu checked and disabled (grayed) flags and a callback
///
/// Bindings to struct tray_menu
#[repr(transparent)]
pub struct TrayMenu {
    inner: CTrayMenu,
}

impl Clone for TrayMenu {
    fn clone(&self) -> Self {
        if !self.inner.context.is_null() {
            unsafe {
                Rc::increment_strong_count(self.inner.context as *const RefCell<TrayMenuContext>);
            }
        }
        Self {
            inner: CTrayMenu { ..self.inner },
        }
    }
}

extern "C" fn shim(menu: *mut CTrayMenu) {
    let menu = unsafe { &mut *(menu as *mut TrayMenu) };
    let tray = unsafe { std::mem::transmute(&mut TRAY.load(Ordering::Relaxed)) };
    let context = unsafe { &*(menu.inner.context as *const RefCell<TrayMenuContext>) };
    if let Some(callback) = &mut context.borrow_mut().callback {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(tray, menu)));
    }
}

impl TrayMenu {
    /// Returns a menu with menu text, menu checked and disabled (grayed) flags and a callback.
    pub fn new_ex<T, F>(text: &str, disabled: bool, checked: bool, callback: F, submenu: T) -> Self
    where
        T: Into<Vec<TrayMenu>>,
        F: FnMut(&mut Tray, &mut TrayMenu) + 'static,
    {
        let context = Rc::new(RefCell::new(TrayMenuContext::new(text, callback, submenu)));
        let submenu = if context.borrow().submenu.len() == 1 {
            std::ptr::null_mut()
        } else {
            context.borrow().submenu.as_ptr() as *mut CTrayMenu
        };
        let text = context.borrow().text.as_ptr() as *mut c_char;
        let context: *const RefCell<TrayMenuContext> = Rc::into_raw(context);

        let tray = TrayMenu {
            inner: CTrayMenu {
                text,
                disabled: disabled as i32,
                checked: checked as i32,
                cb: Some(shim),
                context: context as *mut c_void,
                submenu: submenu,
            },
        };

        tray
    }

    /// Sets menu text.
    pub fn set_text(&mut self, s: &str) {
        if !self.inner.text.is_null() {
            let _ = unsafe { CString::from_raw(self.inner.text) };
        }
        let text = CString::new(s).unwrap();
        self.inner.text = text.into_raw();
    }

    /// Returns menu text.
    pub fn text(&self) -> String {
        unsafe {
            CStr::from_ptr(self.inner.text)
                .to_string_lossy()
                .to_string()
        }
    }

    /// Returns a menu with menu text, menu unchecked and enabled and a callback.
    pub fn new<F: FnMut(&mut Tray, &mut TrayMenu) + 'static>(text: &str, cb: F) -> Self {
        Self::new_ex(text, false, false, cb, Vec::<TrayMenu>::new())
    }

    /// Sets menu checked flag.
    pub fn set_checked(&mut self, checked: bool) {
        self.inner.checked = checked as i32;
    }

    /// Returns menu checked flag.
    pub fn is_checked(&self) -> bool {
        self.inner.checked != 0
    }

    /// Sets menu disabled (grayed) flag.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.inner.disabled = disabled as i32;
    }

    /// Returns menu disabled (grayed) flag.
    pub fn is_disabled(&self) -> bool {
        self.inner.disabled != 0
    }

    pub(crate) fn null() -> TrayMenu {
        unsafe { std::mem::zeroed() }
    }
}

impl Drop for TrayMenu {
    fn drop(&mut self) {
        if !self.inner.context.is_null() { // NONE MENU SHOULD NOT DROPPED.
            let _ = unsafe { Rc::from_raw(self.inner.context as *const RefCell<TrayMenuContext>) };
        }
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
