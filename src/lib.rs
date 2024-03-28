use log::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

mod vertex;
use vertex::Vertex;


//       C       I
//     D   B   J   H
//   E       A       G
//
//
//           F
#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [   0.0,  -0.60, 0.0],  color: [0.5, 0.0, 0.5] }, // F 0
    Vertex { position: [   0.0,   0.20, 0.0],  color: [0.5, 0.0, 0.5] }, // A 1
    Vertex { position: [ -0.65,   0.20, 0.0],  color: [0.5, 0.0, 0.5] }, // E 2
    Vertex { position: [  0.65,   0.20, 0.0],  color: [0.5, 0.0, 0.5] }, // G 3
    Vertex { position: [ -0.12,   0.41, 0.0],  color: [0.5, 0.0, 0.5] }, // B 4
    Vertex { position: [  0.12,   0.41, 0.0],  color: [0.5, 0.0, 0.5] }, // J 5
    Vertex { position: [ -0.61, 0.4025, 0.0],  color: [0.5, 0.0, 0.5] }, // D 6
    Vertex { position: [  0.61, 0.4025, 0.0],  color: [0.5, 0.0, 0.5] }, // H 7
    Vertex { position: [-0.375,  0.575, 0.0],  color: [0.5, 0.0, 0.5] }, // C 8
    Vertex { position: [ 0.375,  0.575, 0.0],  color: [0.5, 0.0, 0.5] }, // I 9
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
    // ABC
    1, 4, 8,
    // CDE
    8, 6, 2,
    // ACE
    1, 8, 2,
    // GHI
    3, 7, 9,
    // IJA
    9, 5, 1,
    // IAG
    9, 1, 3,
    // EFG
    2, 0, 3,
];

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to create window");

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut state = State::new(&window).await;

    // if #[cfg(target_arch = "wasm32")] {
    //     use winit::platform::web::EventLoopExtWebSys;
    //     let event_loop_function = EventLoop::spawn;
    // } else {
    let event_loop_function = EventLoop::run;
    // }

    event_loop_function(
        event_loop,
        |evt, elwt: &EventLoopWindowTarget<()>| match evt {
            Event::WindowEvent { window_id, event }
                if window_id == window.id() && !state.input(&event) =>
            {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => elwt.exit(),
                    WindowEvent::Resized(physical_size) => state.resize(physical_size),
                    WindowEvent::RedrawRequested => {
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => error!("{:?}", e),
                        }

                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        },
    )
    .expect("Failed to run event loop")
}

struct State<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    curr_pipeline_idx: usize,
    render_pipelines: [wgpu::RenderPipeline; 3],

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    last_cursor_position: winit::dpi::PhysicalPosition<f64>,
    color: wgpu::Color,
    is_color_locked: bool,
}

impl<'window> State<'window> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'window Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to get adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits:
                    // if cfg!(target_arch = "wasm32") {
                    //     wgpu::Limits::downlevel_webgl2_defaults()
                    // } else {
                        wgpu::Limits::default(),
                    // },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .expect("Failed to get device/queue");

        // Shader code assumes an sRGB surface texture.
        // Using a different one will result in all the colors coming out darker (which would need to be accounted for when drawing to the frame, if we want to support non sRGB surfaces)
        // let surface_caps = surface.get_capabilities(&adapter);
        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .copied()
        //     .filter(|f| f.is_srgb())
        //     .next()
        //     .unwrap_or(surface_caps.formats[0]);
        // let config = wgpu::SurfaceConfiguration {
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        //     format: surface_format,
        //     width: size.width,
        //     height: size.height,
        //     present_mode: wgpu::PresentMode::Fifo,
        //     alpha_mode: surface_caps.alpha_modes[0],
        //     view_formats: vec![],
        //     desired_maximum_frame_latency: 2,
        // };

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Failed to get default config");
        surface.configure(&device, &config);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let fragment_targets = [Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let triangle_single_color_shader =
            device.create_shader_module(wgpu::include_wgsl!("shader-triangle-single-color.wgsl"));

        let mut render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline 1"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &triangle_single_color_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &triangle_single_color_shader,
                entry_point: "fs_main",
                targets: &fragment_targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };
        let render_pipeline_1 = device.create_render_pipeline(&render_pipeline_descriptor);

        let triangle_multi_color_shader =
            device.create_shader_module(wgpu::include_wgsl!("shader-triangle-multi-color.wgsl"));
        render_pipeline_descriptor.label = Some("Render Pipeline 2");
        render_pipeline_descriptor.vertex.module = &triangle_multi_color_shader;
        render_pipeline_descriptor.fragment.as_mut().unwrap().module = &triangle_multi_color_shader;
        let render_pipeline_2 = device.create_render_pipeline(&render_pipeline_descriptor);

        let heart_shader = device.create_shader_module(wgpu::include_wgsl!("shader-heart.wgsl"));
        let vertex_buffer = [Vertex::buffer_layout()];
        render_pipeline_descriptor.label = Some("Render Pipeline 3");
        render_pipeline_descriptor.vertex.module = &heart_shader;
        render_pipeline_descriptor.vertex.buffers = &vertex_buffer;
        render_pipeline_descriptor.fragment.as_mut().unwrap().module = &heart_shader;
        let render_pipeline_3 = device.create_render_pipeline(&render_pipeline_descriptor);

        let render_pipelines = [render_pipeline_1, render_pipeline_2, render_pipeline_3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            curr_pipeline_idx: render_pipelines.len() - 1,
            render_pipelines,
            vertex_buffer,
            index_buffer,
            num_indices,
            color: wgpu::Color::WHITE,
            is_color_locked: false,
            last_cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, window_event: &WindowEvent) -> bool {
        match window_event {
            WindowEvent::CursorMoved { position, .. } => {
                self.last_cursor_position = *position;

                self.change_color(wgpu::Color {
                    r: self.last_cursor_position.x as f64 / self.size.width as f64,
                    b: self.last_cursor_position.y as f64 / self.size.height as f64,
                    ..self.color
                });
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        self.change_color(wgpu::Color {
                            g: (self.color.g + *y as f64 / 5.0).clamp(0.0, 1.0),
                            ..self.color
                        });
                    }
                    _ => {}
                }
                true
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                self.is_color_locked = !self.is_color_locked;

                self.change_color(wgpu::Color {
                    r: self.last_cursor_position.x as f64 / self.size.width as f64,
                    b: self.last_cursor_position.y as f64 / self.size.height as f64,
                    ..self.color
                });

                true
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.curr_pipeline_idx = (self.curr_pipeline_idx + 1) % self.render_pipelines.len();
                true
            }
            _ => false,
        }
    }

    fn change_color(&mut self, color: wgpu::Color) {
        if !self.is_color_locked {
            self.color = color;
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                        load: wgpu::LoadOp::Clear(self.color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipelines[self.curr_pipeline_idx]);

            if self.curr_pipeline_idx == 0 || self.curr_pipeline_idx == 1 {
                render_pass.draw(0..3, 0..1);
            } else {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
