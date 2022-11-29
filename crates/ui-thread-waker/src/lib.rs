//! A trait that allows the emulator to wake the UI thread from sleep when required.

/// Implementing this trait for a type allows the emulator to "wake" the UI thread
/// from a parked or sleeping state.
///
/// The actual mechanism of waking the UI thread is left to the implementer.
pub trait UiThreadWaker {
    /// Wake the UI thread.
    ///
    /// The implementer should define the actual mechanism of waking the UI thread.
    fn wake_ui_thread(&self);
}
