use std::sync::Arc;

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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let app = match self {
            AppState::Initialized(app) => app,
            AppState::Uninitialized => return,
        };

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::RedrawRequested => app.window.request_redraw(),
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
            // connect js console logs
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).unwrap();

            // connect canvas
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
}
