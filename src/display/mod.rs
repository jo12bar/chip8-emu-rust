use std::fmt;
use std::sync::{Arc, Mutex};

use eframe::wgpu;
use image::RgbaImage;
use thiserror::Error;

pub mod blank_display;
pub mod chip8_display;

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

/// The data contained in a CHIP8-compatible display as a wgpu-compatible Texture.
#[derive(Debug)]
pub struct WgpuDisplayTexture {
    /// Handle to the wgpu texture on the GPU.
    pub texture: wgpu::Texture,
    /// Handle to a wgpu texture view on the GPU.
    pub view: wgpu::TextureView,
    /// Handler to a wgpu texture sampler on the GPU.
    pub sampler: wgpu::Sampler,
    /// The size of the texture.
    pub size: wgpu::Extent3d,
}

impl WgpuDisplayTexture {
    /// Create a new wgpu texture, view, and sampler, ready for GPU rendering,
    /// from something that implements the CHIP8 [`Display`] trait.
    pub fn from_chip8_display(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display: &dyn Display,
        label: Option<&str>,
    ) -> Self {
        let rgba_buf = display.as_rgba8_image();
        let (width, height) = display.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if display.is_srgb() {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            rgba_buf,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size,
        }
    }

    /// Queue a write of new display data to the texture on the GPU.
    ///
    /// If the [`Display`] passed in has different dimensions than the [`Display`]
    /// used to create this `WgpuDisplayTexture`, then an error
    /// ([`WgpuDisplayTextureUpdateError::DimensionsChanged`]) will be returned.
    /// In this case, the `WgpuDisplayTexture` must be recreated from scratch.
    pub fn update<D: Display + ?Sized>(
        &self,
        new_display: &D,
        queue: &wgpu::Queue,
    ) -> Result<(), WgpuDisplayTextureUpdateError> {
        let new_rgba_buf = new_display.as_rgba8_image();
        let (new_width, new_height) = new_display.dimensions();

        if (new_width != self.size.width) || (new_height != self.size.height) {
            return Err(WgpuDisplayTextureUpdateError::DimensionsChanged {
                old: (self.size.width, self.size.height),
                new: (new_width, new_height),
            });
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            new_rgba_buf,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * self.size.width),
                rows_per_image: std::num::NonZeroU32::new(self.size.height),
            },
            self.size,
        );

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum WgpuDisplayTextureUpdateError {
    #[error(
        "The dimensions of the display used to create this texture are different from the \
        dimensions used to update this texture. The WgpuDisplayTexture must be recreated. \
        Old dimensions: {old:?}. New dimensions: {new:?}"
    )]
    DimensionsChanged { old: (u32, u32), new: (u32, u32) },
}
