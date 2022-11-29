//! The basic CHIP8 display.

use display::Display;
use image::{ImageBuffer, Pixel, RgbaImage};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

/// The basic CHIP8 display.
///
/// The CHIP8 display is black-and-white, and is 64 pixels wide and 32 pixels
/// tall. Each pixel can be "on" or "off".
///
/// Original interpreters updated the display at 60 Hz, but Rust Chip currently
/// rerenders as fast as the host GPU allows. Someday, this
/// will be changed to only rerender whenever an instruction is
/// executed that updates the CHIP8 display.
#[derive(Clone, Debug)]
pub struct Chip8Display {
    /// While we *could* just use bitwise operations on numbers to represent
    /// the display (since each pixel can only be on or off), we instead use
    /// an [`image::RgbaImage`]. This is done for the following reasons:
    ///
    /// 1. It's easier to convert an [`image::RgbaImage`] to a GPU texture
    ///    for rendering.
    /// 2. This allows the future implementation of a multi-color display, such
    ///    as that described by the XO-CHIP specification.
    buf: RgbaImage,
}

impl Chip8Display {
    /// Instantiate a new CHIP8 display.
    pub fn new() -> Self {
        tracing::info!("Initializing CHIP8 display");

        let mut buf: RgbaImage = ImageBuffer::from_fn(WIDTH, HEIGHT, |x, y| {
            if (x % 2 == 0 && y % 2 != 0) || (x % 2 != 0 && y % 2 == 0) {
                image::Rgba([0, 0, 0, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        });

        // Add some orientating pixels for testing purposes.

        buf[(0, 0)] = image::Rgba([255, 0, 0, 255]); // top-left
        buf[(63, 0)] = image::Rgba([0, 255, 0, 255]); // top-right
        buf[(0, 31)] = image::Rgba([0, 0, 255, 255]); // bottom-left
        buf[(63, 31)] = image::Rgba([255, 0, 255, 255]); // bottom-right

        Self { buf }
    }
}

impl Default for Chip8Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Chip8Display {
    fn drop(&mut self) {
        tracing::info!("Destroying CHIP8 display");
    }
}

impl Display for Chip8Display {
    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        self.buf.dimensions()
    }

    #[inline]
    fn as_rgba8_image(&self) -> &RgbaImage {
        &self.buf
    }

    #[inline]
    fn is_srgb(&self) -> bool {
        false
    }

    fn flip_pixel(&mut self, x: u32, y: u32) {
        let p = self.buf.get_pixel_mut(x % WIDTH, y % HEIGHT);
        p.invert();
    }
}
