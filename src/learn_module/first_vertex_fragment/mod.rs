use std::sync::Arc;

use pollster::FutureExt;
use wgpu::{
    Backends, BlendState, ColorTargetState, ColorWrites, Device, ExperimentalFeatures, Features,
    FragmentState, Instance, Limits, Operations, PipelineCompilationOptions, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPipeline, Surface, SurfaceConfiguration, SurfaceError,
    TextureUsages,
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
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(core) = &mut self.core {
            core.surface_cfg.width = width;
            core.surface_cfg.height = height;
            core.surface.configure(&core.device, &core.surface_cfg);
        }
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        self.window.as_ref().unwrap().request_redraw();

        let output = self.core.as_ref().unwrap().surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor {
                ..Default::default()
            });

        let mut encoder = self.core.as_ref().unwrap().device.create_command_encoder(
            &wgpu::wgt::CommandEncoderDescriptor {
                label: Some("Create Encoder"),
            },
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Create Render"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.,
                            g: 0.,
                            b: 0.,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.core.as_ref().unwrap().render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        self.core
            .as_ref()
            .unwrap()
            .queue
            .submit(Some(encoder.finish()));
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match self.render() {
                Ok(_) => (),
                Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                    let inner_size = self.window.as_ref().unwrap().inner_size();
                    self.resize(inner_size.width, inner_size.height)
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let None = self.window {
            let size = (800, 500);
            let attribute = Window::default_attributes()
                .with_title("my app")
                .with_inner_size(LogicalSize::new(size.0, size.1));

            let window = Arc::new(
                event_loop
                    .create_window(attribute)
                    .expect("error create window"),
            );

            let instace = Instance::new(&wgpu::InstanceDescriptor {
                backends: Backends::all(),
                ..Default::default()
            });

            let adapter = instace
                .request_adapter(&wgpu::RequestAdapterOptionsBase {
                    power_preference: wgpu::PowerPreference::LowPower,
                    force_fallback_adapter: false,
                    compatible_surface: None,
                })
                .block_on()
                .unwrap();

            let (device, queue) = adapter
                .request_device(&wgpu::wgt::DeviceDescriptor {
                    label: Some("Requst Device And Queue"),
                    required_features: Features::empty(),
                    required_limits: Limits::defaults(),
                    experimental_features: ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    trace: wgpu::Trace::Off,
                })
                .block_on()
                .unwrap();

            let surface = instace
                .create_surface(window.clone())
                .expect("Error init surface");
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let surface_cfg = SurfaceConfiguration {
                width: size.0,
                height: size.1,
                view_formats: Vec::new(),
                alpha_mode: surface_caps.alpha_modes[0],
                desired_maximum_frame_latency: 2,
                format: surface_format,
                present_mode: surface_caps.present_modes[0],
                usage: TextureUsages::RENDER_ATTACHMENT,
            };

            let shaders = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Create Shaders"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Create Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Create Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shaders,
                    entry_point: Some("vertex_main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    buffers: &[],
                },
                primitive: PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                    unclipped_depth: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentState {
                    entry_point: Some("fragment_main"),
                    compilation_options: PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    module: &shaders,
                    targets: &[Some(ColorTargetState {
                        format: surface_cfg.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
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

            self.core = Some(core);
            self.window = Some(window);
        }
    }
}

pub fn first_vertex_fragment_running() {
    let event_loop = EventLoop::new().expect("error create event loop");

    let mut app = MyApp {
        window: None,
        core: None,
    };

    event_loop.run_app(&mut app).expect("error running app");
}
