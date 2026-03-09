mod testing;
use std::sync::Arc;

use pollster::FutureExt;
use wgpu::{
    Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, ExperimentalFeatures, Features,
    Instance, InstanceDescriptor, Limits, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptionsBase, Surface, SurfaceConfiguration, SurfaceError,
    TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::testing::running;

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

impl MyApp {
    fn ready_to_render(&self) -> bool {
        !self.core.is_none() && !self.window.is_none()
    }
    fn render(&self) -> Result<(), SurfaceError> {
        if !self.ready_to_render() {
            return Ok(());
        }

        self.window.as_ref().unwrap().request_redraw();

        if !self.core.as_ref().unwrap().is_surface_cfg {
            return Ok(());
        }

        let output = self.core.as_ref().unwrap().surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor {
                ..Default::default()
            });

        let mut encoder =
            self.core
                .as_ref()
                .unwrap()
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Create Encoder"),
                });

        {
            let brp = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Create Render Pass"),
                depth_stencil_attachment: None,
                multiview_mask: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        store: wgpu::StoreOp::Store,
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                    },
                })],
            });
        }

        self.core
            .as_ref()
            .unwrap()
            .queue
            .submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if !self.core.is_none() {
            let core = self.core.as_mut().unwrap();
            if width > 0 && height > 0 {
                core.config.height = height;
                core.config.width = width;
                core.surface.configure(&core.device, &core.config);
                core.is_surface_cfg = true;
            }
        }
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
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match self.render() {
                Ok(_) => {}
                Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                    let inner_size = self.window.as_ref().unwrap().inner_size();
                    self.resize(inner_size.width, inner_size.height);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let inner_size = (800, 500);
            let attribute = Window::default_attributes()
                .with_title("my app")
                .with_inner_size(LogicalSize::new(inner_size.0, inner_size.1));

            let window = event_loop
                .create_window(attribute)
                .expect("Error Create Window");
            self.window = Some(Arc::new(window));
            println!("window done");

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

            let core = CoreMyApp {
                config,
                device,
                queue,
                surface,
                is_surface_cfg: false,
            };
            self.core = Some(core);
            println!("wgpu done");
        }
    }
}

fn main() {
    running();
    // let event_loop = EventLoop::new().unwrap();
    // event_loop.set_control_flow(ControlFlow::Wait);

    // let mut my_app = MyApp {
    //     window: None,
    //     core: None,
    // };
    // event_loop
    //     .run_app(&mut my_app)
    //     .expect("Error To Running App");
}
