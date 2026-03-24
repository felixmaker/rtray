use std::os::raw::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CTray {
    pub icon: *mut c_char,
    pub menu: *mut CTrayMenu,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CTrayMenu {
    pub text: *mut c_char,
    pub disabled: c_int,
    pub checked: c_int,
    pub cb: ::std::option::Option<unsafe extern "C" fn(arg1: *mut CTrayMenu)>,
    pub context: *mut c_void,
    pub submenu: *mut CTrayMenu,
}
extern "C" {
    #[link_name = "tray_update__extern"]
    pub fn tray_update(tray: *mut CTray);
 
    #[link_name = "tray_init__extern"]
    pub fn tray_init(tray: *mut CTray) -> c_int;
 
    #[link_name = "tray_loop__extern"]
    pub fn tray_loop(blocking: c_int) -> c_int;
 
    #[link_name = "tray_exit__extern"]
    pub fn tray_exit();
}
