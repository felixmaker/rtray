cd rtray-sys
copy tray\tray.h src\tray.h
cd src

bindgen --experimental --wrap-static-fns tray.h --wrap-static-fns-path wrapper.c --no-layout-tests --allowlist-function tray_.* -o tray.rs
