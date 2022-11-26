use color_eyre::Result;

mod app;
mod display;
mod ram;
mod renderer;
mod sys_font;

pub use app::App;

pub fn setup_logging() -> Result<()> {
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
                .with_span_events(fmt::format::FmtSpan::ACTIVE),
        );

    reg.try_init()?;

    Ok(())
}
