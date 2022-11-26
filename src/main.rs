// hide the console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rust_chip::{setup_logging, App};

/// For when compiling to a native target.
///
/// Currently, this app does not support wasm32.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> color_eyre::Result<()> {
    setup_logging()?;

    let options = eframe::NativeOptions {
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        renderer: eframe::Renderer::Wgpu,
        follow_system_theme: true,
        ..Default::default()
    };

    eframe::run_native("Rust Chip", options, Box::new(|cc| Box::new(App::new(cc))));

    Ok(())
}
