use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// Vertex data with position and color
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Cube vertices with colors for each face
const VERTICES: &[Vertex] = &[
    // Front face (red)
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.2, 0.2] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.2, 0.2] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.2, 0.2] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.2, 0.2] },
    // Back face (green)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.2, 1.0, 0.2] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.2, 1.0, 0.2] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.2, 1.0, 0.2] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.2, 1.0, 0.2] },
    // Top face (blue)
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.2, 0.2, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.2, 0.2, 1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [0.2, 0.2, 1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.2, 0.2, 1.0] },
    // Bottom face (yellow)
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 0.2] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 0.2] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 0.2] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 0.2] },
    // Right face (magenta)
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.2, 1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 0.2, 1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.2, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.2, 1.0] },
    // Left face (cyan)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.2, 1.0, 1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.2, 1.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.2, 1.0, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.2, 1.0, 1.0] },
];

// Indices for the cube (two triangles per face)
const INDICES: &[u16] = &[
    0,  1,  2,  0,  2,  3,   // front
    4,  5,  6,  4,  6,  7,   // back
    8,  9,  10, 8,  10, 11,  // top
    12, 13, 14, 12, 14, 15,  // bottom
    16, 17, 18, 16, 18, 19,  // right
    20, 21, 22, 20, 22, 23,  // left
];

// Uniform buffer for MVP matrix
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            mvp: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    fn update(&mut self, rotation: f32, aspect: f32) {
        let model = Mat4::from_rotation_y(rotation) * Mat4::from_rotation_x(rotation * 0.7);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 3.0), // eye position
            Vec3::ZERO,               // target
            Vec3::Y,                  // up
        );
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        self.mvp = (proj * view * model).to_cols_array_2d();
    }
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    start_time: Instant,
    window: Arc<Window>,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window.clone()).unwrap();

        // Request adapter (will use your RTX 3070 on Vulkan)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        log::info!("Using adapter: {:?}", adapter.get_info());

        // Create device and queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Main Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device");

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cube Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create uniform buffer
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout and bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
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
            num_indices: INDICES.len() as u32,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            start_time: Instant::now(),
            window,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn create_depth_texture(&self) -> wgpu::TextureView {
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Calculate rotation based on elapsed time
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let aspect = self.config.width as f32 / self.config.height as f32;
        self.uniforms.update(elapsed, aspect);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        // Get the current frame
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture view
        let depth_view = self.create_depth_texture();

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App {
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Rotating Cube - wgpu + Rust")
            .with_inner_size(PhysicalSize::new(800, 600));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(State::new(window)));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                log::info!("Resized to {:?}", physical_size);
                state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory!");
                        event_loop.exit();
                    }
                    Err(e) => log::error!("Render error: {:?}", e),
                }
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    log::info!("Starting Rotating Cube application");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}