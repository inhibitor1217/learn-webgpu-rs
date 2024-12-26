use std::sync::Arc;

use log::{error, info, warn};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::graphics::gpu;

enum AppState {
    Uninitialized,
    Initialized(App),
}

impl AppState {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            AppState::Initialized(_) => panic!("app already initialized"),
            AppState::Uninitialized => {
                *self = AppState::Initialized(App::new(event_loop));
            }
        }
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            AppState::Initialized(_) => (), // no-op for now
            AppState::Uninitialized => self.init(event_loop),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let app = match self {
            AppState::Initialized(app) => app,
            AppState::Uninitialized => return,
        };

        if window_id != app.window.id() {
            return;
        }

        match event {
            WindowEvent::Resized(physical_size) => {
                app.resize(physical_size.width, physical_size.height);
            }
            WindowEvent::CloseRequested => {
                info!(target: "app", "close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::Destroyed => {
                info!(target: "app", "window destroyed, exiting");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                app.window.request_redraw();

                match app.render() {
                    Ok(_) => (),
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => todo!(),
                    Err(wgpu::SurfaceError::Timeout) => {
                        warn!(target: "app", "timeout while acquiring frame");
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        error!(target: "app", "out of memory while acquiring frame");
                        event_loop.exit();
                    }
                }
            }
            _ => (),
        }
    }
}

pub struct App {
    pub(crate) window: Arc<Window>,
    pub(crate) gpu: Arc<gpu::Gpu>,
}

impl App {
    pub fn run() {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app_state = AppState::Uninitialized;
        event_loop.run_app(&mut app_state).unwrap();
    }

    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window_attrs = Window::default_attributes().with_title("Tutorial Window");
        let window = event_loop.create_window(window_attrs).unwrap();
        let window = Arc::new(window);

        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).unwrap();

            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let root = doc.get_element_by_id("root").unwrap();
                    let canvas = web_sys::Element::from(window.canvas()?);
                    root.append_child(&canvas).unwrap();
                    Some(())
                })
                .unwrap();
        }

        let gpu = pollster::block_on(gpu::Gpu::new(window.clone()));
        let gpu = Arc::new(gpu);

        Self { window, gpu }
    }

    fn resize(&mut self, width: u32, height: u32) {
        info!(target: "app", "resizing window to {}x{}", width, height);

        self.gpu.resize(width, height);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.gpu.surface.get_current_texture()?;
        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

        {
            let _render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.gpu
            .queue
            .submit(std::iter::once(command_encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
