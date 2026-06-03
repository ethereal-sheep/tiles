use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};
use winit::window::Window;

use crate::cell::{CellInstance, LightData};

const MAX_INSTANCES: usize = 131072;
const MAX_LIGHTS: usize = 64;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    projection: [[f32; 4]; 4],
    viewport_offset: [f32; 2],
    viewport_size: [f32; 2],
    viewport_cells: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct LightUniforms {
    lights: [LightData; MAX_LIGHTS],
    light_count: u32,
    ambient: f32,
    _pad: [f32; 2],
}

const SHADER_SRC: &str = r#"
struct Uniforms {
    projection: mat4x4<f32>,
    viewport_offset: vec2<f32>,
    viewport_size: vec2<f32>,
    viewport_cells: vec2<f32>,
    _pad: vec2<f32>,
}

struct LightData {
    position: vec2<f32>,
    radius: f32,
    intensity: f32,
    color: vec3<f32>,
    _pad: f32,
}

struct LightUniforms {
    lights: array<LightData, 64>,
    light_count: u32,
    ambient: f32,
    _pad: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> light_uniforms: LightUniforms;

struct InstanceIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rotation: vec4<f32>,
    @location(3) emissive: f32,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec2<f32>,
    @location(2) emissive: f32,
}

fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let u = q.xyz;
    let s = q.w;
    return 2.0 * dot(u, v) * u
         + (s * s - dot(u, u)) * v
         + 2.0 * s * cross(u, v);
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, instance: InstanceIn) -> VertexOut {
    var unit_quad: array<vec2<f32>, 6>;
    unit_quad[0] = vec2<f32>(0.0, 0.0);
    unit_quad[1] = vec2<f32>(1.0, 0.0);
    unit_quad[2] = vec2<f32>(1.0, 1.0);
    unit_quad[3] = vec2<f32>(0.0, 0.0);
    unit_quad[4] = vec2<f32>(1.0, 1.0);
    unit_quad[5] = vec2<f32>(0.0, 1.0);

    let offset = unit_quad[vi] - vec2<f32>(0.5, 0.5);
    let local_pos = vec3<f32>(offset.x, offset.y, 0.0);
    let rotated = quat_rotate(instance.rotation, local_pos);
    let world_pos = vec4<f32>(
        instance.position + rotated,
        1.0
    );

    var out: VertexOut;
    out.clip_position = uniforms.projection * world_pos;
    out.color = instance.color;
    out.world_pos = instance.position.xy;
    out.emissive = instance.emissive;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var illumination = light_uniforms.ambient;

    if in.emissive < 0.5 {
        // Non-emissive cells are affected by lights
        for (var i = 0u; i < light_uniforms.light_count; i = i + 1u) {
            let light = light_uniforms.lights[i];
            if light.radius <= 0.0 {
                continue;
            }
            let d = distance(in.world_pos, light.position);
            let falloff = 1.0 - smoothstep(0.0, light.radius, d);
            illumination += falloff * light.intensity;
        }
        illumination = clamp(illumination, 0.0, 1.0);
    } else {
        illumination = 1.0;
    }

    return vec4<f32>(in.color.rgb * illumination, in.color.a);
}

// Screen-space vertex shader: top-left origin, Y-down, viewport units
@vertex
fn vs_screen(@builtin(vertex_index) vi: u32, instance: InstanceIn) -> VertexOut {
    var unit_quad: array<vec2<f32>, 6>;
    unit_quad[0] = vec2<f32>(0.0, 0.0);
    unit_quad[1] = vec2<f32>(1.0, 0.0);
    unit_quad[2] = vec2<f32>(1.0, 1.0);
    unit_quad[3] = vec2<f32>(0.0, 0.0);
    unit_quad[4] = vec2<f32>(1.0, 1.0);
    unit_quad[5] = vec2<f32>(0.0, 1.0);

    let offset = unit_quad[vi] - vec2<f32>(0.5, 0.5);
    let local_pos = vec3<f32>(offset.x, offset.y, 0.0);
    let rotated = quat_rotate(instance.rotation, local_pos);

    // Convert top-left Y-down to NDC: x: [0, vp_w] -> [-1, 1], y: [0, vp_h] -> [1, -1]
    let world_pos = instance.position + rotated;
    let ndc_x = (world_pos.x / uniforms.viewport_cells.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (world_pos.y / uniforms.viewport_cells.y) * 2.0;

    var out: VertexOut;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = instance.color;
    out.world_pos = instance.position.xy;
    out.emissive = 1.0;
    return out;
}

@fragment
fn fs_screen(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}

// Bloom vertex shader: expands quad based on light radius
struct BloomInstanceIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rotation: vec4<f32>,
    @location(3) emissive: f32,
}

