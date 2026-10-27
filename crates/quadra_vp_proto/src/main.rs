// M1 viewport prototype. Draws a generated 10M-triangle scene (1,000 objects,
// 10% of them instances of one shared mesh) with an orbiting camera and logs
// frame times, so we know wgpu can hold 60 FPS before building the real viewport.
//
// Run: cargo run --release -p quadra_vp_proto [-- --seconds N]
// With --seconds it quits after N seconds and prints a summary.

use std::sync::Arc;
use std::time::{Duration, Instant};

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const OBJECTS: usize = 1_000;
const INSTANCED: usize = OBJECTS / 10;
// Each mesh is a deformed sphere with RINGS x SEGMENTS quads = 10,000 triangles.
const RINGS: u32 = 50;
const SEGMENTS: u32 = 100;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
}

// Per-object data sent as an instance-rate vertex buffer: position, scale, colour.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Object {
    offset_scale: [f32; 4],
    colour: [f32; 4],
}

// Small deterministic random numbers so every run draws the same scene.
struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }
}

// A sphere with a per-mesh bumpy surface, so the unique meshes really differ.
fn sphere(seed: u32, verts: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
    let mut rng = Rng(seed.wrapping_mul(2_654_435_761) | 1);
    let (fa, fb, amp) = (2.0 + rng.next() * 6.0, 2.0 + rng.next() * 6.0, 0.05 + rng.next() * 0.15);
    for r in 0..=RINGS {
        let theta = std::f32::consts::PI * r as f32 / RINGS as f32;
        for s in 0..=SEGMENTS {
            let phi = std::f32::consts::TAU * s as f32 / SEGMENTS as f32;
            let n = [theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin()];
            let radius = 1.0 + amp * (fa * theta).sin() * (fb * phi).cos();
            verts.push(Vertex { pos: n.map(|c| c * radius), normal: n });
        }
    }
    let row = SEGMENTS + 1;
    for r in 0..RINGS {
        for s in 0..SEGMENTS {
            let a = r * row + s;
            let b = a + row;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
}

fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    // Right-handed, depth 0..1 (wgpu convention), column-major.
    let f = 1.0 / (fovy / 2.0).tan();
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, near * far / (near - far), 0.0],
    ]
}

fn look_at(eye: [f32; 3], target: [f32; 3]) -> [[f32; 4]; 4] {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let norm = |a: [f32; 3]| {
        let l = dot(a, a).sqrt();
        a.map(|c| c / l)
    };
    let f = norm(sub(target, eye));
    let s = norm(cross(f, [0.0, 1.0, 0.0]));
    let u = cross(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0],
    ]
}

fn mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0; 4]; 4];
    for c in 0..4 {
        for r in 0..4 {
            out[c][r] = (0..4).map(|k| a[k][r] * b[c][k]).sum();
        }
    }
    out
}

const SHADER: &str = r#"
struct Camera { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) colour: vec3<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) normal: vec3<f32>,
           @location(2) offset_scale: vec4<f32>, @location(3) colour: vec4<f32>) -> Out {
    var o: Out;
    o.clip = camera.view_proj * vec4<f32>(pos * offset_scale.w + offset_scale.xyz, 1.0);
    o.normal = normal;
    o.colour = colour.rgb;
    return o;
}

@fragment
fn fs_main(i: Out) -> @location(0) vec4<f32> {
    let light = max(dot(normalize(i.normal), normalize(vec3<f32>(0.4, 0.8, 0.3))), 0.0);
    return vec4<f32>(i.colour * (0.25 + 0.75 * light), 1.0);
}
"#;

struct Gpu {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    camera_buf: wgpu::Buffer,
    camera_bind: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    objects: wgpu::Buffer,
}

fn depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

impl Gpu {
    fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance.create_surface(window.clone()).expect("create surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("no GPU adapter");
        let info = adapter.get_info();
        println!("GPU: {} ({:?})", info.name, info.backend);

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("request device");

        let size = window.inner_size();
        let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).expect("surface config");
        // Uncapped, so the log shows real headroom above 60 FPS.
        config.present_mode = wgpu::PresentMode::AutoNoVsync;
        surface.configure(&device, &config);

        // 900 unique meshes, then one shared mesh drawn 100 times.
        let (mut verts, mut idx) = (Vec::new(), Vec::new());
        for m in 0..=(OBJECTS - INSTANCED) as u32 {
            sphere(m, &mut verts, &mut idx);
        }
        let mut rng = Rng(7);
        let objects: Vec<Object> = (0..OBJECTS)
            .map(|_| Object {
                offset_scale: [rng.next() * 200.0 - 100.0, rng.next() * 20.0, rng.next() * 200.0 - 100.0, 1.0 + rng.next() * 3.0],
                colour: [0.3 + rng.next() * 0.7, 0.3 + rng.next() * 0.7, 0.3 + rng.next() * 0.7, 1.0],
            })
            .collect();
        println!("Scene: {} objects, {} triangles", OBJECTS, OBJECTS as u64 * (RINGS * SEGMENTS * 2) as u64);

