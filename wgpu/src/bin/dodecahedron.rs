use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use std::f32::consts::PI;
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

// Vertex data with position and normal for lighting
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
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

// Generate dodecahedron vertices
fn generate_dodecahedron() -> (Vec<Vertex>, Vec<u16>) {
    // Golden ratio
    let phi: f32 = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let inv_phi = 1.0 / phi;

    // 20 vertices of a dodecahedron
    let base_vertices: Vec<Vec3> = vec![
        // Cube vertices (±1, ±1, ±1)
        Vec3::new( 1.0,  1.0,  1.0),
        Vec3::new( 1.0,  1.0, -1.0),
        Vec3::new( 1.0, -1.0,  1.0),
        Vec3::new( 1.0, -1.0, -1.0),
        Vec3::new(-1.0,  1.0,  1.0),
        Vec3::new(-1.0,  1.0, -1.0),
        Vec3::new(-1.0, -1.0,  1.0),
        Vec3::new(-1.0, -1.0, -1.0),
        // Rectangle in xy-plane (±phi, ±1/phi, 0)
        Vec3::new( phi,  inv_phi, 0.0),
        Vec3::new( phi, -inv_phi, 0.0),
        Vec3::new(-phi,  inv_phi, 0.0),
        Vec3::new(-phi, -inv_phi, 0.0),
        // Rectangle in yz-plane (0, ±phi, ±1/phi)
        Vec3::new(0.0,  phi,  inv_phi),
        Vec3::new(0.0,  phi, -inv_phi),
        Vec3::new(0.0, -phi,  inv_phi),
        Vec3::new(0.0, -phi, -inv_phi),
        // Rectangle in xz-plane (±1/phi, 0, ±phi)
        Vec3::new( inv_phi, 0.0,  phi),
        Vec3::new(-inv_phi, 0.0,  phi),
        Vec3::new( inv_phi, 0.0, -phi),
        Vec3::new(-inv_phi, 0.0, -phi),
    ];

    // 12 pentagonal faces (vertex indices)
    let faces: Vec<[usize; 5]> = vec![
        [0, 8, 9, 2, 16],    // face 1
        [0, 16, 17, 4, 12],  // face 2
        [0, 12, 13, 1, 8],   // face 3
        [1, 13, 5, 19, 18],  // face 4
        [1, 18, 3, 9, 8],    // face 5
        [2, 9, 3, 15, 14],   // face 6
        [2, 14, 6, 17, 16],  // face 7
        [3, 18, 19, 7, 15],  // face 8
        [4, 17, 6, 11, 10],  // face 9
        [4, 10, 5, 13, 12],  // face 10
        [5, 10, 11, 7, 19],  // face 11
        [6, 14, 15, 7, 11],  // face 12
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for face in &faces {
        // Calculate face center and normal
        let mut center = Vec3::ZERO;
        for &idx in face {
            center += base_vertices[idx];
        }
        center /= 5.0;
        
        // Calculate normal (pointing outward from center)
        let v0 = base_vertices[face[0]];
        let v1 = base_vertices[face[1]];
        let v2 = base_vertices[face[2]];
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let mut normal = edge1.cross(edge2).normalize();
        
        // Make sure normal points outward
        if normal.dot(center) < 0.0 {
            normal = -normal;
        }

        // Triangulate pentagon (fan from first vertex)
        let base_idx = vertices.len() as u16;
        
        for &idx in face {
            let pos = base_vertices[idx] * 0.5; // Scale down
            vertices.push(Vertex {
                position: pos.to_array(),
                normal: normal.to_array(),
            });
        }

        // Create triangles (fan triangulation)
        for i in 1..4 {
            indices.push(base_idx);
            indices.push(base_idx + i as u16);
            indices.push(base_idx + i as u16 + 1);
        }
    }

    (vertices, indices)
}

// Uniforms for MVP matrix and lighting
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    light_pos: [f32; 4],
    view_pos: [f32; 4],
    // Emerald material properties
    ambient: [f32; 4],
    diffuse: [f32; 4],
    specular: [f32; 4],
    shininess: f32,
    _padding: [f32; 3],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            light_pos: [3.0, 3.0, 3.0, 1.0],
            view_pos: [0.0, 0.0, 4.0, 1.0],
            // Emerald material (classic OpenGL emerald)
            ambient: [0.0215, 0.1745, 0.0215, 1.0],
            diffuse: [0.07568, 0.61424, 0.07568, 1.0],
            specular: [0.633, 0.727811, 0.633, 1.0],
            shininess: 76.8,
            _padding: [0.0; 3],
        }
    }

    fn update(&mut self, rotation: f32, aspect: f32) {
        let model = Mat4::from_rotation_y(rotation) * Mat4::from_rotation_x(rotation * 0.6);
        self.model = model.to_cols_array_2d();
        
        let eye = Vec3::new(0.0, 0.0, 4.0);
        let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
        self.view = view.to_cols_array_2d();
        
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        self.proj = proj.to_cols_array_2d();
        
        self.view_pos = [eye.x, eye.y, eye.z, 1.0];
        
        // Light source behind and above the camera
        self.light_pos = [0.0, 2.0, 6.0, 1.0];
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
    depth_texture: wgpu::TextureView,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        log::info!("Using adapter: {:?}", adapter.get_info());

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
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Emerald Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        // Create uniform buffer
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

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

        // Generate dodecahedron geometry
        let (vertices, indices) = generate_dodecahedron();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let depth_texture = Self::create_depth_texture(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            start_time: Instant::now(),
            window,
            depth_texture,
        }
    }

    fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width.max(1),
                height: config.height.max(1),
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

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Self::create_depth_texture(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let aspect = self.config.width as f32 / self.config.height as f32;
        self.uniforms.update(elapsed, aspect);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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
                            r: 0.02,
                            g: 0.02,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
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
            .with_title("Emerald Dodecahedron - wgpu + Rust")
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

// Phong lighting shader for emerald material
const SHADER: &str = r#"
struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    light_pos: vec4<f32>,
    view_pos: vec4<f32>,
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
    shininess: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_pos = uniforms.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos.xyz;
    
    // Transform normal to world space (using normal matrix approximation)
    let normal_matrix = mat3x3<f32>(
        uniforms.model[0].xyz,
        uniforms.model[1].xyz,
        uniforms.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * in.normal);
    
    out.clip_position = uniforms.proj * uniforms.view * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_pos = uniforms.light_pos.xyz;
    let view_pos = uniforms.view_pos.xyz;
    
    // Light color (warm white)
    let light_color = vec3<f32>(1.0, 0.95, 0.9);
    
    // Ambient
    let ambient = uniforms.ambient.rgb * light_color * 0.3;
    
    // Diffuse
    let light_dir = normalize(light_pos - in.world_pos);
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * uniforms.diffuse.rgb * light_color;
    
    // Specular (Blinn-Phong)
    let view_dir = normalize(view_pos - in.world_pos);
    let halfway_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, halfway_dir), 0.0), uniforms.shininess);
    let specular = spec * uniforms.specular.rgb * light_color;
    
    // Add slight fresnel effect for gem-like appearance
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    let rim = fresnel * uniforms.specular.rgb * 0.3;
    
    let result = ambient + diffuse + specular + rim;
    
    // Tone mapping
    let mapped = result / (result + vec3<f32>(1.0));
    
    return vec4<f32>(mapped, 1.0);
}
"#;

fn main() {
    env_logger::init();
    log::info!("Starting Emerald Dodecahedron application");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}