struct BloomVertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_uv: vec2<f32>,
}

@vertex
fn vs_bloom(@builtin(vertex_index) vi: u32, instance: BloomInstanceIn) -> BloomVertexOut {
    var unit_quad: array<vec2<f32>, 6>;
    unit_quad[0] = vec2<f32>(0.0, 0.0);
    unit_quad[1] = vec2<f32>(1.0, 0.0);
    unit_quad[2] = vec2<f32>(1.0, 1.0);
    unit_quad[3] = vec2<f32>(0.0, 0.0);
    unit_quad[4] = vec2<f32>(1.0, 1.0);
    unit_quad[5] = vec2<f32>(0.0, 1.0);

    // emissive field stores the bloom radius for bloom instances
    let bloom_radius = instance.emissive;
    let scale = bloom_radius * 2.0 + 1.0;
    let offset = (unit_quad[vi] - vec2<f32>(0.5, 0.5)) * scale;
    let world_pos = vec4<f32>(
        instance.position.x + offset.x,
        instance.position.y + offset.y,
        instance.position.z - 0.01,
        1.0
    );

    var out: BloomVertexOut;
    out.clip_position = uniforms.projection * world_pos;
    out.color = instance.color;
    out.local_uv = unit_quad[vi] - vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_bloom(in: BloomVertexOut) -> @location(0) vec4<f32> {
    let d = length(in.local_uv) * 2.0;
    let falloff = 1.0 - smoothstep(0.0, 1.0, d);
    let alpha = falloff * in.color.a;
    if alpha < 0.001 {
        discard;
    }
    return vec4<f32>(in.color.rgb, alpha);
}
"#;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    opaque_pipeline: wgpu::RenderPipeline,
    transparent_pipeline: wgpu::RenderPipeline,
    bloom_pipeline: wgpu::RenderPipeline,
    screen_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    light_uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth_texture: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No adapter found");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Device creation failed");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tiles_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light_uniform_buffer"),
            size: std::mem::size_of::<LightUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tiles_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tiles_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tiles_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CellInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position: vec3 at offset 0
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                // color: vec4 at offset 16 (after position + pad)
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 1,
                },
                // rotation: vec4 at offset 32 (after color)
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 2,
                },
                // emissive: f32 at offset 48
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 48,
                    shader_location: 3,
                },
            ],
        };

        let depth_format = wgpu::TextureFormat::Depth32Float;

        let opaque_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("opaque_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[instance_layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let transparent_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("transparent_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[instance_layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let bloom_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_bloom",
                buffers: &[instance_layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_bloom",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let screen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("screen_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_screen",
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_screen",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance_buffer"),
            size: (MAX_INSTANCES * std::mem::size_of::<CellInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let depth_texture = Self::create_depth_texture(&device, config.width, config.height);

        Self {
            surface,
            device,
            queue,
            config,
            opaque_pipeline,
            transparent_pipeline,
            bloom_pipeline,
            screen_pipeline,
            instance_buffer,
            uniform_buffer,
            light_uniform_buffer,
            bind_group,
            depth_texture,
        }
    }

    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Self::create_depth_texture(&self.device, w, h);
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.config.present_mode = if vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(
        &mut self,
        opaque: &[CellInstance],
        transparent: &[CellInstance],
        screen: &[CellInstance],
        lights: &[LightData],
        bloom_sources: &[LightData],
        ambient: f32,
        projection: Mat4,
        viewport_offset: Vec2,
        viewport_size: Vec2,
        viewport_cells: Vec2,
        window_bg: [f32; 4],
        viewport_bg: [f32; 4],
    ) -> Result<(), wgpu::SurfaceError> {
        let uniforms = Uniforms {
            projection: projection.to_cols_array_2d(),
            viewport_offset: viewport_offset.to_array(),
            viewport_size: viewport_size.to_array(),
            viewport_cells: viewport_cells.to_array(),
            _pad: [0.0; 2],
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Write light uniforms
        let mut light_uniforms = LightUniforms {
            lights: [LightData { position: [0.0; 2], radius: 0.0, intensity: 0.0, color: [0.0; 3], _pad: 0.0 }; MAX_LIGHTS],
            light_count: lights.len().min(MAX_LIGHTS) as u32,
            ambient,
            _pad: [0.0; 2],
        };
        for (i, light) in lights.iter().take(MAX_LIGHTS).enumerate() {
            light_uniforms.lights[i] = *light;
        }
        self.queue.write_buffer(&self.light_uniform_buffer, 0, bytemuck::bytes_of(&light_uniforms));

        // Build bloom instances from bloom sources
        let mut bloom_instances: Vec<CellInstance> = Vec::new();
        for source in bloom_sources.iter() {
            bloom_instances.push(CellInstance {
                position: [source.position[0], source.position[1], 0.0],
                _pad0: 0.0,
                color: [source.color[0], source.color[1], source.color[2], source.intensity * 0.4],
                rotation: [0.0, 0.0, 0.0, 1.0],
                emissive: source.radius,
                _pad1: [0.0; 3],
            });
        }

        // Upload all instances to buffer sequentially
        let world_count = opaque.len() + transparent.len() + bloom_instances.len();
        let total_instances = world_count + screen.len();
        assert!(total_instances <= MAX_INSTANCES, "Too many instances: {total_instances}");
        self.upload_instances(opaque, 0);
        self.upload_instances(transparent, opaque.len());
        self.upload_instances(&bloom_instances, opaque.len() + transparent.len());
        self.upload_instances(screen, world_count);

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });

        // Clear full window with window background color
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: window_bg[0] as f64,
                            g: window_bg[1] as f64,
                            b: window_bg[2] as f64,
                            a: window_bg[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Main render pass with viewport scissor
        let vp_x = viewport_offset.x as u32;
        let vp_y = viewport_offset.y as u32;
        let vp_w = viewport_size.x as u32;
        let vp_h = viewport_size.y as u32;

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: viewport_bg[0] as f64,
                            g: viewport_bg[1] as f64,
                            b: viewport_bg[2] as f64,
                            a: viewport_bg[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
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
            });

            pass.set_scissor_rect(vp_x, vp_y, vp_w.max(1), vp_h.max(1));
            pass.set_viewport(
                vp_x as f32, vp_y as f32,
                vp_w as f32, vp_h as f32,
                0.0, 1.0,
            );
            pass.set_bind_group(0, &self.bind_group, &[]);

            // Opaque pass
            if !opaque.is_empty() {
                pass.set_pipeline(&self.opaque_pipeline);
                Self::draw_range(&mut pass, &self.instance_buffer, 0, opaque.len());
            }

            // Transparent pass
            if !transparent.is_empty() {
                pass.set_pipeline(&self.transparent_pipeline);
                Self::draw_range(&mut pass, &self.instance_buffer, opaque.len(), transparent.len());
            }

            // Bloom pass (additive)
            if !bloom_instances.is_empty() {
                pass.set_pipeline(&self.bloom_pipeline);
                Self::draw_range(&mut pass, &self.instance_buffer, opaque.len() + transparent.len(), bloom_instances.len());
            }
        }

        // Screen-space pass: unlit, no depth, alpha-blend in draw order
        if !screen.is_empty() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("screen_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_scissor_rect(vp_x, vp_y, vp_w.max(1), vp_h.max(1));
            pass.set_viewport(
                vp_x as f32, vp_y as f32,
                vp_w as f32, vp_h as f32,
                0.0, 1.0,
            );
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_pipeline(&self.screen_pipeline);
            Self::draw_range(&mut pass, &self.instance_buffer, world_count, screen.len());
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn upload_instances(&self, instances: &[CellInstance], offset: usize) {
        if instances.is_empty() {
            return;
        }
        let byte_offset = (offset * std::mem::size_of::<CellInstance>()) as u64;
        let data = bytemuck::cast_slice(instances);
        self.queue.write_buffer(&self.instance_buffer, byte_offset, data);
    }

    fn draw_range<'a>(
        pass: &mut wgpu::RenderPass<'a>,
        buffer: &'a wgpu::Buffer,
        offset: usize,
        count: usize,
    ) {
        let stride = std::mem::size_of::<CellInstance>() as u64;
        let byte_offset = (offset as u64) * stride;
        let byte_end = byte_offset + (count as u64) * stride;
        pass.set_vertex_buffer(0, buffer.slice(byte_offset..byte_end));
        pass.draw(0..6, 0..count as u32);
    }

    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    pub fn height(&self) -> f32 {
        self.config.height as f32
    }
}
