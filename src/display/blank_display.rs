//! A blank CHIP8 display to use when no other display is connected to the renderer.

use image::{ImageBuffer, RgbaImage};

use super::Display;

/// This is a fake CHIP8 display that always outputs a single, 1x1 black image.
///
/// It's used by the renderer when no CHIP8-compatible display is attached,
/// just for the sake of rendering *something* without completely tearing down
/// and rebuilding the render pipeline every time a new CHIP8-compatible display
/// is attached to it.
#[derive(Debug, Clone)]
pub struct BlankDisplay {
    buf: RgbaImage,
}

impl BlankDisplay {
    /// Create a new blank display.
    pub fn new() -> Self {
        let buf: RgbaImage = ImageBuffer::new(1, 1);

        Self { buf }
    }
}

impl Display for BlankDisplay {
    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        (1, 1)
    }

    #[inline]
    fn as_rgba8_image(&self) -> &RgbaImage {
        &self.buf
    }

    #[inline]
    fn is_srgb(&self) -> bool {
        false
    }
}
