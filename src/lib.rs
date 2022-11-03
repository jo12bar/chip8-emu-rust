use color_eyre::Result;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod ram;
mod renderer;
mod sys_font;

use renderer::Renderer;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    setup_logging().unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // If running on the web, make a <canvas> to render to
    #[cfg(target_arch = "wasm32")]
    {
        // winit prevents sizing with CSS, so we have to set the size manually
        // when on the web.
        //
        // TODO: change this to use ResizeObserver to auto-resize whenever the
        // browser window resizes once https://github.com/rust-windowing/winit/pull/2074
        // is merged!
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(1280, 720));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("rust-chip-canvas")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append <canvas> to document body.");
    }

    let r = ram::Ram::new();
    tracing::info!("{r:x?}");

    let mut renderer = Renderer::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        use winit::event::*;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !renderer.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,

                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                        }

                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(**new_inner_size);
                        }

                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                renderer.update();
                match renderer.render() {
                    Ok(_) => {}

                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),

                    // System is out of memory, so we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::ExitWithCode(1)
                    }

                    // All other errors (outdated, timeout, etc.) should be resolved by the next frame
                    Err(e) => tracing::error!(render_error=?e),
                }
            }

            Event::MainEventsCleared => {
                // Continually redraw in a hot loop.
                // TODO: Do something sensible like only redrawing when input is
                // detected, or when the emulator signals that a redraw is required.
                window.request_redraw();
            }

            _ => {}
        }
    });
}

fn setup_logging() -> Result<()> {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        } else {
            color_eyre::install()?;
        }
    }

    let default_log_level = if cfg!(debug_assertions) {
        LevelFilter::INFO
    } else {
        LevelFilter::WARN
    };

    let reg = tracing_subscriber::registry().with(
        EnvFilter::builder()
            .with_default_directive(default_log_level.into())
            .from_env_lossy(),
    );

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use tracing_web::{MakeConsoleWriter, performance_layer};
            use tracing_subscriber::fmt::time::UtcTime;

            let wasm_fmt_layer = fmt::layer()
                .with_ansi(false) // Only partially supported across browsers for now
                .with_timer(UtcTime::rfc_3339()) // std::time doesn't work in browsers
                .with_writer(MakeConsoleWriter) // write events to the browser console
                .with_span_events(fmt::format::FmtSpan::ACTIVE);

            let wasm_perf_layer = performance_layer()
                .with_details_from_fields(fmt::format::Pretty::default());

            let reg = reg.with(wasm_fmt_layer).with(wasm_perf_layer);
        } else {
            let reg = reg
                .with(fmt::layer().event_format(fmt::format().compact())
                .with_span_events(fmt::format::FmtSpan::ACTIVE));
        }
    }

    reg.try_init()?;

    Ok(())
}
