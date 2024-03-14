cd rtray-sys
bindgen --experimental --wrap-static-fns tray/tray.h --wrap-static-fns-path wrapper.c -o src/tray.rs
cd ..