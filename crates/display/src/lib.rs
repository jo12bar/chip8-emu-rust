use std::fmt;
use std::sync::{Arc, Mutex};

use image::RgbaImage;

/// A [`Display`] that can be synchronized between threads. The display may or
/// may not even exist.
pub type DisplayRef = Arc<Mutex<Option<Box<dyn Display>>>>;

/// A generic CHIP8-compatible display.
///
/// This allows for the implementation of _multiple_ different sorts of CHIP8
/// displays, from the base, black-and-white 64x32 original display to the
/// upgraded, multicolour XO-CHIP display.
pub trait Display: Send + Sync + fmt::Debug {
    /// Return the dimensions of the CHIP8 display as a pair of `(width, height)`.
    fn dimensions(&self) -> (u32, u32);

    /// Return the image as a regular RGBA8 image from the [`image`] crate.
    fn as_rgba8_image(&self) -> &RgbaImage;

    /// Returns true if the display is in the sRGB colour space, and false if
    /// it's in the regular, linear RGB colour space.
    fn is_srgb(&self) -> bool;

    /// Flip a pixel at some location.
    ///
    /// Out-of-bounds accesses will be silently ignored, for the sake of emulator
    /// stability. Generally, [`Display`] implementations will use some form of
    /// wrap-around to accomplish this.
    fn flip_pixel(&mut self, x: u32, y: u32);
}
