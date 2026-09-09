// Hide the console window on Windows release builds.
// In debug builds we keep the console for logging.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    masternote_lib::run();
}
