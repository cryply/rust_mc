use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
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

/// Generate a torus (ring/toroid)
/// major_radius: distance from center of torus to center of tube
/// minor_radius: radius of the tube
/// major_segments: segments around the ring
/// minor_segments: segments around the tube
fn generate_torus(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=major_segments {
        let u = (i as f32 / major_segments as f32) * 2.0 * PI;
        let cos_u = u.cos();
        let sin_u = u.sin();

        for j in 0..=minor_segments {
            let v = (j as f32 / minor_segments as f32) * 2.0 * PI;
            let cos_v = v.cos();
            let sin_v = v.sin();

            // Position on torus surface
            let x = (major_radius + minor_radius * cos_v) * cos_u;
            let y = minor_radius * sin_v;
            let z = (major_radius + minor_radius * cos_v) * sin_u;

            // Normal vector (points outward from tube surface)
            let nx = cos_v * cos_u;
            let ny = sin_v;
            let nz = cos_v * sin_u;

            vertices.push(Vertex {
                position: [x, y, z],
                normal: [nx, ny, nz],
            });
        }
    }

    // Generate indices
    for i in 0..major_segments {
        for j in 0..minor_segments {
            let a = i * (minor_segments + 1) + j;
            let b = a + minor_segments + 1;
            let c = a + 1;
            let d = b + 1;

            // Two triangles per quad
            indices.push(a as u16);
            indices.push(b as u16);
            indices.push(c as u16);

            indices.push(c as u16);
            indices.push(b as u16);
            indices.push(d as u16);
        }
    }

    (vertices, indices)
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    light_pos: [f32; 4],
    view_pos: [f32; 4],
    // Gold material properties
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    _padding: [f32; 2],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            light_pos: [2.0, 3.0, 2.0, 1.0],
            view_pos: [0.0, 0.0, 4.0, 1.0],
            // Rich saturated gold color
            base_color: [0.83, 0.55, 0.1, 1.0],
            metallic: 1.0,
            roughness: 0.25,
            _padding: [0.0; 2],
        }
    }

    fn update(&mut self, rotation: f32, aspect: f32) {
        // Tilt the ring and rotate
        let model = Mat4::from_rotation_y(rotation) 
            * Mat4::from_rotation_x(0.4)
            * Mat4::from_rotation_z(rotation * 0.3);
        self.model = model.to_cols_array_2d();

        let eye = Vec3::new(0.0, 0.0, 4.0);
        let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
        self.view = view.to_cols_array_2d();

        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        self.proj = proj.to_cols_array_2d();

        self.view_pos = [eye.x, eye.y, eye.z, 1.0];
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gold Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

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

        // Generate torus: major radius 0.7, minor radius 0.25, high detail
        let (vertices, indices) = generate_torus(0.7, 0.25, 64, 32);

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

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
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
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
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
            .with_title("Golden Ring - wgpu + Rust")
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

// PBR-inspired metallic gold shader with directional lighting
const SHADER: &str = r#"
struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    light_pos: vec4<f32>,
    view_pos: vec4<f32>,
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
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
    let view_dir = normalize(uniforms.view_pos.xyz - in.world_pos);
    
    // Gold color
    let gold = uniforms.base_color.rgb;
    let roughness = uniforms.roughness;
    
    // Main directional light from upper right
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.5));
    let light_color = vec3<f32>(1.0, 0.95, 0.8);
    
    // Diffuse (Lambert) - this creates the main shading
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let diffuse = n_dot_l * gold * light_color;
    
    // Specular (Blinn-Phong) - sharp highlight
    let halfway = normalize(light_dir + view_dir);
    let n_dot_h = max(dot(normal, halfway), 0.0);
    let spec_power = (1.0 - roughness) * 128.0 + 8.0;
    let specular = pow(n_dot_h, spec_power) * light_color * 0.8;
    
    // Secondary light from lower left (cool tint) - weaker
    let light2_dir = normalize(vec3<f32>(-1.0, -0.5, 0.3));
    let light2_color = vec3<f32>(0.3, 0.4, 0.5);
    let n_dot_l2 = max(dot(normal, light2_dir), 0.0);
    let diffuse2 = n_dot_l2 * gold * light2_color * 0.3;
    
    // Fresnel rim for metallic look
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 5.0);
    let rim = fresnel * gold * 0.4;
    
    // No ambient! Only directional lights
    var color = diffuse + specular + diffuse2 + rim;
    
    // Tone mapping
    color = color / (color + vec3<f32>(1.0));
    
    // Gamma
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, 1.0);
}
"#;

fn main() {
    env_logger::init();
    log::info!("Starting Golden Ring application");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}