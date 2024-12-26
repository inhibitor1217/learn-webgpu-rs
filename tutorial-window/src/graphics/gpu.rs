use std::sync::{Arc, Mutex};

use super::{render_encoder, render_target};

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: Mutex<wgpu::SurfaceConfiguration>,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Gpu {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_config = Self::surface_default_config(&surface, &adapter, &window);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/triangle.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let gpu = Self {
            instance,
            device,
            queue,
            surface,
            surface_config: Mutex::new(surface_config),
            render_pipeline,
        };

        gpu.surface_configure();

        gpu
    }

    pub fn resize(&self, width: u32, height: u32) {
        let mut surface_config = self.surface_config.lock().unwrap();
        surface_config.width = width;
        surface_config.height = height;
        self.surface.configure(&self.device, &surface_config);
    }

    fn surface_size(window: &winit::window::Window) -> (u32, u32) {
        let size = window.inner_size();
        (size.width.max(1), size.height.max(1))
    }

    fn surface_default_config(
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        window: &winit::window::Window,
    ) -> wgpu::SurfaceConfiguration {
        let surface_capabilities = surface.get_capabilities(&adapter);

        let format = surface_capabilities
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        let (width, height) = Self::surface_size(window);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    fn surface_configure(&self) {
        let surface_config = self.surface_config.lock().unwrap();
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn start_frame(&self) -> render_target::SurfaceRenderTarget {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Timeout) => self
                .surface
                .get_current_texture()
                .expect("failed to acquire surface texture due to timeout"),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                panic!("failed to acquire surface texture due to out of memory")
            }
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface_configure();
                self.surface
                    .get_current_texture()
                    .expect("failed to acquire surface texture after reconfiguration")
            }
        };

        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        render_target::SurfaceRenderTarget {
            surface_texture,
            target_view,
        }
    }

    pub fn render_encoder(&self) -> render_encoder::RenderEncoder {
        render_encoder::RenderEncoder::new(self)
    }
}
