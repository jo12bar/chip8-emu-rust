//! The CHIP8 emulator itself.
//!
//! Typically, the emulator is run in a background thread. It periodically
//! wakes up the UI thread to re-paint only when it executes an instruction that
//! requires re-painting.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use color_eyre::eyre::Context;
use crossbeam::channel::{self, Receiver, Sender};

use crate::{
    display::{chip8_display::Chip8Display, Display, DisplayRef},
    ram::Ram,
};

/// The CHIP8 emulator.
///
/// Some components of this emulator can be switched out at runtime
#[derive(Debug, Clone)]
pub struct Emulator {
    should_run: Arc<AtomicBool>,

    display: DisplayRef,
    display_ref_sender: Sender<DisplayRef>,

    /// A channel receiver for receiving new references to new CHIP8 displays.
    ///
    /// The Emulator will send a reference to this channel whenever it switches
    /// to a brand new display. The UI thread should use this new reference as
    /// a signal to create new textures for rendering the display to.
    ///
    /// This is a bounded channel with a queue size of 1.
    pub display_ref_receiver: Receiver<DisplayRef>,

    frame_ready_to_render: Arc<AtomicBool>,

    ram: Arc<Mutex<Ram>>,
}

impl Emulator {
    /// Create a new emulator.
    ///
    /// To start it, call [`Self::start`].
    pub fn new() -> Self {
        let (display_ref_sender, display_ref_receiver) = channel::bounded(1);

        Self {
            should_run: Arc::new(AtomicBool::new(false)),
            display: Arc::new(Mutex::new(None)),
            display_ref_sender,
            display_ref_receiver,
            frame_ready_to_render: Arc::new(AtomicBool::new(true)),
            ram: Arc::new(Mutex::new(Ram::default())),
        }
    }

    /// Start the emulator's main run loop in a background thread.
    ///
    /// The [`egui::Context`] is used to wake the UI thread whenever repainting
    /// is required.
    ///
    /// Returns an error if the emulator is *already* running.
    pub fn start(self, egui_context: egui::Context) -> color_eyre::Result<()> {
        if self.should_run.load(Ordering::SeqCst) {
            return Err(color_eyre::eyre::eyre!("The emulator is already running!"));
        }

        std::thread::Builder::new()
            .name("emulator".to_string())
            .spawn(move || {
                self.main_run_loop(egui_context);
            })
            .wrap_err("Failed to start emulator background thread")?;

        Ok(())
    }

    /// The emulator's main run loop. This is run in a background thread by [`Self::start()`].
    fn main_run_loop(self, egui_context: egui::Context) {
        self.should_run.store(true, Ordering::SeqCst);

        tracing::info!("Starting main run loop");

        self.attach_display(Box::new(Chip8Display::new()), &egui_context)
            .unwrap();

        let mut x = 0;
        let mut y = 0;

        while self.should_run.load(Ordering::Acquire) {
            {
                let mut display = self.display.lock().unwrap();
                let display = display.as_mut().unwrap();
                display.flip_pixel(x, y);
            }

            x = (x + 1) % 64;
            y = (y + 1) % 32;

            self.set_frame_ready_to_render();
            egui_context.request_repaint();

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Stop the emulator.
    pub fn stop(&mut self) {
        tracing::info!("Stopping emulator");
        self.should_run.store(false, Ordering::SeqCst);
    }

    /// Attach a new display to the emulator. This is usually done when switching
    /// between emulating different types of CHIP8.
    fn attach_display(
        &self,
        display: Box<dyn Display>,
        egui_context: &egui::Context,
    ) -> color_eyre::Result<()> {
        *self.display.lock().unwrap() = Some(display);
        self.display_ref_sender.send(Arc::clone(&self.display))?;
        self.set_frame_ready_to_render();
        egui_context.request_repaint();
        Ok(())
    }

    /// Check if the emulator has prepared a new frame for rendering.
    ///
    /// If this function returns `true`, the frame should be rendered. Once
    /// the frame is successfully rendered, notify the emulator with
    /// [`Self::notify_frame_rendered`].
    #[inline]
    pub fn is_frame_ready_to_render(&self) -> bool {
        self.frame_ready_to_render.load(Ordering::Acquire)
    }

    /// Notify the emulator that its latest frame has been successfully rendered.
    ///
    /// This will cause future calls to [`Self::is_frame_ready_to_render`] to
    /// return `false` until the emulator has a new frame available for rendering.
    #[inline]
    pub fn notify_frame_rendered(&self) {
        self.frame_ready_to_render.store(false, Ordering::Release);
    }

    /// Call to atomically notify other threads that the emulator has a new frame
    /// ready for rendering, once they check with [`Self::is_frame_ready_to_render`].
    #[inline]
    fn set_frame_ready_to_render(&self) {
        self.frame_ready_to_render.store(true, Ordering::Release);
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
