use std::sync::Arc;

use pollster::FutureExt;
use wgpu::{
    Backends, BlendState, ColorTargetState, ColorWrites, Device, ExperimentalFeatures,
    FragmentState, Instance, Limits, Queue, RenderPassColorAttachment, RenderPipeline, Surface,
    SurfaceConfiguration, SurfaceError, TextureUsages, VertexState,
};
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::EventLoop,
    window::Window,
};

struct Core {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_cfg: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
}

struct MyApp {
    window: Option<Arc<Window>>,
    core: Option<Core>,
}

impl MyApp {
    pub fn init() -> MyApp {
        Self {
            window: None,
            core: None,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(core) = &mut self.core {
            core.surface_cfg.width = width;
            core.surface_cfg.height = height;
            core.surface.configure(&core.device, &core.surface_cfg);
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        if let None = self.window {
            return Ok(());
        }
        let window = self.window.as_ref().unwrap();
        let core = self.core.as_ref().unwrap();

        window.request_redraw();

        let output = core.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let mut encoder =
            core.device
                .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
                    label: Some("Create Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Create Begin Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.,
                            g: 1.,
                            b: 1.,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&core.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        core.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
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
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => match self.render() {
                Ok(_) => (),
                Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                    let size = self.window.as_ref().unwrap().inner_size();
                    self.resize(size.width, size.height);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let None = self.window {
            // window
            let size = (800, 500);
            let attribute = Window::default_attributes()
                .with_title("My Buffer Exam")
                .with_inner_size(LogicalSize::new(size.0, size.1));

            let window = Arc::new(
                event_loop
                    .create_window(attribute)
                    .expect("Error Create Window"),
            );
            // window

            // wgpu
            let instance = Instance::new(&wgpu::InstanceDescriptor {
                backends: Backends::all(),
                ..Default::default()
            });

            let surface = instance
                .create_surface(window.clone())
                .expect("Error Create Surface");

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptionsBase {
                    power_preference: wgpu::PowerPreference::LowPower,
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
                .block_on()
                .unwrap();

            let (device, queue) = adapter
                .request_device(&wgpu::wgt::DeviceDescriptor {
                    label: Some("Create Device And Queue"),
                    required_features: wgpu::Features::empty(),
                    required_limits: Limits::defaults(),
                    experimental_features: ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    trace: wgpu::Trace::Off,
                })
                .block_on()
                .unwrap();

            let surface_caps = surface.get_capabilities(&adapter);
            let format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let surface_cfg = SurfaceConfiguration {
                width: size.0,
                height: size.1,
                format,
                alpha_mode: surface_caps.alpha_modes[0],
                desired_maximum_frame_latency: 2,
                present_mode: surface_caps.present_modes[0],
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: vec![],
            };

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Create Pipeline"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

            let shaders = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Create Shaders"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
            });

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Create Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    entry_point: Some("main_vertex"),
                    module: &shaders,
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentState {
                    entry_point: Some("main_fragement"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    module: &shaders,
                    targets: &[Some(ColorTargetState {
                        blend: Some(BlendState::REPLACE),
                        format,
                        write_mask: ColorWrites::all(),
                    })],
                }),
                depth_stencil: None,
                multiview_mask: None,
                cache: None,
            });

            let core = Core {
                device,
                queue,
                surface,
                surface_cfg,
                render_pipeline,
            };
            // wgpu

            self.window = Some(window);
            self.core = Some(core);
        }
    }
}

pub fn buffer_vertex_exam_runnning() {
    let event_loop = EventLoop::new().expect("Error Init Event Loop");
    let mut my_app = MyApp::init();

    event_loop.run_app(&mut my_app).expect("Error Running App");
}