        let init = |label, contents: &[u8], usage| device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(label), contents, usage });
        let vertices = init("vertices", bytemuck::cast_slice(&verts), wgpu::BufferUsages::VERTEX);
        let indices = init("indices", bytemuck::cast_slice(&idx), wgpu::BufferUsages::INDEX);
        let objects = init("objects", bytemuck::cast_slice(&objects), wgpu::BufferUsages::VERTEX);
        let camera_buf = init("camera", &[0u8; 64], wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let camera_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera_buf.as_entire_binding() }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<Object>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4],
                    }),
                ],
            },
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(config.format.into())],
            }),
            multiview_mask: None,
            cache: None,
        });

        let depth = depth_view(&device, &config);
        Gpu { window, surface, device, queue, config, depth, pipeline, camera_buf, camera_bind, vertices, indices, objects }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth = depth_view(&self.device, &self.config);
    }

    fn draw(&mut self, t: f32) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let aspect = self.config.width as f32 / self.config.height as f32;
        let eye = [180.0 * (t * 0.3).cos(), 80.0, 180.0 * (t * 0.3).sin()];
        let view_proj = mul(perspective(0.8, aspect, 0.5, 1000.0), look_at(eye, [0.0, 0.0, 0.0]));
        self.queue.write_buffer(&self.camera_buf, 0, bytemuck::cast_slice(&view_proj));

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.24, g: 0.24, b: 0.24, a: 1.0 }), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.camera_bind, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_vertex_buffer(1, self.objects.slice(..));
            pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
            let index_count = RINGS * SEGMENTS * 6;
            let vertex_count = ((RINGS + 1) * (SEGMENTS + 1)) as i32;
            let unique = (OBJECTS - INSTANCED) as u32;
            // One draw per unique mesh, then one instanced draw for the shared mesh.
            for m in 0..unique {
                pass.draw_indexed(m * index_count..(m + 1) * index_count, m as i32 * vertex_count, m..m + 1);
            }
            pass.draw_indexed(unique * index_count..(unique + 1) * index_count, unique as i32 * vertex_count, unique..OBJECTS as u32);
        }
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
    }
}

#[derive(Default)]
struct Stats {
    frames: Vec<f32>,
    all: Vec<f32>,
    last_report: Option<Instant>,
}

impl Stats {
    fn add(&mut self, ms: f32) {
        self.frames.push(ms);
        self.all.push(ms);
        let now = Instant::now();
        let since = *self.last_report.get_or_insert(now);
        if now - since >= Duration::from_secs(1) {
            println!("{}", summary(&mut self.frames));
            self.frames.clear();
            self.last_report = Some(now);
        }
    }
}

fn summary(ms: &mut [f32]) -> String {
    ms.sort_by(f32::total_cmp);
    let avg = ms.iter().sum::<f32>() / ms.len() as f32;
    let p99 = ms[(ms.len() * 99 / 100).min(ms.len() - 1)];
    format!("{:6.1} FPS  avg {:5.2} ms  p99 {:5.2} ms  worst {:5.2} ms  ({} frames)", 1000.0 / avg, avg, p99, ms[ms.len() - 1], ms.len())
}

struct App {
    gpu: Option<Gpu>,
    start: Instant,
    last_frame: Option<Instant>,
    stats: Stats,
    quit_after: Option<Duration>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        // Opens behind the active window so test runs never steal focus.
        let attrs = Window::default_attributes()
            .with_title("Quadra viewport prototype")
            .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 900.0))
            .with_active(false);
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        let t = Instant::now();
        self.gpu = Some(Gpu::new(window));
        println!("Setup took {:.2} s", t.elapsed().as_secs_f32());
        self.start = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(gpu) = self.gpu.as_mut() else { return };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                gpu.draw(self.start.elapsed().as_secs_f32());
                let now = Instant::now();
                if let Some(prev) = self.last_frame.replace(now) {
                    self.stats.add((now - prev).as_secs_f32() * 1000.0);
                }
                if self.quit_after.is_some_and(|d| self.start.elapsed() >= d) {
                    event_loop.exit();
                }
                gpu.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quit_after = args.iter().position(|a| a == "--seconds").and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).map(Duration::from_secs);

    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App { gpu: None, start: Instant::now(), last_frame: None, stats: Stats::default(), quit_after };
    event_loop.run_app(&mut app).expect("run");

    if !app.stats.all.is_empty() {
        // Skip the first second: shader compile and upload are not steady-state frames.
        let warm: usize = app.stats.all.iter().scan(0.0, |acc, ms| { *acc += ms; Some(*acc) }).take_while(|acc| *acc < 1000.0).count();
        let steady = &mut app.stats.all[warm..];
        if !steady.is_empty() {
            println!("Overall (after first second): {}", summary(steady));
        }
    }
}
