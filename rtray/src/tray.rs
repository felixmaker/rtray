use std::{
    cell::RefCell, ffi::{CStr, CString, c_char, c_void}, pin::Pin, rc::Rc, sync::atomic::{AtomicBool, AtomicPtr, Ordering}
};

use rtray_sys::{CTray, CTrayMenu};

static TRAY: AtomicPtr<CTray> = AtomicPtr::new(std::ptr::null_mut());
static TRAY_INIT: AtomicBool = AtomicBool::new(false);

/// Updates tray icon and menu.
fn tray_update() {
    let tray = TRAY.load(Ordering::Relaxed);
    if tray != std::ptr::null_mut() {
        unsafe {
            rtray_sys::tray::tray_update(tray);
        }
    }
}

/// Loads a new tray.
pub fn tray_load(menu: &mut Tray) {
    TRAY.store(&mut *menu.inner.as_mut(), Ordering::Relaxed);
    if !TRAY_INIT.load(Ordering::Relaxed) {
        unsafe {
            rtray_sys::tray_init(TRAY.load(Ordering::Relaxed));
        }
    }
    TRAY_INIT.store(true, Ordering::Relaxed);
    tray_update();
}

/// Runs one iteration of the UI loop. Returns false if `exit()` has been called.
pub fn tray_loop(blocking: bool) -> bool {
    if !TRAY_INIT.load(Ordering::Relaxed) {
        return false
    }

    let blocking = if blocking { 1 } else { 0 };
    unsafe { rtray_sys::tray::tray_loop(blocking) != -1 }
}

/// Terminates UI loop.
pub fn tray_exit() {
    unsafe { rtray_sys::tray::tray_exit() }
}

#[repr(C)]
/// A tray with an icon and a menu
pub struct Tray {
    inner: Pin<Box<CTray>>,
    icon: CString,
    menu: Box<[TrayMenu]>,
}

impl Tray {
    /// Creates a tray with an icon and a menu.
    /// 
    /// If no tray is initialized, it will be initialized and loaded.
    pub fn new<T>(icon: &str, menus: T) -> Self
    where
        T: Into<Vec<TrayMenu>>,
    {
        let icon = CString::new(icon).unwrap();
        let mut menu: Vec<TrayMenu> = menus.into();
        menu.push(TrayMenu::null()); // Tray menu is null terminated.

        let mut menu = menu.into_boxed_slice();

        let inner = CTray {
            icon: icon.as_ptr() as *mut c_char,
            menu: menu.as_mut_ptr() as *mut CTrayMenu,
        };

        let mut tray = Self { inner: Box::pin(inner), icon, menu };
        tray_load(&mut tray);
        tray
    }
}

#[repr(C)]
// #[derive(Clone)]
struct TrayMenuContext {
    callback: Option<Box<dyn FnMut(&mut TrayMenu)>>,
    submenu: Box<[TrayMenu]>,
    text: CString,
}

impl TrayMenuContext {
    fn new<I, T>(text: &str, callback: I, submenu: T) -> Self
    where
        I: FnMut(&mut TrayMenu) + 'static,
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
    let context = unsafe { &mut *(menu.inner.context as *mut RefCell<TrayMenuContext>) };
    if let Some(callback) = &mut context.get_mut().callback {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(menu)));
    }
}

impl TrayMenu {
    /// Returns a menu with menu text, menu checked and disabled (grayed) flags and a callback.
    pub fn new_ex<T, F>(text: &str, disabled: bool, checked: bool, callback: F, submenu: T) -> Self
    where
        T: Into<Vec<TrayMenu>>,
        F: FnMut(&mut TrayMenu) + 'static,
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
        let text = CString::new(s).unwrap();
        let context = unsafe { &*(self.inner.context as *const RefCell<TrayMenuContext>) };
        self.inner.text = text.as_ptr() as _;
        context.borrow_mut().text = text;

        tray_update();
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
    pub fn new<F: FnMut(&mut TrayMenu) + 'static>(text: &str, cb: F) -> Self {
        Self::new_ex(text, false, false, cb, Vec::<TrayMenu>::new())
    }

    /// Sets menu checked flag.
    pub fn set_checked(&mut self, checked: bool) {
        self.inner.checked = checked as i32;

        tray_update();
    }

    /// Returns menu checked flag.
    pub fn is_checked(&self) -> bool {
        self.inner.checked != 0
    }

    /// Sets menu disabled (grayed) flag.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.inner.disabled = disabled as i32;

        tray_update();
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
        if !self.inner.context.is_null() {
            // NONE MENU SHOULD NOT DROPPED.
            let _ = unsafe { Rc::from_raw(self.inner.context as *const RefCell<TrayMenuContext>) };
        }
    }
}
