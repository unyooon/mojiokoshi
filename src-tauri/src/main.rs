#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = mojiokoshi_lib::run() {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}
