// hide the console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod egui_ui_thread_waker;

use app::App;
use egui_ui_thread_waker::EguiUiThreadWaker;
use emulator::Emulator;

/// For when compiling to a native target.
///
/// Currently, this app does not support wasm32.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> color_eyre::Result<()> {
    setup_logging()?;

    let mut emulator = Emulator::new();
    let emulator_app_ref = emulator.clone();
    let emulator_bg_thread_ref = emulator.clone();

    let options = eframe::NativeOptions {
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        renderer: eframe::Renderer::Wgpu,
        follow_system_theme: true,
        ..Default::default()
    };

    eframe::run_native(
        "Rust Chip",
        options,
        Box::new(move |cc| {
            let emu_egui_context = cc.egui_ctx.clone();

            // Start the emulator in its background thread
            emulator_bg_thread_ref
                .start(EguiUiThreadWaker::from(emu_egui_context))
                .unwrap();

            Box::new(App::new(cc, &emulator_app_ref))
        }),
    );

    emulator.stop();

    Ok(())
}

pub fn setup_logging() -> color_eyre::Result<()> {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

    color_eyre::install()?;

    let default_log_level = if cfg!(debug_assertions) {
        LevelFilter::INFO
    } else {
        LevelFilter::WARN
    };

    let reg = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(default_log_level.into())
                .from_env_lossy(),
        )
        .with(
            fmt::layer()
                .event_format(fmt::format().compact())
                .with_span_events(fmt::format::FmtSpan::ACTIVE)
                .with_thread_names(true),
        );

    reg.try_init()?;

    Ok(())
}
