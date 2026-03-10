use std::{io::SeekFrom, sync::Arc};

use bytemuck::{Pod, Zeroable};
use pollster::FutureExt;
use wgpu::{
    Backends, BlendState, Buffer, BufferAddress, BufferUsages, ColorTargetState, ColorWrites,
    Device, ExperimentalFeatures, Features, FragmentState, Instance, Limits, LoadOp, Operations,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPipeline, Surface,
    SurfaceConfiguration, SurfaceError, TextureUsages, VertexAttribute, VertexBufferLayout,
    VertexState, util::DeviceExt,
};
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::EventLoop,
    window::Window,
};

#[repr(C)]
#[derive(Debug, Zeroable, Pod, Clone, Copy)]
struct MyVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl MyVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<MyVertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 0,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 1,
                },
            ],
        }
    }
}

impl MyVertex {
    pub fn triangle_rgb() -> Vec<MyVertex> {
        vec![
            MyVertex {
                position: [0., 0.5, 0.],
                color: [1., 0., 0.],
            },
            MyVertex {
                position: [-0.5, -0.5, 0.],
                color: [0., 1., 0.],
            },
            MyVertex {
                position: [0.5, -0.5, 0.],
                color: [0., 0., 1.],
            },
        ]
    }
}

struct Core {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_cfg: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
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
        if let Some(core) = self.core.as_mut() {
            core.surface_cfg.width = width;
            core.surface_cfg.height = height;
            core.surface.configure(&core.device, &core.surface_cfg);
        }
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        if let None = self.window {
            return Ok(());
        }
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
                label: Some("Create Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 1.,
                            g: 1.,
                            b: 1.,
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

            render_pass.set_pipeline(&self.core.as_ref().unwrap().pipeline);
            render_pass.set_vertex_buffer(0, self.core.as_ref().unwrap().vertex_buffer.slice(..));
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
            WindowEvent::Resized(inner_size) => self.resize(inner_size.width, inner_size.height),
            WindowEvent::RedrawRequested => match self.render() {
                Ok(_) => (),
                Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                    let inner_size = self.window.as_ref().unwrap().inner_size();
                    self.resize(inner_size.width, inner_size.height)
                }
                _ => (),
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let None = &self.window {
            let inner_size = (800, 400);
            let attribute = Window::default_attributes()
                .with_title("My Exam")
                .with_inner_size(LogicalSize::new(inner_size.0, inner_size.1));

            let window = Arc::new(
                event_loop
                    .create_window(attribute)
                    .expect("Error Create Window"),
            );

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
                    required_features: Features::empty(),
                    required_limits: Limits::defaults(),
                    experimental_features: ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
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

            let surface_cfg = SurfaceConfiguration {
                alpha_mode: surface_caps.alpha_modes[0],
                desired_maximum_frame_latency: 2,
                format: surface_format,
                width: inner_size.0,
                height: inner_size.1,
                present_mode: surface_caps.present_modes[0],
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: vec![],
            };

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Create Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Create Module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Create Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    buffers: &[MyVertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    entry_point: Some("vertex_main"),
                    module: &shader,
                },
                primitive: PrimitiveState {
                    polygon_mode: wgpu::PolygonMode::Fill,
                    cull_mode: Some(wgpu::Face::Back),
                    front_face: wgpu::FrontFace::Ccw,
                    strip_index_format: None,
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    conservative: false,
                    unclipped_depth: false,
                },
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentState {
                    entry_point: Some("fragment_main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    module: &shader,
                    targets: &[Some(ColorTargetState {
                        blend: Some(BlendState::REPLACE),
                        format: surface_cfg.format,
                        write_mask: ColorWrites::all(),
                    })],
                }),
                depth_stencil: None,
                multiview_mask: None,
                cache: None,
            });

            let triangle_rgb = MyVertex::triangle_rgb();
            let vetex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Create Triangle Buffer"),
                contents: bytemuck::cast_slice(&triangle_rgb),
                usage: BufferUsages::VERTEX,
            });

            let core = Core {
                device,
                queue,
                surface,
                surface_cfg,
                pipeline,
                vertex_buffer: vetex_buffer,
            };

            self.core = Some(core);
            self.window = Some(window);
        }
    }
}

pub fn buffer_vertex_runnning() {
    let event_loop = EventLoop::new().expect("Error Create Event Loop");
    let mut app = MyApp::init();
    event_loop.run_app(&mut app).expect("Error Running App");
}
