use std::sync::{Arc, Mutex};

use wgpu::util::DeviceExt;
use winit::{event::*, window::Window};

use crate::display::{
    blank_display::BlankDisplay, chip8_display::Chip8Display, Display, WgpuDisplayTexture,
};

/// A [`wgpu`] renderer for rendering the emulated screen and the GUI.
pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pub size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    display_bind_group_layout: wgpu::BindGroupLayout,

    blank_display_texture_bind_group: wgpu::BindGroup,

    display: Option<Arc<Mutex<dyn Display>>>,
    display_texture: Option<WgpuDisplayTexture>,
    display_texture_bind_group: Option<wgpu::BindGroup>,

    screen_size_uniform: ScreenSizeUniform,
    screen_size_buffer: wgpu::Buffer,
    screen_size_bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// Create a new renderer.
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // webgl doesn't support all of wgpu's features, so if we're building
                    // for the web then we have to disable some of them.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: if surface
                .get_supported_present_modes(&adapter)
                .contains(&wgpu::PresentMode::Mailbox)
            {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let display_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the corresponding
                        // Texture entry above:
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Display bind group layout"),
            });

        let screen_size_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Screen size bind group layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&display_bind_group_layout, &screen_size_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let blank_display = BlankDisplay::new();
        let blank_display_texture = WgpuDisplayTexture::from_chip8_display(
            &device,
            &queue,
            &blank_display,
            Some("Blank display"),
        );

        let blank_display_texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &display_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&blank_display_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&blank_display_texture.sampler),
                    },
                ],
                label: Some("Blank display bind group"),
            });

        let mut screen_size_uniform = ScreenSizeUniform::new();
        screen_size_uniform.update_size(size.into());

        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen size uniform buffer"),
            contents: bytemuck::cast_slice(&[screen_size_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &screen_size_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
            label: Some("Screen size bind group"),
        });

        Self {
            surface,
            device,
            queue,
            config,

            size,

            render_pipeline,

            vertex_buffer,
            index_buffer,
            num_indices,

            display_bind_group_layout,

            blank_display_texture_bind_group,

            display: None,
            display_texture: None,
            display_texture_bind_group: None,

            screen_size_uniform,
            screen_size_buffer,
            screen_size_bind_group,
        }
    }

    /// Resize the renderer. This has the side-effect of re-configuring the
    /// render surface, and re-instantiating the render pipeline.
    #[tracing::instrument(level = "INFO", skip(self))]
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update the DisplayTexScaleUniform with the new paintable area size.
            self.screen_size_uniform.update_size(new_size.into());

            self.queue.write_buffer(
                &self.screen_size_buffer,
                0,
                bytemuck::cast_slice(&[self.screen_size_uniform]),
            );
        }
    }

    /// Attach a new CHIP8-compatible display to the renderer.
    ///
    /// This will allocate the GPU textures and bind groups necessary for the
    /// display. The renderer will then start rendering the display the next
    /// time [`Renderer::render()`] is called.
    ///
    /// Whatever previous display was in use will be released, and its textures
    /// and bind groups deallocated.
    pub fn attach_display(
        &mut self,
        new_display: Arc<Mutex<dyn Display>>,
        display_label: Option<&str>,
        display_bind_group_label: Option<&str>,
    ) {
        let display_texture = {
            let new_display = new_display.lock().unwrap();

            WgpuDisplayTexture::from_chip8_display(
                &self.device,
                &self.queue,
                &*new_display,
                display_label,
            )
        };

        let display_texture_bind_group =
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.display_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&display_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&display_texture.sampler),
                    },
                ],
                label: display_bind_group_label,
            });

        self.display = Some(new_display);
        self.display_texture = Some(display_texture);
        self.display_texture_bind_group = Some(display_texture_bind_group);
    }

    /// Detach the current CHIP8-compatible display.
    ///
    /// A black 1x1 pixel will be rendered in its place on the next call to
    /// [`Self::render()`].
    pub fn detach_display(&mut self) {
        self.display.take();
        self.display_texture.take();
        self.display_texture_bind_group.take();
    }

    /// Handle input. This will probably be moved to some other module at some
    /// point.
    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    /// Update the renderer with new data to render.
    pub fn update(&mut self) {
        // no-op
    }

    /// Render a frame.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut cmd_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render command encoder"),
            });

        {
            let mut render_pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            if let Some(display_texture_bind_group) = &self.display_texture_bind_group {
                render_pass.set_bind_group(0, display_texture_bind_group, &[]);
            } else {
                render_pass.set_bind_group(0, &self.blank_display_texture_bind_group, &[]);
            }

            render_pass.set_bind_group(1, &self.screen_size_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(cmd_encoder.finish()));
        output.present();

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    /// WGPU vertex attributes describing how to address the data contained in
    /// this sort of vertex.
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    /// Return the vertex buffer layout descriptor for this sort of vertex.
    const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 0.0], }, // top-left
    Vertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 0.0], }, // top-right
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0], }, // bottom-left
    Vertex { position: [ 1.0, -1.0, 0.0], tex_coords: [1.0, 1.0], }, // bottom-right
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
    0, 2, 1,
    2, 3, 1,
];

/// A uniform for sending the current paintable area size to the GPU, as well
/// as the size of the [`WgpuDisplayTexture`] being painted.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenSizeUniform {
    screen_size: [f32; 2],

    /// This padding is necessary to ensure that this uniform buffer remains
    /// aligned to 16-byte boundaries, which is required for WebGL.
    _padding: [f32; 2],
}

impl ScreenSizeUniform {
    fn new() -> Self {
        Self {
            screen_size: [1.0, 1.0],
            _padding: [0.0, 0.0],
        }
    }

    fn update_size(&mut self, (paint_area_width, paint_area_height): (u32, u32)) {
        self.screen_size = [paint_area_width as f32, paint_area_height as f32];
    }
}
