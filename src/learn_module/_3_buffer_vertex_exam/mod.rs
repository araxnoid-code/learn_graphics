use std::sync::Arc;

use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::EventLoop,
    window::Window,
};

struct MyApp {
    window: Option<Arc<Window>>,
}

impl MyApp {
    pub fn init() -> MyApp {
        Self { window: None }
    }
}

impl ApplicationHandler for MyApp {
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let None = self.window {
            let size = (800, 500);
            let attribute = Window::default_attributes()
                .with_title("My Buffer Exam")
                .with_inner_size(LogicalSize::new(size.0, size.1));

            let window = Arc::new(
                event_loop
                    .create_window(attribute)
                    .expect("Error Create Window"),
            );

            self.window = Some(window);
        }
    }
}

pub fn buffer_vertex_exam_runnning() {
    let event_loop = EventLoop::new().expect("Error Init Event Loop");
    let mut my_app = MyApp::init();

    event_loop.run_app(&mut my_app).expect("Error Running App");
}
