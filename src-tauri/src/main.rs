// Keep both debug and release desktop bundles independent from a console window.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    manga_shelf_lib::run()
}
