use std::sync::Arc;

use pollster::FutureExt;
use wgpu::{
    Backends, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, Queue, RequestAdapterOptionsBase, Surface, SurfaceConfiguration,
    TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct CoreMyApp {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_cfg: bool,
}

struct MyApp {
    window: Option<Arc<Window>>,
    core: Option<CoreMyApp>,
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
        if self.core.is_none() {
            // init
            let inner_size = (800, 500);
            // wgpu
            let instance = Instance::new(&InstanceDescriptor {
                backends: Backends::all(),
                ..Default::default()
            });

            let surface = instance
                .create_surface(self.window.clone().unwrap())
                .expect("Error Create Surface");

            let adapter = instance
                .request_adapter(&RequestAdapterOptionsBase {
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    power_preference: wgpu::PowerPreference::LowPower,
                })
                .block_on()
                .unwrap();

            let (device, queue) = adapter
                .request_device(&DeviceDescriptor {
                    label: Some("Request Device And Queue"),
                    experimental_features: ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    required_features: Features::empty(),
                    required_limits: Limits {
                        ..Default::default()
                    },
                    trace: wgpu::Trace::Off,
                })
                .block_on()
                .unwrap();

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let config = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                desired_maximum_frame_latency: 2,
                view_formats: vec![],
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                format: surface_format,
                width: inner_size.0,
                height: inner_size.1,
            };
            // wgpu

            let attribute = Window::default_attributes()
                .with_title("my app")
                .with_inner_size(LogicalSize::new(inner_size.0, inner_size.1));

            let window = event_loop
                .create_window(attribute)
                .expect("Error Create Window");

            let core = CoreMyApp {
                config,
                device,
                queue,
                surface,
                is_surface_cfg: false,
            };
            self.core = Some(core);
            self.window = Some(Arc::new(window));
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut my_app = MyApp {
        window: None,
        core: None,
    };
    event_loop
        .run_app(&mut my_app)
        .expect("Error To Running App");
}
